// SPDX-License-Identifier: MPL-2.0
//! Video decoding port definition.
//!
//! This module defines the [`VideoDecoder`] trait for video decoding operations.
//! Infrastructure adapters (like `FFmpeg`) implement this trait.
//!
//! # Design Notes
//!
//! - The decoder is **stateful** - it maintains the current playback position
//! - Methods are not `async` - the Iced framework handles threading via `Task`
//! - Audio decoding is handled separately by infrastructure (A/V sync is complex)
//! - Uses domain types only (`RawImage`, `VideoMetadata`, `VideoError`)

use crate::domain::error::VideoError;
use crate::domain::media::{RawImage, VideoMetadata};
use std::path::Path;
use std::time::Duration;

// =============================================================================
// VideoDecoder Trait
// =============================================================================

/// Port for video decoding operations.
///
/// This trait defines the interface for decoding video files frame by frame.
/// Implementations maintain internal state (current position, decoder context).
///
/// # Thread Safety
///
/// Implementations must be `Send` for use across threads. The decoder is
/// **not** required to be `Sync` since it maintains mutable state.
///
/// # Lifecycle
///
/// 1. Create decoder instance
/// 2. Call `open()` to open a video file
/// 3. Call `decode_frame()` repeatedly to get frames
/// 4. Use `seek()` to jump to specific positions
/// 5. Call `reset()` to start from the beginning
///
/// # Example
///
/// ```ignore
/// use iced_lens::application::port::video::VideoDecoder;
/// use std::path::Path;
///
/// fn play_first_second(decoder: &mut impl VideoDecoder, path: &Path) {
///     let meta = decoder.open(path).expect("Failed to open");
///     println!("Video: {}x{}, {:.1}s", meta.width, meta.height, meta.duration_secs);
///
///     // Decode frames for first second
///     while decoder.position() < std::time::Duration::from_secs(1) {
///         match decoder.decode_frame() {
///             Ok(Some(frame)) => {
///                 println!("Frame: {}x{}", frame.width(), frame.height());
///             }
///             Ok(None) => break, // End of stream
///             Err(e) => {
///                 eprintln!("Decode error: {e}");
///                 break;
///             }
///         }
///     }
/// }
/// ```
pub trait VideoDecoder: Send {
    /// Opens a video file for decoding.
    ///
    /// This initializes the decoder with the video file and returns metadata.
    /// The decoder position is set to the beginning of the video.
    ///
    /// # Errors
    ///
    /// Returns a [`VideoError`] if:
    /// - The file cannot be read
    /// - The format is not supported
    /// - No video stream is found
    fn open(&mut self, path: &Path) -> Result<VideoMetadata, VideoError>;

    /// Decodes the next video frame.
    ///
    /// Returns `Ok(Some(frame))` for each decoded frame, or `Ok(None)` when
    /// the end of the video stream is reached.
    ///
    /// The decoder automatically advances its internal position after each call.
    ///
    /// # Errors
    ///
    /// Returns a [`VideoError`] if decoding fails.
    fn decode_frame(&mut self) -> Result<Option<RawImage>, VideoError>;

    /// Seeks to a specific position in the video.
    ///
    /// The next call to `decode_frame()` will return the frame at or near
    /// the specified position. Seeking is typically to the nearest keyframe.
    ///
    /// # Errors
    ///
    /// Returns a [`VideoError`] if:
    /// - The seek operation fails
    /// - The position is beyond the video duration
    fn seek(&mut self, position: Duration) -> Result<(), VideoError>;

    /// Returns the current playback position.
    ///
    /// This is the presentation timestamp of the last decoded frame.
    fn position(&self) -> Duration;

    /// Resets the decoder to the beginning of the video.
    ///
    /// This is equivalent to `seek(Duration::ZERO)` but may be more efficient.
    ///
    /// # Errors
    ///
    /// Returns a [`VideoError`] if the reset operation fails.
    fn reset(&mut self) -> Result<(), VideoError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test that the trait is object-safe
    fn _assert_object_safe(_: &dyn VideoDecoder) {}

    // Mock implementation for testing
    struct MockDecoder {
        position: Duration,
        is_open: bool,
    }

    impl MockDecoder {
        fn new() -> Self {
            Self {
                position: Duration::ZERO,
                is_open: false,
            }
        }
    }

    impl VideoDecoder for MockDecoder {
        fn open(&mut self, _path: &Path) -> Result<VideoMetadata, VideoError> {
            self.is_open = true;
            self.position = Duration::ZERO;
            Ok(VideoMetadata::new(1920, 1080, 10.0, 30.0, true))
        }

        fn decode_frame(&mut self) -> Result<Option<RawImage>, VideoError> {
            if !self.is_open {
                return Err(VideoError::Other("Not open".to_string()));
            }

            // Simulate frame duration at 30fps
            self.position += Duration::from_millis(33);

            // Simulate 10 second video
            if self.position.as_secs_f64() >= 10.0 {
                return Ok(None);
            }

            // Return a small test frame
            Ok(Some(RawImage::from_rgba(2, 2, vec![0u8; 16])))
        }

        fn seek(&mut self, position: Duration) -> Result<(), VideoError> {
            if !self.is_open {
                return Err(VideoError::Other("Not open".to_string()));
            }
            self.position = position;
            Ok(())
        }

        fn position(&self) -> Duration {
            self.position
        }

        fn reset(&mut self) -> Result<(), VideoError> {
            self.seek(Duration::ZERO)
        }
    }

    #[test]
    fn mock_decoder_lifecycle() {
        let mut decoder = MockDecoder::new();

        // Open
        let meta = decoder.open(Path::new("test.mp4")).unwrap();
        assert_eq!(meta.width, 1920);
        assert_eq!(meta.height, 1080);
        assert_eq!(decoder.position(), Duration::ZERO);

        // Decode frame
        let frame = decoder.decode_frame().unwrap().unwrap();
        assert_eq!(frame.width(), 2);
        assert!(decoder.position() > Duration::ZERO);

        // Seek
        decoder.seek(Duration::from_secs(5)).unwrap();
        assert_eq!(decoder.position(), Duration::from_secs(5));

        // Reset
        decoder.reset().unwrap();
        assert_eq!(decoder.position(), Duration::ZERO);
    }
}
