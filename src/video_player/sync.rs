// SPDX-License-Identifier: MPL-2.0
//! Audio/Video synchronization for media playback.
//!
//! This module provides synchronization between audio and video streams,
//! using audio as the master clock (standard practice for A/V sync).
//!
//! # Synchronization Strategy
//!
//! Audio playback drives the timing because:
//! - Audio discontinuities are more noticeable than video frame drops
//! - Audio sample rate provides a precise time reference
//! - Video frames can be skipped or repeated to match audio timing
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────┐     ┌──────────────┐
//! │ AudioDecoder│────▶│ AudioOutput  │──▶ Audio clock (master)
//! └─────────────┘     └──────────────┘           │
//!                                                │ PTS reference
//!                                                ▼
//! ┌─────────────┐     ┌──────────────┐    ┌──────────────┐
//! │ VideoDecoder│────▶│ Frame Buffer │───▶│ Display Sync │
//! └─────────────┘     └──────────────┘    └──────────────┘
//! ```

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

/// Reference instant for converting `Instant` to/from atomic microseconds.
/// All time measurements are relative to this instant, allowing storage in `AtomicU64`.
static REFERENCE_INSTANT: OnceLock<Instant> = OnceLock::new();

/// Converts an `Instant` to microseconds since the reference instant.
#[allow(clippy::cast_possible_truncation)] // u128 microseconds won't overflow u64 for reasonable durations
fn instant_to_us(instant: Instant) -> u64 {
    let reference = REFERENCE_INSTANT.get_or_init(Instant::now);
    instant.duration_since(*reference).as_micros() as u64
}

/// Converts microseconds since reference back to an `Instant`.
/// Returns `None` for the sentinel value 0.
fn us_to_instant(us: u64) -> Option<Instant> {
    if us == 0 {
        return None;
    }
    let reference = REFERENCE_INSTANT.get_or_init(Instant::now);
    Some(*reference + Duration::from_micros(us))
}

/// Synchronization tolerance in seconds.
/// If audio and video differ by more than this, sync correction is applied.
pub const SYNC_TOLERANCE_SECS: f64 = 0.05; // 50ms

/// Maximum frames to skip when video is behind audio.
pub const MAX_FRAME_SKIP: u32 = 5;

/// Audio/video synchronization clock.
///
/// Tracks the current playback position based on audio output,
/// allowing video frames to synchronize to the audio timeline.
///
/// This struct is fully lock-free, using atomics for all fields.
#[derive(Debug)]
pub struct SyncClock {
    /// Current audio PTS in microseconds (for atomic access).
    audio_pts_us: AtomicU64,

    /// Playback start time as microseconds since `REFERENCE_INSTANT`.
    /// 0 means no start time is set.
    start_time_us: AtomicU64,

    /// Audio PTS at playback start.
    start_pts_us: AtomicU64,

    /// Whether playback is active.
    is_playing: AtomicBool,

    /// Whether sync is enabled (can be disabled for testing).
    sync_enabled: AtomicBool,
}

impl Default for SyncClock {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncClock {
    /// Creates a new sync clock.
    #[must_use]
    pub fn new() -> Self {
        Self {
            audio_pts_us: AtomicU64::new(0),
            start_time_us: AtomicU64::new(0),
            start_pts_us: AtomicU64::new(0),
            is_playing: AtomicBool::new(false),
            sync_enabled: AtomicBool::new(true),
        }
    }

    /// Starts the sync clock at the given audio PTS.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn start(&self, audio_pts_secs: f64) {
        let pts_us = (audio_pts_secs * 1_000_000.0) as u64;
        self.audio_pts_us.store(pts_us, Ordering::SeqCst);
        self.start_pts_us.store(pts_us, Ordering::SeqCst);
        self.start_time_us
            .store(instant_to_us(Instant::now()), Ordering::SeqCst);
        self.is_playing.store(true, Ordering::SeqCst);
    }

    /// Pauses the sync clock, preserving current position.
    pub fn pause(&self) {
        self.is_playing.store(false, Ordering::SeqCst);
    }

    /// Resumes the sync clock from the paused position.
    pub fn resume(&self) {
        let current_pts_us = self.audio_pts_us.load(Ordering::SeqCst);
        self.start_pts_us.store(current_pts_us, Ordering::SeqCst);
        self.start_time_us
            .store(instant_to_us(Instant::now()), Ordering::SeqCst);
        self.is_playing.store(true, Ordering::SeqCst);
    }

    /// Stops the sync clock and resets to beginning.
    pub fn stop(&self) {
        self.audio_pts_us.store(0, Ordering::SeqCst);
        self.start_pts_us.store(0, Ordering::SeqCst);
        self.start_time_us.store(0, Ordering::SeqCst);
        self.is_playing.store(false, Ordering::SeqCst);
    }

    /// Updates the audio PTS (called when audio buffer is played).
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn update_audio_pts(&self, pts_secs: f64) {
        let pts_us = (pts_secs * 1_000_000.0) as u64;
        self.audio_pts_us.store(pts_us, Ordering::SeqCst);
    }

    /// Returns the current playback position in seconds.
    ///
    /// If playing, interpolates based on wall clock time since last audio update.
    /// If paused, returns the last known position.
    #[must_use]
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    pub fn current_time_secs(&self) -> f64 {
        let pts_us = self.audio_pts_us.load(Ordering::SeqCst);

        if self.is_playing.load(Ordering::SeqCst) {
            // Interpolate based on wall clock
            if let Some(start) = us_to_instant(self.start_time_us.load(Ordering::SeqCst)) {
                let start_pts_us = self.start_pts_us.load(Ordering::SeqCst);
                let elapsed = start.elapsed();
                let elapsed_us = elapsed.as_micros() as u64;
                let interpolated_us = start_pts_us + elapsed_us;
                // Use the more recent of interpolated or last audio PTS
                return (interpolated_us.max(pts_us)) as f64 / 1_000_000.0;
            }
        }

        pts_us as f64 / 1_000_000.0
    }

    /// Returns whether playback is active.
    #[must_use]
    pub fn is_playing(&self) -> bool {
        self.is_playing.load(Ordering::SeqCst)
    }

    /// Enables or disables sync correction.
    pub fn set_sync_enabled(&self, enabled: bool) {
        self.sync_enabled.store(enabled, Ordering::SeqCst);
    }

    /// Returns whether sync correction is enabled.
    #[must_use]
    pub fn is_sync_enabled(&self) -> bool {
        self.sync_enabled.load(Ordering::SeqCst)
    }

    /// Seeks to a specific position.
    ///
    /// Always resets the wall-time reference to prevent A/V sync drift
    /// when resuming playback after seek.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn seek(&self, target_secs: f64) {
        let pts_us = (target_secs * 1_000_000.0) as u64;
        self.audio_pts_us.store(pts_us, Ordering::SeqCst);
        self.start_pts_us.store(pts_us, Ordering::SeqCst);
        // Always reset wall-time reference during seek to prevent A/V sync drift
        // that can cause frame skipping when playback resumes.
        self.start_time_us
            .store(instant_to_us(Instant::now()), Ordering::SeqCst);
    }
}

/// Determines the sync action for a video frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SyncAction {
    /// Display the frame now.
    Display,

    /// Wait before displaying (frame is early).
    Wait(Duration),

    /// Skip this frame (video is behind audio).
    Skip,

    /// Repeat previous frame (video is ahead, no new frame ready).
    Repeat,
}

/// Calculates the sync action for a video frame given the current audio time.
///
/// # Arguments
/// * `video_pts_secs` - The PTS of the video frame
/// * `audio_time_secs` - The current audio playback time
///
/// # Returns
/// The action to take for this video frame.
#[must_use]
pub fn calculate_sync_action(video_pts_secs: f64, audio_time_secs: f64) -> SyncAction {
    let diff = video_pts_secs - audio_time_secs;

    if diff.abs() <= SYNC_TOLERANCE_SECS {
        // Within tolerance, display immediately
        SyncAction::Display
    } else if diff > 0.0 {
        // Video is ahead of audio, wait
        SyncAction::Wait(Duration::from_secs_f64(diff))
    } else {
        // Video is behind audio, skip
        SyncAction::Skip
    }
}

/// Thread-safe wrapper around `SyncClock` for sharing between threads.
pub type SharedSyncClock = Arc<SyncClock>;

/// Creates a new shared sync clock.
#[must_use]
pub fn create_sync_clock() -> SharedSyncClock {
    Arc::new(SyncClock::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_clock_starts_at_zero() {
        let clock = SyncClock::new();
        assert!((clock.current_time_secs() - 0.0).abs() < 0.001);
        assert!(!clock.is_playing());
    }

    #[test]
    fn sync_clock_start_sets_position() {
        let clock = SyncClock::new();
        clock.start(10.5);
        assert!(clock.is_playing());
        // Should be close to 10.5 (may have slight drift due to timing)
        let time = clock.current_time_secs();
        assert!((10.5..11.0).contains(&time));
    }

    #[test]
    fn sync_clock_pause_preserves_position() {
        let clock = SyncClock::new();
        clock.start(5.0);
        std::thread::sleep(Duration::from_millis(50));
        clock.pause();

        let paused_time = clock.current_time_secs();
        std::thread::sleep(Duration::from_millis(50));
        let after_pause_time = clock.current_time_secs();

        // Time should not advance while paused
        assert!((paused_time - after_pause_time).abs() < 0.001);
        assert!(!clock.is_playing());
    }

    #[test]
    fn sync_clock_stop_resets_to_zero() {
        let clock = SyncClock::new();
        clock.start(30.0);
        clock.stop();

        assert!(!clock.is_playing());
        assert!((clock.current_time_secs() - 0.0).abs() < 0.001);
    }

    #[test]
    fn sync_clock_update_audio_pts() {
        let clock = SyncClock::new();
        clock.update_audio_pts(15.5);

        // When not playing, returns last audio PTS
        assert!((clock.current_time_secs() - 15.5).abs() < 0.001);
    }

    #[test]
    fn sync_clock_seek_updates_position() {
        let clock = SyncClock::new();
        clock.start(0.0);
        clock.seek(45.0);

        let time = clock.current_time_secs();
        assert!((45.0..46.0).contains(&time));
    }

    #[test]
    fn calculate_sync_action_display_within_tolerance() {
        // Video slightly ahead but within tolerance
        let action = calculate_sync_action(10.02, 10.0);
        assert_eq!(action, SyncAction::Display);

        // Video slightly behind but within tolerance
        let action = calculate_sync_action(9.98, 10.0);
        assert_eq!(action, SyncAction::Display);

        // Exactly matched
        let action = calculate_sync_action(10.0, 10.0);
        assert_eq!(action, SyncAction::Display);
    }

    #[test]
    fn calculate_sync_action_wait_when_video_ahead() {
        let action = calculate_sync_action(10.5, 10.0);
        match action {
            SyncAction::Wait(duration) => {
                assert!((duration.as_secs_f64() - 0.5).abs() < 0.001);
            }
            _ => panic!("Expected Wait action"),
        }
    }

    #[test]
    fn calculate_sync_action_skip_when_video_behind() {
        let action = calculate_sync_action(9.0, 10.0);
        assert_eq!(action, SyncAction::Skip);
    }

    #[test]
    fn sync_tolerance_is_reasonable() {
        // 50ms is a reasonable tolerance for A/V sync
        // Check at runtime to ensure the constant stays within bounds
        let tolerance = SYNC_TOLERANCE_SECS;
        assert!(tolerance >= 0.02, "Sync tolerance should be at least 20ms");
        assert!(
            tolerance <= 0.1,
            "Sync tolerance should be no more than 100ms"
        );
    }

    #[test]
    fn shared_sync_clock_can_be_cloned() {
        let clock = create_sync_clock();
        let clock2 = Arc::clone(&clock);

        clock.start(5.0);
        assert!(clock2.is_playing());
    }

    #[test]
    fn sync_enabled_flag_works() {
        let clock = SyncClock::new();
        assert!(clock.is_sync_enabled()); // Default is enabled

        clock.set_sync_enabled(false);
        assert!(!clock.is_sync_enabled());

        clock.set_sync_enabled(true);
        assert!(clock.is_sync_enabled());
    }
}
