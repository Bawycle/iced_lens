// SPDX-License-Identifier: MPL-2.0
//! UI newtypes.
//!
//! This module provides type-safe wrappers for UI values,
//! ensuring they are always within valid ranges.

use std::time::Duration;

// =============================================================================
// Zoom Bounds
// =============================================================================

/// Zoom percentage bounds (10% to 800%).
pub mod zoom_bounds {
    /// Minimum zoom percentage.
    pub const MIN_PERCENT: f32 = 10.0;
    /// Maximum zoom percentage.
    pub const MAX_PERCENT: f32 = 800.0;
    /// Default zoom percentage.
    pub const DEFAULT_PERCENT: f32 = 100.0;
    /// Minimum zoom step percentage.
    pub const MIN_STEP: f32 = 1.0;
    /// Maximum zoom step percentage.
    pub const MAX_STEP: f32 = 200.0;
    /// Default zoom step percentage.
    pub const DEFAULT_STEP: f32 = 10.0;
}

// =============================================================================
// ZoomPercent
// =============================================================================

/// Zoom percentage, guaranteed to be within valid range (10%–800%).
///
/// This type ensures that zoom values are always valid, eliminating
/// the need for manual clamping at usage sites.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZoomPercent(f32);

impl ZoomPercent {
    /// Creates a new zoom percentage, clamping the value to the valid range.
    #[must_use]
    pub fn new(percent: f32) -> Self {
        Self(percent.clamp(zoom_bounds::MIN_PERCENT, zoom_bounds::MAX_PERCENT))
    }

    /// Returns the raw percentage value.
    #[must_use]
    pub fn value(self) -> f32 {
        self.0
    }

    /// Returns the zoom as a multiplier (e.g., 100% → 1.0).
    #[must_use]
    pub fn as_factor(self) -> f32 {
        self.0 / 100.0
    }

    /// Returns whether the zoom is at the minimum value.
    #[must_use]
    pub fn is_min(self) -> bool {
        self.0 <= zoom_bounds::MIN_PERCENT
    }

    /// Returns whether the zoom is at the maximum value.
    #[must_use]
    pub fn is_max(self) -> bool {
        self.0 >= zoom_bounds::MAX_PERCENT
    }

    /// Increases zoom by the given step.
    #[must_use]
    pub fn zoom_in(self, step: f32) -> Self {
        Self::new(self.0 + step)
    }

    /// Decreases zoom by the given step.
    #[must_use]
    pub fn zoom_out(self, step: f32) -> Self {
        Self::new(self.0 - step)
    }
}

impl Default for ZoomPercent {
    fn default() -> Self {
        Self(zoom_bounds::DEFAULT_PERCENT)
    }
}

// =============================================================================
// ZoomStep
// =============================================================================

/// Zoom step percentage, guaranteed to be within valid range (1%–200%).
///
/// This type ensures that zoom step values are always valid, eliminating
/// the need for manual clamping at usage sites.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZoomStep(f32);

impl ZoomStep {
    /// Creates a new zoom step, clamping the value to the valid range.
    #[must_use]
    pub fn new(percent: f32) -> Self {
        Self(percent.clamp(zoom_bounds::MIN_STEP, zoom_bounds::MAX_STEP))
    }

    /// Returns the raw percentage value.
    #[must_use]
    pub fn value(self) -> f32 {
        self.0
    }

    /// Returns whether the step is at the minimum value.
    #[must_use]
    pub fn is_min(self) -> bool {
        self.0 <= zoom_bounds::MIN_STEP
    }

    /// Returns whether the step is at the maximum value.
    #[must_use]
    pub fn is_max(self) -> bool {
        self.0 >= zoom_bounds::MAX_STEP
    }
}

impl Default for ZoomStep {
    fn default() -> Self {
        Self(zoom_bounds::DEFAULT_STEP)
    }
}

// =============================================================================
// Overlay Timeout Bounds
// =============================================================================

/// Overlay timeout bounds (1 to 30 seconds).
pub mod overlay_bounds {
    /// Minimum overlay timeout in seconds.
    pub const MIN: u32 = 1;
    /// Maximum overlay timeout in seconds.
    pub const MAX: u32 = 30;
    /// Default overlay timeout in seconds.
    pub const DEFAULT: u32 = 3;
}

// =============================================================================
// OverlayTimeout
// =============================================================================

/// Overlay timeout in seconds for fullscreen mode.
///
/// This newtype enforces validity at the type level, ensuring the value
/// is always within the valid range (1–30 seconds).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OverlayTimeout(u32);

impl OverlayTimeout {
    /// Creates a new overlay timeout value, clamping to valid range.
    #[must_use]
    pub fn new(value: u32) -> Self {
        Self(value.clamp(overlay_bounds::MIN, overlay_bounds::MAX))
    }

    /// Returns the value as u32.
    #[must_use]
    pub fn value(self) -> u32 {
        self.0
    }

    /// Returns the timeout as a Duration.
    #[must_use]
    pub fn as_duration(self) -> Duration {
        Duration::from_secs(u64::from(self.0))
    }

    /// Returns true if this is the minimum value.
    #[must_use]
    pub fn is_min(self) -> bool {
        self.0 <= overlay_bounds::MIN
    }

    /// Returns true if this is the maximum value.
    #[must_use]
    pub fn is_max(self) -> bool {
        self.0 >= overlay_bounds::MAX
    }
}

impl Default for OverlayTimeout {
    fn default() -> Self {
        Self(overlay_bounds::DEFAULT)
    }
}

// =============================================================================
// RotationAngle
// =============================================================================

/// Rotation angle in 90° increments.
///
/// This newtype enforces validity at the type level, ensuring the value
/// is always one of: 0°, 90°, 180°, or 270°.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
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

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // ZoomPercent tests
    // -------------------------------------------------------------------------

    #[test]
    fn zoom_percent_clamps() {
        assert!((ZoomPercent::new(5.0).value() - zoom_bounds::MIN_PERCENT).abs() < f32::EPSILON);
        assert!((ZoomPercent::new(1000.0).value() - zoom_bounds::MAX_PERCENT).abs() < f32::EPSILON);
        assert!((ZoomPercent::new(150.0).value() - 150.0).abs() < f32::EPSILON);
    }

    #[test]
    fn zoom_percent_default() {
        assert!(
            (ZoomPercent::default().value() - zoom_bounds::DEFAULT_PERCENT).abs() < f32::EPSILON
        );
    }

    #[test]
    fn zoom_percent_as_factor() {
        assert!((ZoomPercent::new(100.0).as_factor() - 1.0).abs() < f32::EPSILON);
        assert!((ZoomPercent::new(200.0).as_factor() - 2.0).abs() < f32::EPSILON);
        assert!((ZoomPercent::new(50.0).as_factor() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn zoom_percent_min_max() {
        assert!(ZoomPercent::new(zoom_bounds::MIN_PERCENT).is_min());
        assert!(ZoomPercent::new(zoom_bounds::MAX_PERCENT).is_max());
        assert!(!ZoomPercent::new(100.0).is_min());
        assert!(!ZoomPercent::new(100.0).is_max());
    }

    #[test]
    fn zoom_percent_zoom_in_out() {
        let zoom = ZoomPercent::new(100.0);
        assert!((zoom.zoom_in(10.0).value() - 110.0).abs() < f32::EPSILON);
        assert!((zoom.zoom_out(10.0).value() - 90.0).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // ZoomStep tests
    // -------------------------------------------------------------------------

    #[test]
    fn zoom_step_clamps() {
        assert!((ZoomStep::new(0.0).value() - zoom_bounds::MIN_STEP).abs() < f32::EPSILON);
        assert!((ZoomStep::new(500.0).value() - zoom_bounds::MAX_STEP).abs() < f32::EPSILON);
    }

    #[test]
    fn zoom_step_default() {
        assert!((ZoomStep::default().value() - zoom_bounds::DEFAULT_STEP).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // OverlayTimeout tests
    // -------------------------------------------------------------------------

    #[test]
    fn overlay_timeout_clamps() {
        assert_eq!(OverlayTimeout::new(0).value(), overlay_bounds::MIN);
        assert_eq!(OverlayTimeout::new(100).value(), overlay_bounds::MAX);
    }

    #[test]
    fn overlay_timeout_default() {
        assert_eq!(OverlayTimeout::default().value(), overlay_bounds::DEFAULT);
    }

    #[test]
    fn overlay_timeout_as_duration() {
        let timeout = OverlayTimeout::new(5);
        assert_eq!(timeout.as_duration(), Duration::from_secs(5));
    }

    #[test]
    fn overlay_timeout_min_max() {
        assert!(OverlayTimeout::new(overlay_bounds::MIN).is_min());
        assert!(OverlayTimeout::new(overlay_bounds::MAX).is_max());
    }

    // -------------------------------------------------------------------------
    // RotationAngle tests
    // -------------------------------------------------------------------------

    #[test]
    fn rotation_normalizes() {
        assert_eq!(RotationAngle::new(0).degrees(), 0);
        assert_eq!(RotationAngle::new(45).degrees(), 0);
        assert_eq!(RotationAngle::new(90).degrees(), 90);
        assert_eq!(RotationAngle::new(360).degrees(), 0);
        assert_eq!(RotationAngle::new(450).degrees(), 90);
    }

    #[test]
    fn rotation_clockwise() {
        let angle = RotationAngle::ZERO;
        assert_eq!(angle.rotate_clockwise().degrees(), 90);
        assert_eq!(RotationAngle::new(270).rotate_clockwise().degrees(), 0);
    }

    #[test]
    fn rotation_counterclockwise() {
        let angle = RotationAngle::ZERO;
        assert_eq!(angle.rotate_counterclockwise().degrees(), 270);
        assert_eq!(
            RotationAngle::new(90).rotate_counterclockwise().degrees(),
            0
        );
    }

    #[test]
    fn rotation_is_rotated() {
        assert!(!RotationAngle::ZERO.is_rotated());
        assert!(RotationAngle::new(90).is_rotated());
    }

    #[test]
    fn rotation_swaps_dimensions() {
        assert!(!RotationAngle::new(0).swaps_dimensions());
        assert!(RotationAngle::new(90).swaps_dimensions());
        assert!(!RotationAngle::new(180).swaps_dimensions());
        assert!(RotationAngle::new(270).swaps_dimensions());
    }

    #[test]
    fn rotation_radians() {
        use std::f32::consts::PI;
        assert!((RotationAngle::new(0).radians()).abs() < f32::EPSILON);
        assert!((RotationAngle::new(90).radians() - PI / 2.0).abs() < 0.001);
        assert!((RotationAngle::new(180).radians() - PI).abs() < 0.001);
    }
}
