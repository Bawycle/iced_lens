// SPDX-License-Identifier: MPL-2.0
//! Metadata domain types.
//!
//! This module provides pure domain types for media metadata.
//!
//! # Current Status
//!
//! The existing metadata types (`ImageMetadata`, `DublinCoreMetadata`, etc.)
//! in `src/media/metadata.rs` and `src/media/xmp.rs` are already pure data
//! containers. However, their extraction logic depends on external crates
//! (`exif`, `quick_xml`, `ffmpeg_next`).
//!
//! This domain module provides:
//! - [`GpsCoordinates`]: Pure GPS coordinate representation
//!
//! # Future Extensions
//!
//! Additional pure domain types could be added here:
//! - Structured EXIF data types
//! - Structured XMP/Dublin Core types
//! - ISO 8601 `DateTime` wrapper

mod types;

pub use types::GpsCoordinates;
