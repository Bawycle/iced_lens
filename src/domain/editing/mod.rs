// SPDX-License-Identifier: MPL-2.0
//! Editing domain types.
//!
//! This module provides pure domain types for image editing operations:
//! - [`ResizeScale`]: Scale percentage for image resizing
//! - [`AdjustmentPercent`]: Brightness/contrast adjustment value
//! - [`MaxSkipAttempts`]: Navigation skip attempts limit

pub mod newtypes;

pub use newtypes::{AdjustmentPercent, MaxSkipAttempts, ResizeScale};
