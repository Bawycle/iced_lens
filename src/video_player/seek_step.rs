// SPDX-License-Identifier: MPL-2.0
//! Keyboard seek step domain type for video playback.
//!
//! This module provides a type-safe wrapper for the keyboard seek step
//! duration in seconds.

use crate::config::{
    DEFAULT_KEYBOARD_SEEK_STEP_SECS, MAX_KEYBOARD_SEEK_STEP_SECS, MIN_KEYBOARD_SEEK_STEP_SECS,
};

/// Keyboard seek step in seconds for video navigation.
///
/// This newtype enforces validity at the type level, ensuring the value
/// is always within the valid range (0.5–30.0 seconds).
///
/// # Example
///
/// ```
/// use iced_lens::video_player::KeyboardSeekStep;
///
/// let step = KeyboardSeekStep::new(5.0);
/// assert_eq!(step.value(), 5.0);
///
/// // Values outside range are clamped
/// let too_high = KeyboardSeekStep::new(100.0);
/// assert_eq!(too_high.value(), 30.0); // Clamped to max
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KeyboardSeekStep(f64);

impl KeyboardSeekStep {
    /// Creates a new keyboard seek step value, clamping to valid range.
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.clamp(MIN_KEYBOARD_SEEK_STEP_SECS, MAX_KEYBOARD_SEEK_STEP_SECS))
    }

    /// Returns the value as f64.
    #[must_use]
    pub fn value(self) -> f64 {
        self.0
    }

    /// Returns the step as a Duration.
    #[must_use]
    pub fn as_duration(self) -> std::time::Duration {
        std::time::Duration::from_secs_f64(self.0)
    }

    /// Returns true if this is the minimum value.
    #[must_use]
    pub fn is_min(self) -> bool {
        self.0 <= MIN_KEYBOARD_SEEK_STEP_SECS
    }

    /// Returns true if this is the maximum value.
    #[must_use]
    pub fn is_max(self) -> bool {
        self.0 >= MAX_KEYBOARD_SEEK_STEP_SECS
    }
}

impl Default for KeyboardSeekStep {
    fn default() -> Self {
        Self(DEFAULT_KEYBOARD_SEEK_STEP_SECS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_clamps_to_valid_range() {
        assert_eq!(
            KeyboardSeekStep::new(0.0).value(),
            MIN_KEYBOARD_SEEK_STEP_SECS
        );
        assert_eq!(
            KeyboardSeekStep::new(100.0).value(),
            MAX_KEYBOARD_SEEK_STEP_SECS
        );
    }

    #[test]
    fn new_accepts_valid_values() {
        assert_eq!(KeyboardSeekStep::new(0.5).value(), 0.5);
        assert_eq!(KeyboardSeekStep::new(5.0).value(), 5.0);
        assert_eq!(KeyboardSeekStep::new(30.0).value(), 30.0);
    }

    #[test]
    fn default_returns_expected_value() {
        assert_eq!(
            KeyboardSeekStep::default().value(),
            DEFAULT_KEYBOARD_SEEK_STEP_SECS
        );
    }

    #[test]
    fn is_min_detects_minimum() {
        assert!(KeyboardSeekStep::new(0.5).is_min());
        assert!(!KeyboardSeekStep::new(5.0).is_min());
    }

    #[test]
    fn is_max_detects_maximum() {
        assert!(KeyboardSeekStep::new(30.0).is_max());
        assert!(!KeyboardSeekStep::new(5.0).is_max());
    }

    #[test]
    fn as_duration_converts_correctly() {
        let step = KeyboardSeekStep::new(2.5);
        assert_eq!(step.as_duration(), std::time::Duration::from_secs_f64(2.5));
    }

    #[test]
    fn equality_works() {
        assert_eq!(KeyboardSeekStep::new(5.0), KeyboardSeekStep::new(5.0));
        assert_ne!(KeyboardSeekStep::new(5.0), KeyboardSeekStep::new(10.0));
    }
}
