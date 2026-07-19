//! Paparazzi-compatible protocol primitives.
//!
//! The first milestone intentionally exposes only a small, fully tested frame
//! representation. Wire-level compatibility is added only with accompanying
//! upstream captures and golden tests.

/// A raw protocol frame, before message-specific decoding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Frame {
    /// The message identifier assigned by the selected protocol dictionary.
    pub message_id: u8,
    /// The message payload exactly as received.
    pub payload: Vec<u8>,
}

impl Frame {
    /// Constructs a frame from an identifier and payload.
    #[must_use]
    pub fn new(message_id: u8, payload: impl Into<Vec<u8>>) -> Self {
        Self {
            message_id,
            payload: payload.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Frame;

    #[test]
    fn retains_message_data() {
        let frame = Frame::new(7, [1, 2, 3]);
        assert_eq!(frame.message_id, 7);
        assert_eq!(frame.payload, [1, 2, 3]);
    }
}
