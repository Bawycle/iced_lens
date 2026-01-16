// SPDX-License-Identifier: MPL-2.0
//! Domain layer - Core business logic with ZERO external dependencies.
//!
//! This module contains pure domain types, value objects, and business rules.
//! It has no dependencies on external crates (except `std`) to ensure
//! testability and architectural purity.
//!
//! # Modules
//!
//! - [`diagnostics`]: Diagnostics types ([`BufferCapacity`](diagnostics::BufferCapacity))
//! - [`editing`]: Image editing types ([`ResizeScale`](editing::ResizeScale),
//!   [`AdjustmentPercent`](editing::AdjustmentPercent), [`MaxSkipAttempts`](editing::MaxSkipAttempts))
//! - [`error`]: Domain error types ([`VideoError`](error::VideoError))
//! - [`media`]: Media types ([`MediaType`](media::MediaType), [`RawImage`](media::RawImage),
//!   [`VideoMetadata`](media::VideoMetadata), [`MediaFilter`](media::MediaFilter))
//! - [`metadata`]: Metadata types ([`GpsCoordinates`](metadata::GpsCoordinates))
//! - [`ui`]: UI value objects ([`ZoomPercent`](ui::newtypes::ZoomPercent),
//!   [`RotationAngle`](ui::newtypes::RotationAngle), [`OverlayTimeout`](ui::newtypes::OverlayTimeout))
//! - [`video`]: Video playback types ([`PlaybackState`](video::PlaybackState),
//!   [`Volume`](video::Volume), [`PlaybackSpeed`](video::PlaybackSpeed))

pub mod diagnostics;
pub mod editing;
pub mod error;
pub mod media;
pub mod metadata;
pub mod ui;
pub mod video;
