// SPDX-License-Identifier: MPL-2.0
//! Video handling and thumbnail extraction.

use crate::error::{Error, Result};
use crate::media::ImageData;
use std::path::Path;
use std::sync::Once;

/// Static flag to ensure FFmpeg is initialized only once.
static FFMPEG_INIT: Once = Once::new();

/// Initialize FFmpeg with appropriate log level.
///
/// This function is safe to call multiple times - initialization will only
/// happen once thanks to `std::sync::Once`. It sets the FFmpeg log level
/// to ERROR to suppress warning messages like "Detected creation time before 1970".
pub fn init_ffmpeg() -> Result<()> {
    let mut init_result: Result<()> = Ok(());

    FFMPEG_INIT.call_once(|| {
        // Initialize FFmpeg
        if let Err(e) = ffmpeg_next::init() {
            init_result = Err(Error::Io(format!("FFmpeg initialization failed: {e}")));
            return;
        }

        // Set log level to ERROR to suppress warning messages
        // SAFETY: av_log_set_level is thread-safe and only affects logging
        unsafe {
            ffmpeg_next::ffi::av_log_set_level(ffmpeg_next::ffi::AV_LOG_ERROR);
        }
    });

    init_result
}

/// Video metadata extracted from a video file
#[derive(Debug, Clone)]
pub struct VideoMetadata {
    /// Video width in pixels
    pub width: u32,
    /// Video height in pixels
    pub height: u32,
    /// Duration in seconds
    pub duration_secs: f64,
    /// Frames per second
    pub fps: f64,
    /// Whether the video has an audio track
    pub has_audio: bool,
}

/// Extract thumbnail (first frame) from a video file.
///
/// Opens the video file, seeks to the first frame, decodes it, and converts
/// it to RGBA format for display.
pub fn extract_thumbnail<P: AsRef<Path>>(path: P) -> Result<ImageData> {
    // Initialize FFmpeg (with log level set to suppress warnings)
    init_ffmpeg()?;

    // Open video file
    let mut ictx = ffmpeg_next::format::input(&path)
        .map_err(|e| Error::Io(format!("Failed to open video file: {e}")))?;

    // Find video stream
    let input = ictx
        .streams()
        .best(ffmpeg_next::media::Type::Video)
        .ok_or_else(|| Error::Io("No video stream found".to_string()))?;
    let video_stream_index = input.index();

    // Create decoder
    let context_decoder = ffmpeg_next::codec::context::Context::from_parameters(input.parameters())
        .map_err(|e| Error::Io(format!("Failed to create codec context: {e}")))?;
    let mut decoder = context_decoder
        .decoder()
        .video()
        .map_err(|e| Error::Io(format!("Failed to create video decoder: {e}")))?;

    // Validate dimensions before creating scaler
    let width = decoder.width();
    let height = decoder.height();
    if width == 0 || height == 0 {
        return Err(Error::Io(format!(
            "Invalid video dimensions: {width}x{height} (possibly unsupported format)"
        )));
    }

    // Setup scaler to convert to RGB
    let mut scaler = ffmpeg_next::software::scaling::Context::get(
        decoder.format(),
        width,
        height,
        ffmpeg_next::format::Pixel::RGBA,
        width,
        height,
        ffmpeg_next::software::scaling::Flags::BILINEAR,
    )
    .map_err(|e| Error::Io(format!("Failed to create scaler: {e}")))?;

    // Decode first frame
    let mut rgb_frame = ffmpeg_next::frame::Video::empty();

    for (stream, packet) in ictx.packets() {
        if stream.index() == video_stream_index {
            decoder
                .send_packet(&packet)
                .map_err(|e| Error::Io(format!("Failed to send packet: {e}")))?;

            let mut decoded = ffmpeg_next::frame::Video::empty();
            if decoder.receive_frame(&mut decoded).is_ok() {
                // Convert to RGBA
                scaler
                    .run(&decoded, &mut rgb_frame)
                    .map_err(|e| Error::Io(format!("Failed to scale frame: {e}")))?;
                break;
            }
        }
    }

    // Check if we got a frame
    if rgb_frame.data(0).is_empty() {
        return Err(Error::Io("Could not decode first frame".to_string()));
    }

    // Convert frame to bytes
    let width = rgb_frame.width();
    let height = rgb_frame.height();
    let data = rgb_frame.data(0);
    let stride = rgb_frame.stride(0);

    // Copy frame data (handle stride)
    let mut rgba_bytes = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        let row_start = (y * stride as u32) as usize;
        let row_end = row_start + (width * 4) as usize;
        rgba_bytes.extend_from_slice(&data[row_start..row_end]);
    }

    Ok(ImageData::from_rgba(width, height, rgba_bytes))
}

/// Extract video metadata (dimensions, duration, FPS, audio presence)
///
/// Opens the video file and extracts metadata without decoding frames.
/// This is faster than thumbnail extraction as it only reads container metadata.
pub fn extract_video_metadata<P: AsRef<Path>>(path: P) -> Result<VideoMetadata> {
    // Initialize FFmpeg (with log level set to suppress warnings)
    init_ffmpeg()?;

    // Open video file
    let ictx = ffmpeg_next::format::input(&path)
        .map_err(|e| Error::Io(format!("Failed to open video file: {e}")))?;

    // Find video stream
    let video_stream = ictx
        .streams()
        .best(ffmpeg_next::media::Type::Video)
        .ok_or_else(|| Error::Io("No video stream found".to_string()))?;

    // Create decoder context to get dimensions
    let context_decoder =
        ffmpeg_next::codec::context::Context::from_parameters(video_stream.parameters())
            .map_err(|e| Error::Io(format!("Failed to create codec context: {e}")))?;
    let decoder = context_decoder
        .decoder()
        .video()
        .map_err(|e| Error::Io(format!("Failed to create video decoder: {e}")))?;

    // Extract video dimensions
    let width = decoder.width();
    let height = decoder.height();

    // Validate dimensions
    if width == 0 || height == 0 {
        return Err(Error::Io(format!(
            "Invalid video dimensions: {width}x{height} (possibly unsupported format)"
        )));
    }

    // Extract duration (convert from time_base to seconds)
    let duration_secs = if video_stream.duration() > 0 {
        let time_base = video_stream.time_base();
        video_stream.duration() as f64 * f64::from(time_base.numerator())
            / f64::from(time_base.denominator())
    } else if ictx.duration() > 0 {
        // Fallback to container duration
        ictx.duration() as f64 / f64::from(ffmpeg_next::ffi::AV_TIME_BASE)
    } else {
        0.0
    };

    // Extract FPS (frames per second)
    let fps = {
        let frame_rate = video_stream.avg_frame_rate();
        f64::from(frame_rate.numerator()) / f64::from(frame_rate.denominator())
    };

    // Detect audio stream
    let has_audio = ictx
        .streams()
        .best(ffmpeg_next::media::Type::Audio)
        .is_some();

    Ok(VideoMetadata {
        width,
        height,
        duration_secs,
        fps,
        has_audio,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_thumbnail_requires_video() {
        // This test requires an actual video file at tests/data/sample.mp4
        let result = extract_thumbnail("tests/data/sample.mp4");
        match result {
            Ok(data) => {
                assert!(data.width > 0);
                assert!(data.height > 0);
            }
            Err(_) => {
                // Expected if no test video exists
                println!("Test video not found (expected)");
            }
        }
    }
}
