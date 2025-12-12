// SPDX-License-Identifier: MPL-2.0
//! Video playback engine for IcedLens.
//!
//! This module provides video playback functionality using FFmpeg for decoding
//! and async Tokio tasks for non-blocking frame delivery.

pub mod audio;
pub mod audio_output;
mod decoder;
pub mod frame_cache;
pub mod normalization;
mod state;
pub mod subscription;
pub mod sync;
pub mod time_units;
mod webp_decoder;

pub use decoder::{AsyncDecoder, DecodedFrame, DecoderCommand, DecoderEvent};
pub use frame_cache::{CacheConfig, CacheStats, FrameCache};
pub use normalization::{
    create_lufs_cache, LufsAnalyzer, LufsCache, NormalizationSettings, SharedLufsCache,
    DEFAULT_TARGET_LUFS,
};
pub use state::{PlaybackState, VideoPlayer};
pub use subscription::{video_playback, DecoderCommandSender, PlaybackMessage, VideoPlaybackId};
pub use sync::{calculate_sync_action, SharedSyncClock, SyncAction, SyncClock};
pub use webp_decoder::{WebpAnimDecoder, WebpMetadata};

use crate::error::Result;
use crate::media::VideoData;

/// Creates a new video player instance for the given video data.
pub fn create_player(video_data: &VideoData) -> Result<VideoPlayer> {
    VideoPlayer::new(video_data)
}
