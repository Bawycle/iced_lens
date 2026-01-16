// SPDX-License-Identifier: MPL-2.0
//! Overlay timeout domain type for fullscreen UI.
//!
//! This module re-exports the domain type and provides backward compatibility.

#[cfg(test)]
use crate::config::{
    DEFAULT_OVERLAY_TIMEOUT_SECS, MAX_OVERLAY_TIMEOUT_SECS, MIN_OVERLAY_TIMEOUT_SECS,
};

// Re-export domain type
#[allow(unused_imports)] // Used by tests and may be used by external consumers
pub use crate::domain::ui::newtypes::overlay_bounds;
pub use crate::domain::ui::newtypes::OverlayTimeout;

#[cfg(test)]
mod tests {
    use super::*;

    // Verify domain bounds match config constants
    #[test]
    fn domain_bounds_match_config() {
        assert_eq!(overlay_bounds::MIN, MIN_OVERLAY_TIMEOUT_SECS);
        assert_eq!(overlay_bounds::MAX, MAX_OVERLAY_TIMEOUT_SECS);
        assert_eq!(overlay_bounds::DEFAULT, DEFAULT_OVERLAY_TIMEOUT_SECS);
    }

    #[test]
    fn new_clamps_to_valid_range() {
        assert_eq!(OverlayTimeout::new(0).value(), MIN_OVERLAY_TIMEOUT_SECS);
        assert_eq!(OverlayTimeout::new(100).value(), MAX_OVERLAY_TIMEOUT_SECS);
    }

    #[test]
    fn new_accepts_valid_values() {
        assert_eq!(OverlayTimeout::new(1).value(), 1);
        assert_eq!(OverlayTimeout::new(15).value(), 15);
        assert_eq!(OverlayTimeout::new(30).value(), 30);
    }

    #[test]
    fn default_returns_expected_value() {
        assert_eq!(
            OverlayTimeout::default().value(),
            DEFAULT_OVERLAY_TIMEOUT_SECS
        );
    }

    #[test]
    fn is_min_detects_minimum() {
        assert!(OverlayTimeout::new(1).is_min());
        assert!(!OverlayTimeout::new(15).is_min());
    }

    #[test]
    fn is_max_detects_maximum() {
        assert!(OverlayTimeout::new(30).is_max());
        assert!(!OverlayTimeout::new(15).is_max());
    }

    #[test]
    fn as_duration_converts_correctly() {
        let timeout = OverlayTimeout::new(5);
        assert_eq!(timeout.as_duration(), std::time::Duration::from_secs(5));
    }

    #[test]
    fn equality_works() {
        assert_eq!(OverlayTimeout::new(5), OverlayTimeout::new(5));
        assert_ne!(OverlayTimeout::new(5), OverlayTimeout::new(10));
    }
}
