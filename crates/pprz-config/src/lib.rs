//! Configuration-domain types for Paparazzi-compatible tools.

use quick_xml::{Reader, events::Event};

/// Identifies an airframe configuration without interpreting its XML yet.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AirframeId(String);

impl AirframeId {
    /// Validates and creates a non-empty airframe identifier.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::EmptyAirframeId`] when `value` contains no
    /// visible characters.
    pub fn new(value: impl Into<String>) -> Result<Self, ConfigError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(ConfigError::EmptyAirframeId);
        }
        Ok(Self(value))
    }

    /// Returns the identifier as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Errors raised while validating configuration values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfigError {
    /// An airframe identifier contained no visible characters.
    EmptyAirframeId,
}

/// A parsed subset of a Paparazzi airframe configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Airframe {
    /// The human-readable airframe name.
    pub name: AirframeId,
    /// The selected firmware and its declared build targets.
    pub firmware: Firmware,
}

/// A firmware declaration within an airframe configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Firmware {
    /// The Paparazzi firmware family, such as `rotorcraft` or `fixedwing`.
    pub name: String,
    /// Build targets declared by this firmware.
    pub targets: Vec<Target>,
}

/// A named firmware build target.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Target {
    /// The target name, for example `ap` or `nps`.
    pub name: String,
    /// The board used by this target.
    pub board: String,
}

/// Errors raised while parsing an airframe XML document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    /// The XML document was malformed or otherwise unreadable.
    Xml(String),
    /// The document did not contain an `airframe` root with a name.
    MissingAirframeName,
    /// The document did not declare a firmware name.
    MissingFirmwareName,
    /// A required target attribute was missing.
    MissingTargetAttribute(&'static str),
    /// The root airframe name was not a valid identifier.
    InvalidAirframeName(ConfigError),
}

/// Parses the initial compatibility subset of a Paparazzi airframe XML file.
///
/// The initial subset intentionally reads the airframe name, the first firmware
/// declaration, and that firmware's targets. Other Paparazzi configuration
/// elements remain preserved as future migration work.
///
/// # Errors
///
/// Returns [`ParseError`] when XML is malformed or the required airframe,
/// firmware, or target attributes are absent.
pub fn parse_airframe(xml: &str) -> Result<Airframe, ParseError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buffer = Vec::new();
    let mut airframe_name = None;
    let mut firmware_name = None;
    let mut targets = Vec::new();
    let mut firmware_depth = 0_usize;

    loop {
        match reader
            .read_event_into(&mut buffer)
            .map_err(|error| ParseError::Xml(error.to_string()))?
        {
            Event::Start(element) => match element.name().as_ref() {
                b"airframe" => {
                    airframe_name = attribute(&element, b"name")?;
                }
                b"firmware" if firmware_name.is_none() => {
                    firmware_name = attribute(&element, b"name")?;
                    firmware_depth = 1;
                }
                b"firmware" if firmware_depth > 0 => firmware_depth += 1,
                b"target" if firmware_depth > 0 => targets.push(parse_target(&element)?),
                _ => {}
            },
            Event::Empty(element) if element.name().as_ref() == b"target" && firmware_depth > 0 => {
                targets.push(parse_target(&element)?);
            }
            Event::End(element) if element.name().as_ref() == b"firmware" && firmware_depth > 0 => {
                firmware_depth -= 1;
            }
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }

    let name = airframe_name.ok_or(ParseError::MissingAirframeName)?;
    let name = AirframeId::new(name).map_err(ParseError::InvalidAirframeName)?;
    let firmware = Firmware {
        name: firmware_name.ok_or(ParseError::MissingFirmwareName)?,
        targets,
    };
    Ok(Airframe { name, firmware })
}

fn parse_target(element: &quick_xml::events::BytesStart<'_>) -> Result<Target, ParseError> {
    let name = attribute(element, b"name")?.ok_or(ParseError::MissingTargetAttribute("name"))?;
    let board = attribute(element, b"board")?.ok_or(ParseError::MissingTargetAttribute("board"))?;
    Ok(Target { name, board })
}

fn attribute(
    element: &quick_xml::events::BytesStart<'_>,
    name: &[u8],
) -> Result<Option<String>, ParseError> {
    element
        .try_get_attribute(name)
        .map_err(|error| ParseError::Xml(error.to_string()))?
        .map(|attribute| {
            attribute
                .unescape_value()
                .map(std::borrow::Cow::into_owned)
                .map_err(|error| ParseError::Xml(error.to_string()))
        })
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::{AirframeId, ConfigError, ParseError, Target, parse_airframe};

    #[test]
    fn rejects_empty_identifier() {
        assert_eq!(AirframeId::new("  "), Err(ConfigError::EmptyAirframeId));
    }

    #[test]
    fn parses_bebop_style_airframe_targets() {
        let xml = r#"
            <!DOCTYPE airframe SYSTEM "../airframe.dtd">
            <airframe name="bebop">
              <firmware name="rotorcraft">
                <target name="ap" board="bebop"/>
                <target name="nps" board="pc"/>
              </firmware>
            </airframe>
        "#;

        let airframe = parse_airframe(xml).expect("fixture is valid");
        assert_eq!(airframe.name.as_str(), "bebop");
        assert_eq!(airframe.firmware.name, "rotorcraft");
        assert_eq!(
            airframe.firmware.targets,
            vec![
                Target {
                    name: "ap".into(),
                    board: "bebop".into(),
                },
                Target {
                    name: "nps".into(),
                    board: "pc".into(),
                },
            ]
        );
    }

    #[test]
    fn rejects_target_without_board() {
        let xml = r#"<airframe name="test"><firmware name="fixedwing"><target name="ap"/></firmware></airframe>"#;
        assert_eq!(
            parse_airframe(xml),
            Err(ParseError::MissingTargetAttribute("board"))
        );
    }
}
