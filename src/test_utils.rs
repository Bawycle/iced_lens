// SPDX-License-Identifier: MPL-2.0
//! Test utilities for float comparisons and other common test helpers.
//!
//! This module re-exports the `approx` crate's assertion macros for float comparison,
//! which properly handle floating-point precision issues that `assert_eq!` cannot.

// Re-export approx macros for convenient use in tests
pub use approx::{assert_abs_diff_eq, assert_abs_diff_ne, assert_relative_eq, assert_relative_ne};

/// Default epsilon for f32 comparisons.
/// Suitable for values that should be "exactly equal" but may have minor floating-point errors.
pub const F32_EPSILON: f32 = 1e-6;

/// Default epsilon for f64 comparisons.
/// Suitable for values that should be "exactly equal" but may have minor floating-point errors.
pub const F64_EPSILON: f64 = 1e-10;
