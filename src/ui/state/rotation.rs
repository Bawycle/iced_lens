// SPDX-License-Identifier: MPL-2.0
//! Rotation angle domain type for temporary media rotation.
//!
//! This module re-exports the domain type and provides backward compatibility.

// Re-export domain type
pub use crate::domain::ui::newtypes::RotationAngle;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_normalizes_to_90_increments() {
        assert_eq!(RotationAngle::new(0).degrees(), 0);
        assert_eq!(RotationAngle::new(45).degrees(), 0);
        assert_eq!(RotationAngle::new(89).degrees(), 0);
        assert_eq!(RotationAngle::new(90).degrees(), 90);
        assert_eq!(RotationAngle::new(135).degrees(), 90);
        assert_eq!(RotationAngle::new(180).degrees(), 180);
        assert_eq!(RotationAngle::new(270).degrees(), 270);
    }

    #[test]
    fn new_wraps_at_360() {
        assert_eq!(RotationAngle::new(360).degrees(), 0);
        assert_eq!(RotationAngle::new(450).degrees(), 90);
        assert_eq!(RotationAngle::new(720).degrees(), 0);
    }

    #[test]
    fn rotate_clockwise_increments_by_90() {
        let angle = RotationAngle::ZERO;
        assert_eq!(angle.rotate_clockwise().degrees(), 90);
        assert_eq!(angle.rotate_clockwise().rotate_clockwise().degrees(), 180);
    }

    #[test]
    fn rotate_clockwise_wraps_at_360() {
        let angle = RotationAngle::new(270);
        assert_eq!(angle.rotate_clockwise().degrees(), 0);
    }

    #[test]
    fn rotate_counterclockwise_decrements_by_90() {
        let angle = RotationAngle::new(180);
        assert_eq!(angle.rotate_counterclockwise().degrees(), 90);
        assert_eq!(
            angle
                .rotate_counterclockwise()
                .rotate_counterclockwise()
                .degrees(),
            0
        );
    }

    #[test]
    fn rotate_counterclockwise_wraps_at_0() {
        let angle = RotationAngle::ZERO;
        assert_eq!(angle.rotate_counterclockwise().degrees(), 270);
    }

    #[test]
    fn is_rotated_detects_non_zero() {
        assert!(!RotationAngle::ZERO.is_rotated());
        assert!(RotationAngle::new(90).is_rotated());
        assert!(RotationAngle::new(180).is_rotated());
        assert!(RotationAngle::new(270).is_rotated());
    }

    #[test]
    fn swaps_dimensions_for_90_and_270() {
        assert!(!RotationAngle::new(0).swaps_dimensions());
        assert!(RotationAngle::new(90).swaps_dimensions());
        assert!(!RotationAngle::new(180).swaps_dimensions());
        assert!(RotationAngle::new(270).swaps_dimensions());
    }

    #[test]
    fn radians_conversion() {
        use std::f32::consts::PI;
        assert!((RotationAngle::new(0).radians() - 0.0).abs() < f32::EPSILON);
        assert!((RotationAngle::new(90).radians() - PI / 2.0).abs() < 0.001);
        assert!((RotationAngle::new(180).radians() - PI).abs() < 0.001);
        assert!((RotationAngle::new(270).radians() - 3.0 * PI / 2.0).abs() < 0.001);
    }

    #[test]
    fn default_is_zero() {
        assert_eq!(RotationAngle::default(), RotationAngle::ZERO);
        assert_eq!(RotationAngle::default().degrees(), 0);
    }

    #[test]
    fn full_clockwise_rotation_returns_to_zero() {
        let angle = RotationAngle::ZERO
            .rotate_clockwise()
            .rotate_clockwise()
            .rotate_clockwise()
            .rotate_clockwise();
        assert_eq!(angle.degrees(), 0);
    }

    #[test]
    fn full_counterclockwise_rotation_returns_to_zero() {
        let angle = RotationAngle::ZERO
            .rotate_counterclockwise()
            .rotate_counterclockwise()
            .rotate_counterclockwise()
            .rotate_counterclockwise();
        assert_eq!(angle.degrees(), 0);
    }
}
