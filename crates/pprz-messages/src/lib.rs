//! Paparazzi XML message dictionary parsing and offline payload decoding.
//!
//! This crate only interprets captured PPRZ payload bytes. It does not open a
//! network connection, serial device, or actuator interface.

use quick_xml::{Reader, events::Event};

use pprz_protocol::Frame;

/// A collection of message definitions for one Paparazzi protocol class.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageDictionary {
    messages: Vec<MessageDefinition>,
}

impl MessageDictionary {
    /// Finds a definition by the PPRZ transport message identifier.
    #[must_use]
    pub fn message(&self, id: u8) -> Option<&MessageDefinition> {
        self.messages.iter().find(|message| message.id == id)
    }

    /// Returns every parsed message definition in XML order.
    #[must_use]
    pub fn messages(&self) -> &[MessageDefinition] {
        &self.messages
    }

    /// Decodes one frame using this dictionary.
    ///
    /// Numeric values use Paparazzi's little-endian wire representation.
    /// Variable-length array fields consume the remaining payload and must be
    /// the final field in a message definition.
    ///
    /// # Errors
    ///
    /// Returns [`DecodeError`] when no message definition matches the frame or
    /// its payload does not satisfy the selected field layout.
    pub fn decode(&self, frame: &Frame) -> Result<DecodedMessage, DecodeError> {
        let definition = self
            .message(frame.message_id)
            .ok_or(DecodeError::UnknownMessage(frame.message_id))?;
        let mut payload = frame.payload.as_slice();
        let mut fields = Vec::with_capacity(definition.fields.len());
        for (index, field) in definition.fields.iter().enumerate() {
            let remaining_fields = definition.fields.len() - index - 1;
            let value = decode_value(field.kind, &mut payload, remaining_fields == 0)?;
            fields.push(DecodedField {
                name: field.name.clone(),
                value,
            });
        }
        if !payload.is_empty() {
            return Err(DecodeError::TrailingBytes(payload.len()));
        }
        Ok(DecodedMessage {
            aircraft_id: frame.aircraft_id,
            id: definition.id,
            name: definition.name.clone(),
            fields,
        })
    }
}

/// A named message in a Paparazzi protocol class.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageDefinition {
    /// Transport message identifier.
    pub id: u8,
    /// Dictionary message name.
    pub name: String,
    /// Fields encoded in payload order.
    pub fields: Vec<FieldDefinition>,
}

/// A named field in a message definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldDefinition {
    /// Dictionary field name.
    pub name: String,
    /// Primitive scalar or array wire representation.
    pub kind: FieldKind,
}

/// Supported Paparazzi primitive field representations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FieldKind {
    /// An unsigned 8-bit integer.
    U8,
    /// A signed 8-bit integer.
    I8,
    /// An unsigned 16-bit little-endian integer.
    U16,
    /// A signed 16-bit little-endian integer.
    I16,
    /// An unsigned 32-bit little-endian integer.
    U32,
    /// A signed 32-bit little-endian integer.
    I32,
    /// An IEEE-754 32-bit little-endian float.
    F32,
    /// A variable-length byte sequence.
    U8Array,
    /// A variable-length signed 16-bit integer sequence.
    I16Array,
    /// A variable-length unsigned 16-bit integer sequence.
    U16Array,
}

/// A message decoded from a PPRZ transport frame.
#[derive(Clone, Debug, PartialEq)]
pub struct DecodedMessage {
    /// Sender or destination aircraft identifier.
    pub aircraft_id: u8,
    /// Decoded message identifier.
    pub id: u8,
    /// Dictionary message name.
    pub name: String,
    /// Decoded payload fields in definition order.
    pub fields: Vec<DecodedField>,
}

/// One named decoded payload value.
#[derive(Clone, Debug, PartialEq)]
pub struct DecodedField {
    /// Dictionary field name.
    pub name: String,
    /// Decoded payload value.
    pub value: FieldValue,
}

/// A decoded primitive scalar or variable-length array.
#[derive(Clone, Debug, PartialEq)]
pub enum FieldValue {
    /// Unsigned 8-bit value.
    U8(u8),
    /// Signed 8-bit value.
    I8(i8),
    /// Unsigned 16-bit value.
    U16(u16),
    /// Signed 16-bit value.
    I16(i16),
    /// Unsigned 32-bit value.
    U32(u32),
    /// Signed 32-bit value.
    I32(i32),
    /// 32-bit floating-point value.
    F32(f32),
    /// Variable-length byte sequence.
    U8Array(Vec<u8>),
    /// Variable-length signed 16-bit sequence.
    I16Array(Vec<i16>),
    /// Variable-length unsigned 16-bit sequence.
    U16Array(Vec<u16>),
}

/// Errors raised while parsing a message dictionary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DictionaryError {
    /// XML was malformed or unreadable.
    Xml(String),
    /// The requested class was absent.
    MissingClass(String),
    /// A message did not have a valid `id` or `ID` attribute.
    InvalidMessageId(String),
    /// A message or field was missing a required attribute.
    MissingAttribute(&'static str),
    /// A field representation is intentionally unsupported.
    UnsupportedFieldType(String),
    /// A variable array was followed by another field.
    VariableArrayNotLast(String),
}

/// Errors raised while decoding a frame with a known dictionary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DecodeError {
    /// The dictionary contains no definition for this message ID.
    UnknownMessage(u8),
    /// The payload ended before a scalar could be decoded.
    PayloadTooShort {
        /// Width of the scalar that could not be read.
        needed: usize,
        /// Bytes remaining in the payload.
        available: usize,
    },
    /// An array payload was not an exact multiple of its element width.
    MisalignedArray {
        /// Width in bytes of one array element.
        element_width: usize,
        /// Bytes available for the variable-length array.
        available: usize,
    },
    /// Decoding known fields left bytes unconsumed.
    TrailingBytes(usize),
}

/// Parses one named `<class>` from a Paparazzi message XML document.
///
/// # Errors
///
/// Returns [`DictionaryError`] if the XML is malformed, the requested class is
/// absent, or its message definitions use unsupported or invalid layouts.
pub fn parse_dictionary(xml: &str, class_name: &str) -> Result<MessageDictionary, DictionaryError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buffer = Vec::new();
    let mut in_class = false;
    let mut found_class = false;
    let mut current: Option<MessageDefinition> = None;
    let mut messages = Vec::new();

    loop {
        match reader
            .read_event_into(&mut buffer)
            .map_err(|error| DictionaryError::Xml(error.to_string()))?
        {
            Event::Start(element) if element.name().as_ref() == b"class" => {
                in_class = attribute(&element, b"name")?.as_deref() == Some(class_name);
                found_class |= in_class;
            }
            Event::Start(element) if in_class && element.name().as_ref() == b"message" => {
                current = Some(parse_message(&element)?);
            }
            Event::Empty(element) if in_class && element.name().as_ref() == b"message" => {
                messages.push(parse_message(&element)?);
            }
            Event::Empty(element) if in_class && element.name().as_ref() == b"field" => {
                if let Some(message) = current.as_mut() {
                    message.fields.push(parse_field(&element)?);
                }
            }
            Event::End(element) if in_class && element.name().as_ref() == b"message" => {
                if let Some(message) = current.take() {
                    validate_message(&message)?;
                    messages.push(message);
                }
            }
            Event::End(element) if element.name().as_ref() == b"class" => in_class = false,
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    if !found_class {
        return Err(DictionaryError::MissingClass(class_name.into()));
    }
    Ok(MessageDictionary { messages })
}

fn parse_message(
    element: &quick_xml::events::BytesStart<'_>,
) -> Result<MessageDefinition, DictionaryError> {
    let name = attribute(element, b"name")?.ok_or(DictionaryError::MissingAttribute("name"))?;
    let id = attribute(element, b"id")?
        .or(attribute(element, b"ID")?)
        .ok_or(DictionaryError::MissingAttribute("id"))?;
    let id = parse_id(&id)?;
    Ok(MessageDefinition {
        id,
        name,
        fields: Vec::new(),
    })
}

fn parse_field(
    element: &quick_xml::events::BytesStart<'_>,
) -> Result<FieldDefinition, DictionaryError> {
    let name = attribute(element, b"name")?.ok_or(DictionaryError::MissingAttribute("name"))?;
    let field_type =
        attribute(element, b"type")?.ok_or(DictionaryError::MissingAttribute("type"))?;
    let kind = match field_type.as_str() {
        "uint8" => FieldKind::U8,
        "int8" => FieldKind::I8,
        "uint16" => FieldKind::U16,
        "int16" => FieldKind::I16,
        "uint32" => FieldKind::U32,
        "int32" => FieldKind::I32,
        "float" => FieldKind::F32,
        "uint8[]" => FieldKind::U8Array,
        "int16[]" => FieldKind::I16Array,
        "uint16[]" => FieldKind::U16Array,
        _ => return Err(DictionaryError::UnsupportedFieldType(field_type)),
    };
    Ok(FieldDefinition { name, kind })
}

fn validate_message(message: &MessageDefinition) -> Result<(), DictionaryError> {
    for field in message
        .fields
        .iter()
        .take(message.fields.len().saturating_sub(1))
    {
        if matches!(
            field.kind,
            FieldKind::U8Array | FieldKind::I16Array | FieldKind::U16Array
        ) {
            return Err(DictionaryError::VariableArrayNotLast(message.name.clone()));
        }
    }
    Ok(())
}

fn parse_id(text: &str) -> Result<u8, DictionaryError> {
    let parsed = text
        .strip_prefix("0x")
        .map_or_else(|| text.parse::<u8>(), |hex| u8::from_str_radix(hex, 16));
    parsed.map_err(|_| DictionaryError::InvalidMessageId(text.into()))
}

fn attribute(
    element: &quick_xml::events::BytesStart<'_>,
    name: &[u8],
) -> Result<Option<String>, DictionaryError> {
    element
        .try_get_attribute(name)
        .map_err(|error| DictionaryError::Xml(error.to_string()))?
        .map(|value| {
            value
                .unescape_value()
                .map(std::borrow::Cow::into_owned)
                .map_err(|error| DictionaryError::Xml(error.to_string()))
        })
        .transpose()
}

fn decode_value(
    kind: FieldKind,
    bytes: &mut &[u8],
    is_last: bool,
) -> Result<FieldValue, DecodeError> {
    match kind {
        FieldKind::U8 => Ok(FieldValue::U8(take(bytes, 1)?[0])),
        FieldKind::I8 => Ok(FieldValue::I8(i8::from_le_bytes([take(bytes, 1)?[0]]))),
        FieldKind::U16 => Ok(FieldValue::U16(u16::from_le_bytes(
            take(bytes, 2)?.try_into().expect("width checked"),
        ))),
        FieldKind::I16 => Ok(FieldValue::I16(i16::from_le_bytes(
            take(bytes, 2)?.try_into().expect("width checked"),
        ))),
        FieldKind::U32 => Ok(FieldValue::U32(u32::from_le_bytes(
            take(bytes, 4)?.try_into().expect("width checked"),
        ))),
        FieldKind::I32 => Ok(FieldValue::I32(i32::from_le_bytes(
            take(bytes, 4)?.try_into().expect("width checked"),
        ))),
        FieldKind::F32 => Ok(FieldValue::F32(f32::from_le_bytes(
            take(bytes, 4)?.try_into().expect("width checked"),
        ))),
        FieldKind::U8Array => Ok(FieldValue::U8Array(take_array(bytes, 1, is_last)?.to_vec())),
        FieldKind::I16Array => Ok(FieldValue::I16Array(
            take_array(bytes, 2, is_last)?
                .chunks_exact(2)
                .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
                .collect(),
        )),
        FieldKind::U16Array => Ok(FieldValue::U16Array(
            take_array(bytes, 2, is_last)?
                .chunks_exact(2)
                .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
                .collect(),
        )),
    }
}

fn take<'a>(bytes: &mut &'a [u8], count: usize) -> Result<&'a [u8], DecodeError> {
    if bytes.len() < count {
        return Err(DecodeError::PayloadTooShort {
            needed: count,
            available: bytes.len(),
        });
    }
    let (value, rest) = bytes.split_at(count);
    *bytes = rest;
    Ok(value)
}

fn take_array<'a>(
    bytes: &mut &'a [u8],
    width: usize,
    is_last: bool,
) -> Result<&'a [u8], DecodeError> {
    if !is_last {
        return Err(DecodeError::TrailingBytes(bytes.len()));
    }
    if bytes.len() % width != 0 {
        return Err(DecodeError::MisalignedArray {
            element_width: width,
            available: bytes.len(),
        });
    }
    let value = *bytes;
    *bytes = &[];
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::{DecodeError, FieldValue, parse_dictionary};
    use pprz_protocol::Frame;

    const DICTIONARY: &str = r#"
      <protocol><class name="telemetry">
        <message name="ATTITUDE" ID="6"><field name="phi" type="int16"/><field name="psi" type="int16"/><field name="theta" type="int16"/></message>
        <message name="COMMANDS" id="0x66"><field name="values" type="int16[]"/></message>
      </class></protocol>
    "#;

    #[test]
    fn parses_and_decodes_scalars() {
        let dictionary = parse_dictionary(DICTIONARY, "telemetry").expect("dictionary parses");
        let message = dictionary
            .decode(&Frame::new(61, 6, [0x34, 0x12, 0xfe, 0xff, 0, 0x80]))
            .expect("payload decodes");
        assert_eq!(message.name, "ATTITUDE");
        assert_eq!(message.fields[0].value, FieldValue::I16(0x1234));
        assert_eq!(message.fields[1].value, FieldValue::I16(-2));
        assert_eq!(message.fields[2].value, FieldValue::I16(i16::MIN));
    }

    #[test]
    fn decodes_variable_array_at_end() {
        let dictionary = parse_dictionary(DICTIONARY, "telemetry").expect("dictionary parses");
        let message = dictionary
            .decode(&Frame::new(61, 102, [1, 0, 0xfe, 0xff]))
            .expect("payload decodes");
        assert_eq!(message.fields[0].value, FieldValue::I16Array(vec![1, -2]));
    }

    #[test]
    fn rejects_unknown_message() {
        let dictionary = parse_dictionary(DICTIONARY, "telemetry").expect("dictionary parses");
        assert_eq!(
            dictionary.decode(&Frame::new(61, 7, [])),
            Err(DecodeError::UnknownMessage(7))
        );
    }
}
