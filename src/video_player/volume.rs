// SPDX-License-Identifier: MPL-2.0
//! Volume domain type for audio playback.
//!
//! This module re-exports the domain type and provides backward compatibility.

// Re-export domain type
#[allow(unused_imports)] // Used by tests and may be used by external consumers
pub use crate::domain::video::newtypes::volume_bounds;
pub use crate::domain::video::newtypes::Volume;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DEFAULT_VOLUME, MAX_VOLUME, MIN_VOLUME, VOLUME_STEP};
    use crate::test_utils::assert_abs_diff_eq;

    // Verify domain bounds match config constants
    #[test]
    fn domain_bounds_match_config() {
        assert_abs_diff_eq!(volume_bounds::MIN, MIN_VOLUME);
        assert_abs_diff_eq!(volume_bounds::MAX, MAX_VOLUME);
        assert_abs_diff_eq!(volume_bounds::DEFAULT, DEFAULT_VOLUME);
        assert_abs_diff_eq!(volume_bounds::STEP, VOLUME_STEP);
    }

    #[test]
    fn new_clamps_to_valid_range() {
        assert_abs_diff_eq!(Volume::new(-0.5).value(), MIN_VOLUME);
        assert_abs_diff_eq!(Volume::new(1.5).value(), MAX_VOLUME);
        assert_abs_diff_eq!(Volume::new(0.5).value(), 0.5);
    }

    #[test]
    fn default_is_expected_volume() {
        assert_abs_diff_eq!(Volume::default().value(), DEFAULT_VOLUME);
    }

    #[test]
    fn is_muted_detects_zero_volume() {
        assert!(Volume::new(0.0).is_muted());
        assert!(Volume::new(0.0005).is_muted());
        assert!(!Volume::new(0.01).is_muted());
        assert!(!Volume::new(0.5).is_muted());
    }

    #[test]
    fn increase_adds_step() {
        let vol = Volume::new(0.5);
        let louder = vol.increase();
        assert_abs_diff_eq!(louder.value(), 0.5 + VOLUME_STEP, epsilon = 0.001);

        // At max, stays at max
        let max_vol = Volume::new(MAX_VOLUME);
        assert_abs_diff_eq!(max_vol.increase().value(), MAX_VOLUME);
    }

    #[test]
    fn decrease_subtracts_step() {
        let vol = Volume::new(0.5);
        let quieter = vol.decrease();
        assert_abs_diff_eq!(quieter.value(), 0.5 - VOLUME_STEP, epsilon = 0.001);

        // At min, stays at min
        let min_vol = Volume::new(MIN_VOLUME);
        assert_abs_diff_eq!(min_vol.decrease().value(), MIN_VOLUME);
    }

    #[test]
    fn is_min_and_is_max() {
        assert!(Volume::new(MIN_VOLUME).is_min());
        assert!(!Volume::new(0.5).is_min());

        assert!(Volume::new(MAX_VOLUME).is_max());
        assert!(!Volume::new(0.5).is_max());
    }
}
