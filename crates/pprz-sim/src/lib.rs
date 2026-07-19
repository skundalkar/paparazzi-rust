//! Simulation and replay support.
//!
//! This crate is deliberately restricted to offline simulation and telemetry
//! replay. It does not provide hardware or actuator interfaces.

use pprz_protocol::{Decoder, Frame};

/// The result of replaying an arbitrary PPRZ byte stream offline.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayReport {
    /// Successfully decoded transport frames, in receive order.
    pub frames: Vec<Frame>,
    /// Complete frames rejected because their length or checksums were invalid.
    pub rejected_frames: usize,
}

/// The count of frames for one aircraft/message identifier pair.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MessageCount {
    /// The aircraft identifier in the counted frames.
    pub aircraft_id: u8,
    /// The message identifier in the counted frames.
    pub message_id: u8,
    /// How many matching frames were accepted.
    pub count: usize,
}

impl ReplayReport {
    /// Groups accepted frames by aircraft and message identifier in stable order.
    #[must_use]
    pub fn message_counts(&self) -> Vec<MessageCount> {
        let mut counts = std::collections::BTreeMap::new();
        for frame in &self.frames {
            *counts
                .entry((frame.aircraft_id, frame.message_id))
                .or_insert(0_usize) += 1;
        }
        counts
            .into_iter()
            .map(|((aircraft_id, message_id), count)| MessageCount {
                aircraft_id,
                message_id,
                count,
            })
            .collect()
    }
}

/// Decodes a recorded PPRZ byte stream without opening any device or network.
#[must_use]
pub fn replay(bytes: impl IntoIterator<Item = u8>) -> ReplayReport {
    let mut decoder = Decoder::new();
    let mut frames = Vec::new();
    let mut rejected_frames = 0;
    for byte in bytes {
        match decoder.push(byte) {
            Ok(Some(frame)) => frames.push(frame),
            Ok(None) => {}
            Err(_) => rejected_frames += 1,
        }
    }
    ReplayReport {
        frames,
        rejected_frames,
    }
}

/// Counts frames in an offline replay stream.
#[must_use]
pub fn frame_count(frames: &[Frame]) -> usize {
    frames.len()
}

#[cfg(test)]
mod tests {
    use super::{MessageCount, frame_count, replay};
    use pprz_protocol::Frame;

    #[test]
    fn counts_replayed_frames() {
        assert_eq!(
            frame_count(&[Frame::new(1, 1, []), Frame::new(2, 2, [])]),
            2
        );
    }

    #[test]
    fn reports_valid_and_corrupt_recorded_frames() {
        let valid = Frame::new(1, 2, [3]).encode().expect("small frame encodes");
        let mut captured = valid.clone();
        captured.extend([0x99, 6, 4, 5, 0xaa, 0]);
        let report = replay(captured);
        assert_eq!(report.frames, vec![Frame::new(1, 2, [3])]);
        assert_eq!(report.rejected_frames, 1);
    }

    #[test]
    fn groups_replayed_frames_by_message() {
        let bytes = [
            Frame::new(2, 5, []).encode().expect("frame encodes"),
            Frame::new(1, 9, []).encode().expect("frame encodes"),
            Frame::new(2, 5, [3]).encode().expect("frame encodes"),
        ]
        .concat();
        let report = replay(bytes);
        assert_eq!(
            report.message_counts(),
            vec![
                MessageCount {
                    aircraft_id: 1,
                    message_id: 9,
                    count: 1,
                },
                MessageCount {
                    aircraft_id: 2,
                    message_id: 5,
                    count: 2,
                },
            ]
        );
    }
}
