// SPDX-License-Identifier: MPL-2.0
//! Playback speed domain type for video playback.
//!
//! This module re-exports the domain type and provides backward compatibility.

// Re-export domain type
#[allow(unused_imports)] // Used by tests and may be used by external consumers
pub use crate::domain::video::newtypes::speed_bounds;
pub use crate::domain::video::newtypes::PlaybackSpeed;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        MAX_PLAYBACK_SPEED, MIN_PLAYBACK_SPEED, PLAYBACK_SPEED_AUTO_MUTE_THRESHOLD,
    };
    use crate::test_utils::assert_abs_diff_eq;

    // Verify domain bounds match config constants
    #[test]
    fn domain_bounds_match_config() {
        assert_abs_diff_eq!(speed_bounds::MIN, MIN_PLAYBACK_SPEED);
        assert_abs_diff_eq!(speed_bounds::MAX, MAX_PLAYBACK_SPEED);
        assert_abs_diff_eq!(speed_bounds::AUTO_MUTE_THRESHOLD, PLAYBACK_SPEED_AUTO_MUTE_THRESHOLD);
    }

    #[test]
    fn new_clamps_to_valid_range() {
        assert_abs_diff_eq!(PlaybackSpeed::new(0.01).value(), MIN_PLAYBACK_SPEED);
        assert_abs_diff_eq!(PlaybackSpeed::new(100.0).value(), MAX_PLAYBACK_SPEED);
        assert_abs_diff_eq!(PlaybackSpeed::new(2.0).value(), 2.0);
    }

    #[test]
    fn default_is_normal_speed() {
        assert_abs_diff_eq!(PlaybackSpeed::default().value(), 1.0);
    }

    #[test]
    fn should_auto_mute_above_threshold() {
        assert!(!PlaybackSpeed::new(1.0).should_auto_mute());
        assert!(!PlaybackSpeed::new(2.0).should_auto_mute());
        assert!(PlaybackSpeed::new(2.5).should_auto_mute());
        assert!(PlaybackSpeed::new(4.0).should_auto_mute());
    }

    #[test]
    fn increase_cycles_through_presets() {
        let speed = PlaybackSpeed::new(1.0);
        let faster = speed.increase();
        assert!(faster.value() > 1.0);

        // At max, stays at max
        let max_speed = PlaybackSpeed::new(MAX_PLAYBACK_SPEED);
        assert_abs_diff_eq!(max_speed.increase().value(), MAX_PLAYBACK_SPEED);
    }

    #[test]
    fn decrease_cycles_through_presets() {
        let speed = PlaybackSpeed::new(1.0);
        let slower = speed.decrease();
        assert!(slower.value() < 1.0);

        // At min, stays at min
        let min_speed = PlaybackSpeed::new(MIN_PLAYBACK_SPEED);
        assert_abs_diff_eq!(min_speed.decrease().value(), MIN_PLAYBACK_SPEED);
    }

    #[test]
    fn is_min_and_is_max() {
        assert!(PlaybackSpeed::new(MIN_PLAYBACK_SPEED).is_min());
        assert!(!PlaybackSpeed::new(1.0).is_min());

        assert!(PlaybackSpeed::new(MAX_PLAYBACK_SPEED).is_max());
        assert!(!PlaybackSpeed::new(1.0).is_max());
    }
}
