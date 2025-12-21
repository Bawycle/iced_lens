// SPDX-License-Identifier: MPL-2.0
//! Maximum skip attempts domain type for navigation.
//!
//! This module provides a type-safe wrapper for the maximum number of
//! consecutive corrupted files to skip during navigation.

use crate::config::{DEFAULT_MAX_SKIP_ATTEMPTS, MAX_MAX_SKIP_ATTEMPTS, MIN_MAX_SKIP_ATTEMPTS};

/// Maximum number of consecutive corrupted files to skip during navigation.
///
/// This newtype enforces validity at the type level, ensuring the value
/// is always within the valid range (1–20).
///
/// # Example
///
/// ```
/// use iced_lens::media::MaxSkipAttempts;
///
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
    pub fn new(value: u32) -> Self {
        Self(value.clamp(MIN_MAX_SKIP_ATTEMPTS, MAX_MAX_SKIP_ATTEMPTS))
    }

    /// Returns the value as u32.
    pub fn value(self) -> u32 {
        self.0
    }

    /// Returns true if this is the minimum value.
    pub fn is_min(self) -> bool {
        self.0 <= MIN_MAX_SKIP_ATTEMPTS
    }

    /// Returns true if this is the maximum value.
    pub fn is_max(self) -> bool {
        self.0 >= MAX_MAX_SKIP_ATTEMPTS
    }
}

impl Default for MaxSkipAttempts {
    fn default() -> Self {
        Self(DEFAULT_MAX_SKIP_ATTEMPTS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_clamps_to_valid_range() {
        assert_eq!(MaxSkipAttempts::new(0).value(), MIN_MAX_SKIP_ATTEMPTS);
        assert_eq!(MaxSkipAttempts::new(100).value(), MAX_MAX_SKIP_ATTEMPTS);
    }

    #[test]
    fn new_accepts_valid_values() {
        assert_eq!(MaxSkipAttempts::new(1).value(), 1);
        assert_eq!(MaxSkipAttempts::new(10).value(), 10);
        assert_eq!(MaxSkipAttempts::new(20).value(), 20);
    }

    #[test]
    fn default_returns_expected_value() {
        assert_eq!(
            MaxSkipAttempts::default().value(),
            DEFAULT_MAX_SKIP_ATTEMPTS
        );
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
