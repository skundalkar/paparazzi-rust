//! Simulation and replay support.
//!
//! This crate is deliberately restricted to offline simulation and telemetry
//! replay. It does not provide hardware or actuator interfaces.

use pprz_protocol::Frame;

/// Counts frames in an offline replay stream.
#[must_use]
pub fn frame_count(frames: &[Frame]) -> usize {
    frames.len()
}

#[cfg(test)]
mod tests {
    use super::frame_count;
    use pprz_protocol::Frame;

    #[test]
    fn counts_replayed_frames() {
        assert_eq!(
            frame_count(&[Frame::new(1, 1, []), Frame::new(2, 2, [])]),
            2
        );
    }
}
