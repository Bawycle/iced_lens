// SPDX-License-Identifier: MPL-2.0
//! Infrastructure layer adapters.
//!
//! This module contains concrete implementations of the port traits defined in
//! `application::port`. These adapters wrap external dependencies like `FFmpeg`,
//! ONNX Runtime, and system I/O.
//!
//! # Available Adapters
//!
//! - [`ffmpeg`]: Video decoding via `FFmpeg` (implements [`VideoDecoder`])
//! - [`onnx`]: AI processing via ONNX Runtime (implements [`AIProcessor`])
//! - [`diagnostics`]: Diagnostics collection and export
//!
//! # Design Notes
//!
//! - Adapters implement traits from `application::port`
//! - They wrap existing implementations with minimal changes
//! - Re-exports maintain backward compatibility
//!
//! [`VideoDecoder`]: crate::application::port::VideoDecoder
//! [`AIProcessor`]: crate::application::port::AIProcessor

pub mod diagnostics;
pub mod ffmpeg;
pub mod onnx;

// Re-export main types for convenience
pub use diagnostics::{DiagnosticsCollector, DiagnosticsExporter, DiagnosticsHandle};
pub use ffmpeg::FfmpegVideoDecoder;
pub use onnx::{OnnxDeblurProcessor, OnnxUpscaleProcessor};
