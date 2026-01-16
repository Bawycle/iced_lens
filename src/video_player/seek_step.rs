// SPDX-License-Identifier: MPL-2.0
//! Keyboard seek step domain type for video playback.
//!
//! This module re-exports the domain type and provides backward compatibility.

// Re-export domain type
#[allow(unused_imports)] // Used by tests and may be used by external consumers
pub use crate::domain::video::newtypes::seek_step_bounds;
pub use crate::domain::video::newtypes::KeyboardSeekStep;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        DEFAULT_KEYBOARD_SEEK_STEP_SECS, MAX_KEYBOARD_SEEK_STEP_SECS, MIN_KEYBOARD_SEEK_STEP_SECS,
    };
    use crate::test_utils::assert_abs_diff_eq;

    // Verify domain bounds match config constants
    #[test]
    fn domain_bounds_match_config() {
        assert_abs_diff_eq!(seek_step_bounds::MIN, MIN_KEYBOARD_SEEK_STEP_SECS);
        assert_abs_diff_eq!(seek_step_bounds::MAX, MAX_KEYBOARD_SEEK_STEP_SECS);
        assert_abs_diff_eq!(seek_step_bounds::DEFAULT, DEFAULT_KEYBOARD_SEEK_STEP_SECS);
    }

    #[test]
    fn new_clamps_to_valid_range() {
        assert_abs_diff_eq!(
            KeyboardSeekStep::new(0.0).value(),
            MIN_KEYBOARD_SEEK_STEP_SECS
        );
        assert_abs_diff_eq!(
            KeyboardSeekStep::new(100.0).value(),
            MAX_KEYBOARD_SEEK_STEP_SECS
        );
    }

    #[test]
    fn new_accepts_valid_values() {
        assert_abs_diff_eq!(KeyboardSeekStep::new(0.5).value(), 0.5);
        assert_abs_diff_eq!(KeyboardSeekStep::new(5.0).value(), 5.0);
        assert_abs_diff_eq!(KeyboardSeekStep::new(30.0).value(), 30.0);
    }

    #[test]
    fn default_returns_expected_value() {
        assert_abs_diff_eq!(
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
