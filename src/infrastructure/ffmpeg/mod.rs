// SPDX-License-Identifier: MPL-2.0
//! `FFmpeg` adapter implementing the [`VideoDecoder`] port trait.
//!
//! This module provides [`FfmpegVideoDecoder`], a synchronous video decoder
//! that wraps `FFmpeg` for basic frame decoding operations.
//!
//! # Design Notes
//!
//! - This adapter provides a simple synchronous interface
//! - For UI playback with A/V sync, use [`AsyncDecoder`] in `video_player`
//! - The decoder maintains internal state (current position, decoder context)
//!
//! [`VideoDecoder`]: crate::application::port::VideoDecoder
//! [`AsyncDecoder`]: crate::video_player::AsyncDecoder

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use crate::application::port::VideoDecoder;
use crate::domain::error::VideoError;
use crate::domain::media::{RawImage, VideoMetadata};

/// `FFmpeg`-based video decoder implementing the [`VideoDecoder`] trait.
///
/// This provides a synchronous interface for frame-by-frame video decoding.
/// It wraps `FFmpeg` directly and maintains internal decoder state.
///
/// # Thread Safety
///
/// This type is `Send` but not `Sync` due to internal mutable state.
/// Create separate instances for concurrent decoding operations.
///
/// # Example
///
/// ```ignore
/// use iced_lens::infrastructure::ffmpeg::FfmpegVideoDecoder;
/// use iced_lens::application::port::VideoDecoder;
///
/// let mut decoder = FfmpegVideoDecoder::new();
/// let metadata = decoder.open(Path::new("video.mp4"))?;
///
/// while let Some(frame) = decoder.decode_frame()? {
///     println!("Frame: {}x{}", frame.width(), frame.height());
/// }
/// ```
pub struct FfmpegVideoDecoder {
    /// Decoder state (wrapped for Send safety).
    state: Option<DecoderState>,
    /// Current position in seconds.
    position_secs: f64,
    /// Video dimensions.
    width: u32,
    height: u32,
    /// Video duration in seconds.
    duration_secs: f64,
    /// Frame rate.
    fps: f64,
    /// Whether the video has an audio track.
    has_audio: bool,
}

/// Internal decoder state that holds `FFmpeg` contexts.
///
/// This is kept separate to manage the non-Send `FFmpeg` types properly.
/// The state is created fresh for each file and dropped when done.
struct DecoderState {
    /// Input format context.
    input_context: ffmpeg_next::format::context::Input,
    /// Video decoder.
    decoder: ffmpeg_next::decoder::Video,
    /// Video stream index.
    video_stream_index: usize,
    /// Time base for PTS conversion.
    time_base_f64: f64,
    /// Source pixel format (for scaler creation).
    src_format: ffmpeg_next::format::Pixel,
}

// SAFETY: DecoderState contains FFmpeg types with internal raw pointers.
// These are safe to send between threads because:
// 1. FFmpeg's decoder/format contexts are thread-safe for single-threaded access per instance
// 2. We maintain exclusive access through Rust's ownership model
// 3. The decoder is only used from one thread at a time (move semantics)
unsafe impl Send for DecoderState {}

impl Default for FfmpegVideoDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl FfmpegVideoDecoder {
    /// Creates a new `FFmpeg` video decoder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: None,
            position_secs: 0.0,
            width: 0,
            height: 0,
            duration_secs: 0.0,
            fps: 0.0,
            has_audio: false,
        }
    }

    /// Creates a fresh scaler for RGBA conversion.
    fn create_scaler(
        src_format: ffmpeg_next::format::Pixel,
        width: u32,
        height: u32,
    ) -> Result<ffmpeg_next::software::scaling::Context, VideoError> {
        ffmpeg_next::software::scaling::Context::get(
            src_format,
            width,
            height,
            ffmpeg_next::format::Pixel::RGBA,
            width,
            height,
            ffmpeg_next::software::scaling::Flags::BILINEAR,
        )
        .map_err(|e| VideoError::Other(format!("Failed to create scaler: {e}")))
    }

    /// Extracts RGBA data from a decoded frame, handling stride correctly.
    #[allow(clippy::cast_possible_truncation)] // stride is always < u32::MAX for video frames
    fn extract_rgba_data(frame: &ffmpeg_next::frame::Video) -> Vec<u8> {
        let width = frame.width();
        let height = frame.height();
        let data = frame.data(0);
        let stride = frame.stride(0);

        let mut rgba_bytes = Vec::with_capacity((width * height * 4) as usize);
        for y in 0..height {
            let row_start = (y * stride as u32) as usize;
            let row_end = row_start + (width * 4) as usize;
            rgba_bytes.extend_from_slice(&data[row_start..row_end]);
        }

        rgba_bytes
    }

    /// Converts a decoded frame to `RawImage` and updates position.
    ///
    /// This is a static method to avoid borrow checker issues when both
    /// state and self need to be modified.
    fn convert_frame_static(
        decoded_frame: &ffmpeg_next::frame::Video,
        scaler: &mut ffmpeg_next::software::scaling::Context,
        time_base_f64: f64,
        width: u32,
        height: u32,
        position_secs: &mut f64,
    ) -> Result<Option<RawImage>, VideoError> {
        // Update position from PTS
        #[allow(clippy::cast_precision_loss)]
        if let Some(pts) = decoded_frame.timestamp() {
            *position_secs = pts as f64 * time_base_f64;
        }

        // Scale to RGBA
        let mut rgb_frame = ffmpeg_next::frame::Video::empty();
        scaler
            .run(decoded_frame, &mut rgb_frame)
            .map_err(|e| VideoError::DecodingFailed(format!("Scaling failed: {e}")))?;

        // Extract RGBA data
        let rgba_data = Self::extract_rgba_data(&rgb_frame);

        Ok(Some(RawImage::new(width, height, Arc::new(rgba_data))))
    }
}

impl VideoDecoder for FfmpegVideoDecoder {
    fn open(&mut self, path: &Path) -> Result<VideoMetadata, VideoError> {
        // Initialize FFmpeg
        crate::media::video::init_ffmpeg()
            .map_err(|e| VideoError::Other(format!("Failed to initialize FFmpeg: {e}")))?;

        // Open input file
        let input_context = ffmpeg_next::format::input(path)
            .map_err(|e| VideoError::Other(format!("Failed to open video: {e}")))?;

        // Find video stream
        let video_stream = input_context
            .streams()
            .best(ffmpeg_next::media::Type::Video)
            .ok_or(VideoError::NoVideoStream)?;

        let video_stream_index = video_stream.index();

        // Get duration
        #[allow(clippy::cast_precision_loss)]
        let duration_secs = if input_context.duration() > 0 {
            input_context.duration() as f64 / f64::from(ffmpeg_next::ffi::AV_TIME_BASE)
        } else {
            0.0
        };

        // Get frame rate
        let fps = video_stream.avg_frame_rate();
        #[allow(clippy::cast_precision_loss)]
        let fps_f64 = if fps.denominator() != 0 {
            f64::from(fps.numerator()) / f64::from(fps.denominator())
        } else {
            30.0 // Default fallback
        };

        // Check for audio stream
        let has_audio = input_context
            .streams()
            .best(ffmpeg_next::media::Type::Audio)
            .is_some();

        // Create decoder
        let context_decoder =
            ffmpeg_next::codec::context::Context::from_parameters(video_stream.parameters())
                .map_err(|e| VideoError::Other(format!("Failed to create codec context: {e}")))?;

        let decoder = context_decoder
            .decoder()
            .video()
            .map_err(|e| VideoError::Other(format!("Failed to create video decoder: {e}")))?;

        let width = decoder.width();
        let height = decoder.height();
        let src_format = decoder.format();

        // Extract time base
        let time_base = video_stream.time_base();
        let time_base_f64 = f64::from(time_base.numerator()) / f64::from(time_base.denominator());

        // Store state
        self.state = Some(DecoderState {
            input_context,
            decoder,
            video_stream_index,
            time_base_f64,
            src_format,
        });
        self.position_secs = 0.0;
        self.width = width;
        self.height = height;
        self.duration_secs = duration_secs;
        self.fps = fps_f64;
        self.has_audio = has_audio;

        Ok(VideoMetadata::new(
            width,
            height,
            duration_secs,
            fps_f64,
            has_audio,
        ))
    }

    fn decode_frame(&mut self) -> Result<Option<RawImage>, VideoError> {
        // Extract state info needed for scaler creation
        let (src_format, time_base_f64, video_stream_index) = {
            let state = self
                .state
                .as_ref()
                .ok_or_else(|| VideoError::Other("Decoder not opened".to_string()))?;
            (state.src_format, state.time_base_f64, state.video_stream_index)
        };

        // Create scaler for this frame (recreated each time for Send safety)
        let mut scaler = Self::create_scaler(src_format, self.width, self.height)?;

        // First try to receive a buffered frame
        let mut decoded_frame = ffmpeg_next::frame::Video::empty();
        {
            let state = self.state.as_mut().unwrap(); // Safe: checked above
            if state.decoder.receive_frame(&mut decoded_frame).is_ok() {
                return Self::convert_frame_static(
                    &decoded_frame,
                    &mut scaler,
                    time_base_f64,
                    self.width,
                    self.height,
                    &mut self.position_secs,
                );
            }
        }

        // Process packets until we get a frame or reach end of stream
        loop {
            let state = self.state.as_mut().unwrap(); // Safe: checked above

            // Get next video packet
            let packet_opt = state.input_context.packets().find(|(stream, _)| {
                stream.index() == video_stream_index
            });

            match packet_opt {
                Some((_, packet)) => {
                    if let Err(e) = state.decoder.send_packet(&packet) {
                        return Err(VideoError::DecodingFailed(format!(
                            "Packet send failed: {e}"
                        )));
                    }

                    if state.decoder.receive_frame(&mut decoded_frame).is_ok() {
                        return Self::convert_frame_static(
                            &decoded_frame,
                            &mut scaler,
                            time_base_f64,
                            self.width,
                            self.height,
                            &mut self.position_secs,
                        );
                    }
                }
                None => {
                    // End of stream
                    return Ok(None);
                }
            }
        }
    }

    fn seek(&mut self, position: Duration) -> Result<(), VideoError> {
        let state = self
            .state
            .as_mut()
            .ok_or_else(|| VideoError::Other("Decoder not opened".to_string()))?;

        let target_secs = position.as_secs_f64();

        // Convert to timestamp
        #[allow(clippy::cast_possible_truncation)]
        let timestamp = (target_secs * 1_000_000.0) as i64;

        // Seek to position
        state
            .input_context
            .seek(timestamp, ..timestamp)
            .map_err(|e| VideoError::DecodingFailed(format!("Seek failed: {e}")))?;

        // Flush decoder
        state.decoder.flush();

        self.position_secs = target_secs;
        Ok(())
    }

    fn position(&self) -> Duration {
        Duration::from_secs_f64(self.position_secs)
    }

    fn reset(&mut self) -> Result<(), VideoError> {
        self.seek(Duration::ZERO)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decoder_can_be_created() {
        let decoder = FfmpegVideoDecoder::new();
        assert_eq!(decoder.position(), Duration::ZERO);
    }

    #[test]
    fn decoder_default_is_same_as_new() {
        let decoder = FfmpegVideoDecoder::default();
        assert_eq!(decoder.position(), Duration::ZERO);
    }

    #[test]
    fn decode_frame_fails_when_not_opened() {
        let mut decoder = FfmpegVideoDecoder::new();
        let result = decoder.decode_frame();
        assert!(result.is_err());
    }

    #[test]
    fn seek_fails_when_not_opened() {
        let mut decoder = FfmpegVideoDecoder::new();
        let result = decoder.seek(Duration::from_secs(5));
        assert!(result.is_err());
    }

    #[test]
    fn reset_fails_when_not_opened() {
        let mut decoder = FfmpegVideoDecoder::new();
        let result = decoder.reset();
        assert!(result.is_err());
    }

    // Verify Send is implemented
    fn assert_send<T: Send>() {}

    #[test]
    fn decoder_is_send() {
        assert_send::<FfmpegVideoDecoder>();
    }
}
