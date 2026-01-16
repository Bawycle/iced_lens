// SPDX-License-Identifier: MPL-2.0
//! Maximum skip attempts domain type for navigation.
//!
//! This module re-exports the domain type and provides backward compatibility.

// Re-export domain type
#[allow(unused_imports)] // Used by tests and may be used by external consumers
pub use crate::domain::editing::newtypes::skip_bounds;
pub use crate::domain::editing::MaxSkipAttempts;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DEFAULT_MAX_SKIP_ATTEMPTS, MAX_MAX_SKIP_ATTEMPTS, MIN_MAX_SKIP_ATTEMPTS};

    // Verify domain bounds match config constants
    #[test]
    fn domain_bounds_match_config() {
        assert_eq!(skip_bounds::MIN, MIN_MAX_SKIP_ATTEMPTS);
        assert_eq!(skip_bounds::MAX, MAX_MAX_SKIP_ATTEMPTS);
        assert_eq!(skip_bounds::DEFAULT, DEFAULT_MAX_SKIP_ATTEMPTS);
    }

    #[test]
    fn new_clamps_to_valid_range() {
        assert_eq!(MaxSkipAttempts::new(0).value(), skip_bounds::MIN);
        assert_eq!(MaxSkipAttempts::new(100).value(), skip_bounds::MAX);
    }

    #[test]
    fn new_accepts_valid_values() {
        assert_eq!(MaxSkipAttempts::new(1).value(), 1);
        assert_eq!(MaxSkipAttempts::new(10).value(), 10);
        assert_eq!(MaxSkipAttempts::new(20).value(), 20);
    }

    #[test]
    fn default_returns_expected_value() {
        assert_eq!(MaxSkipAttempts::default().value(), skip_bounds::DEFAULT);
    }

    #[test]
    fn is_min_detects_minimum() {
        assert!(MaxSkipAttempts::new(1).is_min());
        assert!(!MaxSkipAttempts::new(5).is_min());
    }

    #[test]
    fn is_max_detects_maximum() {
        assert!(MaxSkipAttempts::new(20).is_max());
        assert!(!MaxSkipAttempts::new(5).is_max());
    }

    #[test]
    fn equality_works() {
        assert_eq!(MaxSkipAttempts::new(5), MaxSkipAttempts::new(5));
        assert_ne!(MaxSkipAttempts::new(5), MaxSkipAttempts::new(10));
    }
}
