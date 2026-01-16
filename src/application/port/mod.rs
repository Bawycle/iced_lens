// SPDX-License-Identifier: MPL-2.0
//! Port definitions (traits) for dependency inversion.
//!
//! This module defines abstract interfaces that infrastructure adapters implement.
//! These traits use only domain types, ensuring the application layer remains
//! independent of concrete implementations.
//!
//! # Available Ports
//!
//! - [`ai`]: AI processing capabilities (deblur, upscale)
//! - [`media`]: Media loading and format detection
//! - [`metadata`]: Metadata reading and writing (EXIF, XMP)
//! - [`video`]: Video decoding and playback
//!
//! # Design Notes
//!
//! - All traits use domain types only (no Iced handles, no `FFmpeg` types)
//! - Traits are `Send + Sync` where appropriate for thread-safe usage
//! - Methods return `Result` with domain error types
//! - No `async fn` - use Iced's `Task` return type pattern in callers
//!
//! # Example
//!
//! ```ignore
//! use iced_lens::application::port::video::VideoDecoder;
//! use iced_lens::domain::media::RawImage;
//! use std::path::Path;
//!
//! fn decode_first_frame(decoder: &mut impl VideoDecoder, path: &Path) -> Option<RawImage> {
//!     decoder.open(path).ok()?;
//!     decoder.decode_frame().ok().flatten()
//! }
//! ```

pub mod ai;
pub mod media;
pub mod metadata;
pub mod video;

// Re-export main types for convenience
pub use ai::{AIError, AIProcessor, ProcessorCapabilities};
pub use media::{LoadedMedia, MediaError, MediaLoader};
pub use metadata::{MediaMetadata, MetadataError, MetadataReader, MetadataWriter};
pub use video::VideoDecoder;
