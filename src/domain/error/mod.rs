// SPDX-License-Identifier: MPL-2.0
//! Domain error types.
//!
//! This module provides pure domain error types that are independent
//! of external crates and infrastructure concerns.

mod video;

pub use video::VideoError;
