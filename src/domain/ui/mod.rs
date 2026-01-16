// SPDX-License-Identifier: MPL-2.0
//! UI domain types.
//!
//! This module contains UI-related value objects that are independent
//! of any presentation framework.

pub mod newtypes;

// Re-export commonly used types
pub use newtypes::{OverlayTimeout, RotationAngle, ZoomPercent, ZoomStep};
