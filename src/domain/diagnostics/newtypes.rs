// SPDX-License-Identifier: MPL-2.0
//! Diagnostics newtypes.
//!
//! This module provides type-safe wrappers for diagnostics values,
//! ensuring they are always within valid ranges.

// =============================================================================
// Buffer Capacity Bounds
// =============================================================================

/// Buffer capacity bounds (100 to 10000 events).
pub mod buffer_capacity_bounds {
    /// Minimum buffer capacity.
    pub const MIN: usize = 100;
    /// Maximum buffer capacity.
    pub const MAX: usize = 10000;
    /// Default buffer capacity.
    pub const DEFAULT: usize = 1000;
}

// =============================================================================
// BufferCapacity
// =============================================================================

/// Buffer capacity for diagnostic events.
///
/// This newtype enforces validity at the type level, ensuring the value
/// is always within the valid range (100–10000 events).
///
/// # Example
///
/// ```ignore
/// let capacity = BufferCapacity::new(1000);
/// assert_eq!(capacity.value(), 1000);
///
/// // Values outside range are clamped
/// let too_high = BufferCapacity::new(50000);
/// assert_eq!(too_high.value(), 10000); // Clamped to max
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufferCapacity(usize);

impl BufferCapacity {
    /// Creates a new buffer capacity, clamping to valid range.
    #[must_use]
    pub fn new(value: usize) -> Self {
        Self(value.clamp(buffer_capacity_bounds::MIN, buffer_capacity_bounds::MAX))
    }

    /// Returns the value as usize.
    #[must_use]
    pub fn value(self) -> usize {
        self.0
    }

    /// Returns true if this is the minimum value.
    #[must_use]
    pub fn is_min(self) -> bool {
        self.0 <= buffer_capacity_bounds::MIN
    }

    /// Returns true if this is the maximum value.
    #[must_use]
    pub fn is_max(self) -> bool {
        self.0 >= buffer_capacity_bounds::MAX
    }
}

impl Default for BufferCapacity {
    fn default() -> Self {
        Self(buffer_capacity_bounds::DEFAULT)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_capacity_clamps() {
        assert_eq!(BufferCapacity::new(0).value(), buffer_capacity_bounds::MIN);
        assert_eq!(
            BufferCapacity::new(100_000).value(),
            buffer_capacity_bounds::MAX
        );
    }

    #[test]
    fn buffer_capacity_default() {
        assert_eq!(
            BufferCapacity::default().value(),
            buffer_capacity_bounds::DEFAULT
        );
    }

    #[test]
    fn buffer_capacity_accepts_valid_values() {
        assert_eq!(BufferCapacity::new(100).value(), 100);
        assert_eq!(BufferCapacity::new(1000).value(), 1000);
        assert_eq!(BufferCapacity::new(5000).value(), 5000);
    }

    #[test]
    fn buffer_capacity_min_max() {
        assert!(BufferCapacity::new(buffer_capacity_bounds::MIN).is_min());
        assert!(BufferCapacity::new(buffer_capacity_bounds::MAX).is_max());
        assert!(!BufferCapacity::new(1000).is_min());
        assert!(!BufferCapacity::new(1000).is_max());
    }

    #[test]
    fn buffer_capacity_equality() {
        assert_eq!(BufferCapacity::new(500), BufferCapacity::new(500));
        assert_ne!(BufferCapacity::new(500), BufferCapacity::new(1000));
    }
}
