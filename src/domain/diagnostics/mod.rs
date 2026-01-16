// SPDX-License-Identifier: MPL-2.0
//! Diagnostics domain types.
//!
//! This module provides pure domain types for diagnostics:
//! - [`BufferCapacity`]: Capacity for the diagnostic event buffer

mod newtypes;

pub use newtypes::{buffer_capacity_bounds, BufferCapacity};
