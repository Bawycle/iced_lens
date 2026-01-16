// SPDX-License-Identifier: MPL-2.0
//! Video playback domain types.
//!
//! This module contains video-related value objects and enums that are
//! independent of any presentation or infrastructure concerns.

pub mod newtypes;
pub mod playback;

// Re-export commonly used types
pub use newtypes::{KeyboardSeekStep, PlaybackSpeed, Volume};
pub use playback::PlaybackState;
