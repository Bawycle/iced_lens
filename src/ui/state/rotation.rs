// SPDX-License-Identifier: MPL-2.0
//! Rotation angle domain type for temporary media rotation.
//!
//! This module provides a type-safe wrapper for rotation angles,
//! ensuring only valid 90° increments (0°, 90°, 180°, 270°).

/// Rotation angle in 90° increments.
///
/// This newtype enforces validity at the type level, ensuring the value
/// is always one of: 0°, 90°, 180°, or 270°.
///
/// # Example
///
/// ```
/// use iced_lens::ui::state::RotationAngle;
///
/// let angle = RotationAngle::default();
/// assert_eq!(angle.degrees(), 0);
///
/// let rotated = angle.rotate_clockwise();
/// assert_eq!(rotated.degrees(), 90);
///
/// // Full rotation cycle
/// let full = rotated.rotate_clockwise().rotate_clockwise().rotate_clockwise();
/// assert_eq!(full.degrees(), 0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RotationAngle(u16);

impl RotationAngle {
    /// No rotation (0°).
    pub const ZERO: Self = Self(0);

    /// Creates a new rotation angle, normalizing to valid 90° increments.
    ///
    /// Any value is normalized to the nearest lower 90° increment,
    /// then wrapped to 0-270° range.
    #[must_use]
    pub fn new(degrees: u16) -> Self {
        // Round down to nearest 90° and wrap
        Self(((degrees / 90) * 90) % 360)
    }

    /// Returns the angle in degrees.
    #[must_use]
    pub fn degrees(self) -> u16 {
        self.0
    }

    /// Returns the angle in radians.
    #[must_use]
    pub fn radians(self) -> f32 {
        f32::from(self.0) * std::f32::consts::PI / 180.0
    }

    /// Rotates 90° clockwise.
    #[must_use]
    pub fn rotate_clockwise(self) -> Self {
        Self((self.0 + 90) % 360)
    }

    /// Rotates 90° counter-clockwise.
    #[must_use]
    pub fn rotate_counterclockwise(self) -> Self {
        Self((self.0 + 270) % 360)
    }

    /// Returns true if the angle is not zero (media is rotated).
    #[must_use]
    pub fn is_rotated(self) -> bool {
        self.0 != 0
    }

    /// Returns true if width and height should be swapped when rendering.
    ///
    /// This is true for 90° and 270° rotations.
    #[must_use]
    pub fn swaps_dimensions(self) -> bool {
        self.0 == 90 || self.0 == 270
    }
}

impl Default for RotationAngle {
    fn default() -> Self {
        Self::ZERO
    }
}

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
