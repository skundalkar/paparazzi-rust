//! PPRZ transport v1 primitives.
//!
//! This crate implements the byte-oriented framing documented by Paparazzi's
//! `pprz_transport.py` reference implementation. It deliberately does not
//! interpret message payloads or communicate with hardware.

/// The PPRZ v1 start-of-frame byte.
pub const STX: u8 = 0x99;

/// Bytes in a frame that are not payload: STX, length, aircraft ID, message ID,
/// and two checksums.
pub const NON_PAYLOAD_BYTES: usize = 6;

/// A decoded PPRZ v1 frame.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Frame {
    /// The sender or destination aircraft identifier.
    pub aircraft_id: u8,
    /// The message identifier assigned by the selected protocol dictionary.
    pub message_id: u8,
    /// The message payload exactly as received.
    pub payload: Vec<u8>,
}

impl Frame {
    /// Constructs a frame from its aircraft ID, message ID, and payload.
    #[must_use]
    pub fn new(aircraft_id: u8, message_id: u8, payload: impl Into<Vec<u8>>) -> Self {
        Self {
            aircraft_id,
            message_id,
            payload: payload.into(),
        }
    }

    /// Serializes this frame using the PPRZ v1 transport wire format.
    ///
    /// The checksum is the two-byte Fletcher-style checksum used by Paparazzi:
    /// both accumulators start at zero and consume the length, aircraft ID,
    /// message ID, and payload, excluding `STX` and the checksum bytes.
    ///
    /// # Errors
    ///
    /// Returns [`EncodeError::PayloadTooLong`] when the payload cannot fit in
    /// the one-byte PPRZ v1 total-length field.
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        let length = self
            .payload
            .len()
            .checked_add(NON_PAYLOAD_BYTES)
            .ok_or(EncodeError::PayloadTooLong)?;
        let length = u8::try_from(length).map_err(|_| EncodeError::PayloadTooLong)?;

        let mut bytes = Vec::with_capacity(usize::from(length));
        bytes.extend([STX, length, self.aircraft_id, self.message_id]);
        bytes.extend_from_slice(&self.payload);
        let (checksum_a, checksum_b) = checksum(&bytes);
        bytes.extend([checksum_a, checksum_b]);
        Ok(bytes)
    }
}

/// An error raised when a frame cannot be encoded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncodeError {
    /// PPRZ v1 stores total frame length in one byte.
    PayloadTooLong,
}

/// An error raised while decoding an input stream.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecodeError {
    /// A frame length smaller than the transport overhead was received.
    InvalidLength(u8),
    /// The received checksums do not match the encoded contents.
    ChecksumMismatch,
}

/// A stateful PPRZ v1 stream decoder.
#[derive(Clone, Debug, Default)]
pub struct Decoder {
    bytes: Vec<u8>,
    expected_length: Option<usize>,
}

impl Decoder {
    /// Creates a decoder ready to accept arbitrary stream chunks.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Consumes one byte and returns a frame once a complete valid frame arrives.
    ///
    /// Bytes before `STX` are ignored. On malformed length or checksum the
    /// decoder resets, allowing the caller to continue processing the stream.
    ///
    /// # Errors
    ///
    /// Returns [`DecodeError::InvalidLength`] for a declared frame shorter than
    /// the transport overhead and [`DecodeError::ChecksumMismatch`] when a
    /// complete frame has invalid checksums.
    pub fn push(&mut self, byte: u8) -> Result<Option<Frame>, DecodeError> {
        if self.bytes.is_empty() {
            if byte == STX {
                self.bytes.push(byte);
            }
            return Ok(None);
        }

        self.bytes.push(byte);
        if self.bytes.len() == 2 {
            let length = usize::from(byte);
            if length < NON_PAYLOAD_BYTES {
                self.reset();
                return Err(DecodeError::InvalidLength(byte));
            }
            self.expected_length = Some(length);
            return Ok(None);
        }

        let Some(expected_length) = self.expected_length else {
            self.reset();
            return Err(DecodeError::InvalidLength(0));
        };
        if self.bytes.len() < expected_length {
            return Ok(None);
        }

        let completed = std::mem::take(&mut self.bytes);
        self.expected_length = None;
        let payload_end = expected_length - 2;
        let (checksum_a, checksum_b) = checksum(&completed[..payload_end]);
        if completed[payload_end] != checksum_a || completed[payload_end + 1] != checksum_b {
            return Err(DecodeError::ChecksumMismatch);
        }

        Ok(Some(Frame::new(
            completed[2],
            completed[3],
            completed[4..payload_end].to_vec(),
        )))
    }

    /// Clears any partial frame while preserving the decoder allocation.
    pub fn reset(&mut self) {
        self.bytes.clear();
        self.expected_length = None;
    }
}

fn checksum(bytes: &[u8]) -> (u8, u8) {
    let mut checksum_a = 0_u8;
    let mut checksum_b = 0_u8;
    for byte in bytes.iter().skip(1) {
        checksum_a = checksum_a.wrapping_add(*byte);
        checksum_b = checksum_b.wrapping_add(checksum_a);
    }
    (checksum_a, checksum_b)
}

#[cfg(test)]
mod tests {
    use super::{DecodeError, Decoder, EncodeError, Frame, NON_PAYLOAD_BYTES, STX};

    #[test]
    fn encodes_reference_frame() {
        let frame = Frame::new(42, 7, [1, 2]);
        assert_eq!(frame.encode(), Ok(vec![STX, 8, 42, 7, 1, 2, 0x3c, 0xe9]));
    }

    #[test]
    fn decodes_reference_frame_one_byte_at_a_time() {
        let mut decoder = Decoder::new();
        let bytes = [STX, 8, 42, 7, 1, 2, 0x3c, 0xe9];
        let mut output = None;
        for byte in bytes {
            output = decoder.push(byte).expect("reference frame is valid");
        }
        assert_eq!(output, Some(Frame::new(42, 7, [1, 2])));
    }

    #[test]
    fn decodes_first_frame_from_upstream_pprz_recording() {
        let bytes = [STX, 9, 61, 103, 2, 1, 35, 0xd3, 0x2e];
        let mut decoder = Decoder::new();
        let frame = bytes
            .into_iter()
            .find_map(|byte| decoder.push(byte).expect("captured frame is valid"));
        assert_eq!(frame, Some(Frame::new(61, 103, [2, 1, 35])));
    }

    #[test]
    fn round_trips_every_legal_payload_length() {
        for payload_length in 0..=(u8::MAX as usize - NON_PAYLOAD_BYTES) {
            let payload = (0..payload_length)
                .map(|index| u8::try_from(index).expect("length is bounded"))
                .collect::<Vec<_>>();
            let expected = Frame::new(17, 42, payload);
            let mut decoder = Decoder::new();
            let decoded_frame = expected
                .encode()
                .expect("legal payload encodes")
                .into_iter()
                .find_map(|byte| decoder.push(byte).expect("encoded frame is valid"));
            assert_eq!(
                decoded_frame,
                Some(expected),
                "payload length {payload_length}"
            );
        }
    }

    #[test]
    fn ignores_noise_before_a_frame() {
        let mut decoder = Decoder::new();
        for byte in [0, 1, 2] {
            assert_eq!(decoder.push(byte), Ok(None));
        }
        let mut output = None;
        for byte in [STX, 6, 3, 9, 0x12, 0x21] {
            output = decoder.push(byte).expect("frame is valid");
        }
        assert_eq!(output, Some(Frame::new(3, 9, [])));
    }

    #[test]
    fn rejects_corrupt_checksum_and_recovers() {
        let mut decoder = Decoder::new();
        let corrupt = [STX, 6, 3, 9, 0x12, 0];
        for byte in corrupt.into_iter().take(NON_PAYLOAD_BYTES - 1) {
            assert_eq!(decoder.push(byte), Ok(None));
        }
        assert_eq!(decoder.push(0), Err(DecodeError::ChecksumMismatch));
        let recovered = Frame::new(1, 2, []).encode().expect("small frame encodes");
        let mut output = None;
        for byte in recovered {
            output = decoder.push(byte).expect("decoder recovers after error");
        }
        assert_eq!(output, Some(Frame::new(1, 2, [])));
    }

    #[test]
    fn rejects_invalid_length() {
        let mut decoder = Decoder::new();
        assert_eq!(decoder.push(STX), Ok(None));
        assert_eq!(decoder.push(5), Err(DecodeError::InvalidLength(5)));
    }

    #[test]
    fn refuses_payloads_that_do_not_fit_the_wire_length() {
        let payload = vec![0; usize::from(u8::MAX) - NON_PAYLOAD_BYTES + 1];
        assert_eq!(
            Frame::new(1, 2, payload).encode(),
            Err(EncodeError::PayloadTooLong)
        );
    }
}
