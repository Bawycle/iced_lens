// SPDX-License-Identifier: MPL-2.0
//! Media domain types.
//!
//! This module contains core media types that are independent of any
//! presentation or infrastructure concerns.

pub mod filter;
pub mod types;

// Re-export commonly used types
pub use filter::{DateFilterField, DateRangeFilter, MediaFilter, MediaTypeFilter};
pub use types::{MediaType, RawImage, VideoMetadata};
