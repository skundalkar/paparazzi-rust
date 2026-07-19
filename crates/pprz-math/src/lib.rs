//! Deterministic math primitives used by simulation-safe components.

/// Converts degrees to radians.
#[must_use]
pub fn degrees_to_radians(degrees: f64) -> f64 {
    degrees.to_radians()
}

#[cfg(test)]
mod tests {
    use super::degrees_to_radians;

    #[test]
    fn converts_right_angle() {
        assert!((degrees_to_radians(90.0) - std::f64::consts::FRAC_PI_2).abs() < 1e-12);
    }
}
