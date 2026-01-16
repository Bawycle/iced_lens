// SPDX-License-Identifier: MPL-2.0
//! Editing newtypes.
//!
//! This module provides type-safe wrappers for editing values,
//! ensuring they are always within valid ranges.

// =============================================================================
// Resize Scale Bounds
// =============================================================================

/// Resize scale bounds (10% to 400%).
pub mod resize_bounds {
    /// Minimum resize scale percentage.
    pub const MIN: f32 = 10.0;
    /// Maximum resize scale percentage.
    pub const MAX: f32 = 400.0;
    /// Default resize scale percentage.
    pub const DEFAULT: f32 = 100.0;
}

// =============================================================================
// ResizeScale
// =============================================================================

/// Resize scale percentage, guaranteed to be within valid range (10%–400%).
///
/// This value object encapsulates the business rules for resize scaling:
/// - Valid range is defined by constants
/// - Values are automatically clamped to the valid range
/// - Provides conversion to dimensions based on original image size
///
/// # Example
///
/// ```ignore
/// let scale = ResizeScale::new(200.0); // 200% = 2x enlargement
/// let (new_width, new_height) = scale.apply_to_dimensions(800, 600);
/// assert_eq!((new_width, new_height), (1600, 1200));
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResizeScale(f32);

impl ResizeScale {
    /// Creates a new resize scale, clamping the value to the valid range.
    #[must_use]
    pub fn new(percent: f32) -> Self {
        Self(percent.clamp(resize_bounds::MIN, resize_bounds::MAX))
    }

    /// Returns the raw percentage value.
    #[must_use]
    pub fn value(self) -> f32 {
        self.0
    }

    /// Returns the scale as a multiplier (e.g., 100% → 1.0, 200% → 2.0).
    #[must_use]
    pub fn as_factor(self) -> f32 {
        self.0 / 100.0
    }

    /// Applies the scale to the given dimensions, returning the new dimensions.
    ///
    /// Both dimensions are guaranteed to be at least 1 pixel.
    #[must_use]
    pub fn apply_to_dimensions(self, width: u32, height: u32) -> (u32, u32) {
        let factor = f64::from(self.as_factor());
        // Use f64 for intermediate calculation to avoid precision loss with large dimensions
        let new_width = (f64::from(width) * factor).round().max(1.0);
        let new_height = (f64::from(height) * factor).round().max(1.0);
        // Saturate to u32::MAX for safety (though images this large are impractical)
        // The conditional guarantees value is <= u32::MAX, so cast is safe
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let new_width = if new_width > f64::from(u32::MAX) {
            u32::MAX
        } else {
            new_width as u32
        };
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let new_height = if new_height > f64::from(u32::MAX) {
            u32::MAX
        } else {
            new_height as u32
        };
        (new_width, new_height)
    }

    /// Returns whether the scale is at the minimum value.
    #[must_use]
    pub fn is_min(self) -> bool {
        self.0 <= resize_bounds::MIN
    }

    /// Returns whether the scale is at the maximum value.
    #[must_use]
    pub fn is_max(self) -> bool {
        self.0 >= resize_bounds::MAX
    }

    /// Returns whether the scale represents 100% (no resize).
    #[must_use]
    pub fn is_original(self) -> bool {
        (self.0 - resize_bounds::DEFAULT).abs() < f32::EPSILON
    }

    /// Returns whether this scale represents an enlargement (> 100%).
    #[must_use]
    pub fn is_enlargement(self) -> bool {
        self.0 > resize_bounds::DEFAULT
    }

    /// Returns whether this scale represents a reduction (< 100%).
    #[must_use]
    pub fn is_reduction(self) -> bool {
        self.0 < resize_bounds::DEFAULT
    }
}

impl Default for ResizeScale {
    fn default() -> Self {
        Self(resize_bounds::DEFAULT)
    }
}

// =============================================================================
// Adjustment Bounds
// =============================================================================

/// Adjustment bounds (-100 to +100).
pub mod adjustment_bounds {
    /// Minimum adjustment value.
    pub const MIN: i32 = -100;
    /// Maximum adjustment value.
    pub const MAX: i32 = 100;
    /// Default (neutral) adjustment value.
    pub const DEFAULT: i32 = 0;
}

// =============================================================================
// AdjustmentPercent
// =============================================================================

/// Adjustment percentage for brightness/contrast, guaranteed to be within valid range (-100 to +100).
///
/// This type ensures that adjustment values are always valid, eliminating
/// the need for manual clamping at usage sites. A value of 0 means no adjustment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AdjustmentPercent(i32);

impl AdjustmentPercent {
    /// Creates a new adjustment value, clamping to the valid range.
    #[must_use]
    pub fn new(value: i32) -> Self {
        Self(value.clamp(adjustment_bounds::MIN, adjustment_bounds::MAX))
    }

    /// Returns the raw value.
    #[must_use]
    pub fn value(self) -> i32 {
        self.0
    }

    /// Returns whether this represents no adjustment (value is 0).
    #[must_use]
    pub fn is_neutral(self) -> bool {
        self.0 == adjustment_bounds::DEFAULT
    }

    /// Returns whether the adjustment is at the minimum value.
    #[must_use]
    pub fn is_min(self) -> bool {
        self.0 <= adjustment_bounds::MIN
    }

    /// Returns whether the adjustment is at the maximum value.
    #[must_use]
    pub fn is_max(self) -> bool {
        self.0 >= adjustment_bounds::MAX
    }
}

// =============================================================================
// Skip Attempts Bounds
// =============================================================================

/// Skip attempts bounds (1 to 20).
pub mod skip_bounds {
    /// Minimum skip attempts.
    pub const MIN: u32 = 1;
    /// Maximum skip attempts.
    pub const MAX: u32 = 20;
    /// Default skip attempts.
    pub const DEFAULT: u32 = 5;
}

// =============================================================================
// MaxSkipAttempts
// =============================================================================

/// Maximum number of consecutive corrupted files to skip during navigation.
///
/// This newtype enforces validity at the type level, ensuring the value
/// is always within the valid range (1–20).
///
/// # Example
///
/// ```ignore
/// let attempts = MaxSkipAttempts::new(5);
/// assert_eq!(attempts.value(), 5);
///
/// // Values outside range are clamped
/// let too_high = MaxSkipAttempts::new(100);
/// assert_eq!(too_high.value(), 20); // Clamped to max
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaxSkipAttempts(u32);

impl MaxSkipAttempts {
    /// Creates a new max skip attempts value, clamping to valid range.
    #[must_use]
    pub fn new(value: u32) -> Self {
        Self(value.clamp(skip_bounds::MIN, skip_bounds::MAX))
    }

    /// Returns the value as u32.
    #[must_use]
    pub fn value(self) -> u32 {
        self.0
    }

    /// Returns true if this is the minimum value.
    #[must_use]
    pub fn is_min(self) -> bool {
        self.0 <= skip_bounds::MIN
    }

    /// Returns true if this is the maximum value.
    #[must_use]
    pub fn is_max(self) -> bool {
        self.0 >= skip_bounds::MAX
    }
}

impl Default for MaxSkipAttempts {
    fn default() -> Self {
        Self(skip_bounds::DEFAULT)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // ResizeScale tests
    // -------------------------------------------------------------------------

    #[test]
    fn resize_scale_clamps() {
        assert!((ResizeScale::new(5.0).value() - resize_bounds::MIN).abs() < f32::EPSILON);
        assert!((ResizeScale::new(1000.0).value() - resize_bounds::MAX).abs() < f32::EPSILON);
        assert!((ResizeScale::new(150.0).value() - 150.0).abs() < f32::EPSILON);
    }

    #[test]
    fn resize_scale_default() {
        assert!((ResizeScale::default().value() - resize_bounds::DEFAULT).abs() < f32::EPSILON);
    }

    #[test]
    fn resize_scale_as_factor() {
        assert!((ResizeScale::new(100.0).as_factor() - 1.0).abs() < f32::EPSILON);
        assert!((ResizeScale::new(200.0).as_factor() - 2.0).abs() < f32::EPSILON);
        assert!((ResizeScale::new(50.0).as_factor() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn resize_scale_min_max() {
        assert!(ResizeScale::new(resize_bounds::MIN).is_min());
        assert!(ResizeScale::new(resize_bounds::MAX).is_max());
        assert!(!ResizeScale::new(100.0).is_min());
        assert!(!ResizeScale::new(100.0).is_max());
    }

    #[test]
    fn resize_scale_apply_dimensions() {
        let scale = ResizeScale::new(200.0); // 2x
        let (w, h) = scale.apply_to_dimensions(100, 50);
        assert_eq!((w, h), (200, 100));

        let scale = ResizeScale::new(50.0); // 0.5x
        let (w, h) = scale.apply_to_dimensions(100, 50);
        assert_eq!((w, h), (50, 25));
    }

    #[test]
    fn resize_scale_minimum_1px() {
        let scale = ResizeScale::new(10.0); // 0.1x
        let (w, h) = scale.apply_to_dimensions(5, 5);
        assert!(w >= 1);
        assert!(h >= 1);
    }

    #[test]
    fn resize_scale_enlargement_reduction() {
        assert!(ResizeScale::new(150.0).is_enlargement());
        assert!(!ResizeScale::new(150.0).is_reduction());

        assert!(ResizeScale::new(50.0).is_reduction());
        assert!(!ResizeScale::new(50.0).is_enlargement());

        assert!(!ResizeScale::new(100.0).is_enlargement());
        assert!(!ResizeScale::new(100.0).is_reduction());
        assert!(ResizeScale::new(100.0).is_original());
    }

    // -------------------------------------------------------------------------
    // AdjustmentPercent tests
    // -------------------------------------------------------------------------

    #[test]
    fn adjustment_percent_clamps() {
        assert_eq!(AdjustmentPercent::new(150).value(), adjustment_bounds::MAX);
        assert_eq!(AdjustmentPercent::new(-150).value(), adjustment_bounds::MIN);
        assert_eq!(AdjustmentPercent::new(50).value(), 50);
    }

    #[test]
    fn adjustment_percent_default() {
        assert_eq!(
            AdjustmentPercent::default().value(),
            adjustment_bounds::DEFAULT
        );
    }

    #[test]
    fn adjustment_percent_boundary_checks() {
        assert!(AdjustmentPercent::new(-100).is_min());
        assert!(AdjustmentPercent::new(100).is_max());
        assert!(AdjustmentPercent::new(0).is_neutral());
        assert!(!AdjustmentPercent::new(50).is_neutral());
    }

    // -------------------------------------------------------------------------
    // MaxSkipAttempts tests
    // -------------------------------------------------------------------------

    #[test]
    fn skip_attempts_clamps() {
        assert_eq!(MaxSkipAttempts::new(0).value(), skip_bounds::MIN);
        assert_eq!(MaxSkipAttempts::new(100).value(), skip_bounds::MAX);
    }

    #[test]
    fn skip_attempts_default() {
        assert_eq!(MaxSkipAttempts::default().value(), skip_bounds::DEFAULT);
    }

    #[test]
    fn skip_attempts_min_max() {
        assert!(MaxSkipAttempts::new(skip_bounds::MIN).is_min());
        assert!(MaxSkipAttempts::new(skip_bounds::MAX).is_max());
    }

    #[test]
    fn skip_attempts_valid_values() {
        assert_eq!(MaxSkipAttempts::new(1).value(), 1);
        assert_eq!(MaxSkipAttempts::new(10).value(), 10);
        assert_eq!(MaxSkipAttempts::new(20).value(), 20);
    }
}
