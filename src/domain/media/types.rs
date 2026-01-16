// SPDX-License-Identifier: MPL-2.0
//! Core media types for the domain layer.
//!
//! These types represent pure data without any presentation dependencies.

use std::sync::Arc;

/// Represents different types of media formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaType {
    /// Static image (JPEG, PNG, BMP, etc.)
    Image,
    /// Video or animated image (MP4, GIF, animated WebP, etc.)
    Video,
}

/// Raw image data without presentation dependencies.
///
/// This is the domain representation of an image, containing only the
/// pure pixel data. Presentation layer converts this to framework-specific
/// handles (e.g., `iced::widget::image::Handle`).
///
/// # Example
///
/// ```
/// use iced_lens::domain::media::RawImage;
/// use std::sync::Arc;
///
/// let pixels = vec![255u8; 100 * 100 * 4]; // 100x100 RGBA
/// let image = RawImage::new(100, 100, Arc::new(pixels));
///
/// assert_eq!(image.width(), 100);
/// assert_eq!(image.height(), 100);
/// ```
#[derive(Debug, Clone)]
pub struct RawImage {
    /// Image width in pixels.
    width: u32,
    /// Image height in pixels.
    height: u32,
    /// RGBA pixel data (4 bytes per pixel).
    rgba_bytes: Arc<Vec<u8>>,
}

impl RawImage {
    /// Creates a new `RawImage` from dimensions and RGBA pixel data.
    ///
    /// # Panics
    ///
    /// Panics if the pixel data length doesn't match `width * height * 4`.
    #[must_use]
    pub fn new(width: u32, height: u32, rgba_bytes: Arc<Vec<u8>>) -> Self {
        let expected_len = (width as usize) * (height as usize) * 4;
        assert_eq!(
            rgba_bytes.len(),
            expected_len,
            "RGBA data length mismatch: expected {expected_len}, got {}",
            rgba_bytes.len()
        );

        Self {
            width,
            height,
            rgba_bytes,
        }
    }

    /// Creates a new `RawImage` from dimensions and owned RGBA pixel data.
    ///
    /// # Panics
    ///
    /// Panics if the pixel data length doesn't match `width * height * 4`.
    #[must_use]
    pub fn from_rgba(width: u32, height: u32, rgba_bytes: Vec<u8>) -> Self {
        Self::new(width, height, Arc::new(rgba_bytes))
    }

    /// Returns the image width in pixels.
    #[must_use]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Returns the image height in pixels.
    #[must_use]
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Returns a reference to the RGBA pixel data.
    #[must_use]
    pub fn rgba_bytes(&self) -> &[u8] {
        &self.rgba_bytes
    }

    /// Returns the shared reference to the RGBA pixel data.
    #[must_use]
    pub fn rgba_bytes_arc(&self) -> Arc<Vec<u8>> {
        Arc::clone(&self.rgba_bytes)
    }

    /// Returns the total number of pixels.
    #[must_use]
    pub fn pixel_count(&self) -> usize {
        (self.width as usize) * (self.height as usize)
    }
}

impl PartialEq for RawImage {
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width
            && self.height == other.height
            && self.rgba_bytes == other.rgba_bytes
    }
}

impl Eq for RawImage {}

/// Video metadata without presentation dependencies.
///
/// This contains only the pure metadata about a video file.
/// The thumbnail (which requires a presentation Handle) is managed
/// separately in the presentation layer.
#[derive(Debug, Clone, PartialEq)]
pub struct VideoMetadata {
    /// Video width in pixels.
    pub width: u32,
    /// Video height in pixels.
    pub height: u32,
    /// Duration in seconds.
    pub duration_secs: f64,
    /// Frames per second.
    pub fps: f64,
    /// Whether the video has an audio track.
    pub has_audio: bool,
}

impl VideoMetadata {
    /// Creates a new `VideoMetadata`.
    #[must_use]
    pub fn new(width: u32, height: u32, duration_secs: f64, fps: f64, has_audio: bool) -> Self {
        Self {
            width,
            height,
            duration_secs,
            fps,
            has_audio,
        }
    }

    /// Returns the total number of frames (approximate).
    ///
    /// Returns 0 for negative durations or frame rates.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn frame_count(&self) -> u64 {
        let frames = self.duration_secs * self.fps;
        if frames < 0.0 {
            0
        } else {
            frames.round() as u64
        }
    }

    /// Returns the aspect ratio (width / height).
    #[must_use]
    pub fn aspect_ratio(&self) -> f64 {
        if self.height == 0 {
            1.0
        } else {
            f64::from(self.width) / f64::from(self.height)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_type_equality() {
        assert_eq!(MediaType::Image, MediaType::Image);
        assert_eq!(MediaType::Video, MediaType::Video);
        assert_ne!(MediaType::Image, MediaType::Video);
    }

    #[test]
    fn test_raw_image_creation() {
        let pixels = vec![0u8; 10 * 10 * 4];
        let image = RawImage::from_rgba(10, 10, pixels);

        assert_eq!(image.width(), 10);
        assert_eq!(image.height(), 10);
        assert_eq!(image.pixel_count(), 100);
        assert_eq!(image.rgba_bytes().len(), 400);
    }

    #[test]
    fn test_raw_image_with_arc() {
        let pixels = Arc::new(vec![255u8; 5 * 5 * 4]);
        let image = RawImage::new(5, 5, pixels);

        assert_eq!(image.width(), 5);
        assert_eq!(image.height(), 5);
    }

    #[test]
    #[should_panic(expected = "RGBA data length mismatch")]
    fn test_raw_image_invalid_size() {
        let pixels = vec![0u8; 100]; // Wrong size
        let _ = RawImage::from_rgba(10, 10, pixels);
    }

    #[test]
    fn test_raw_image_equality() {
        let pixels1 = vec![0u8; 10 * 10 * 4];
        let pixels2 = vec![0u8; 10 * 10 * 4];
        let pixels3 = vec![1u8; 10 * 10 * 4];

        let image1 = RawImage::from_rgba(10, 10, pixels1);
        let image2 = RawImage::from_rgba(10, 10, pixels2);
        let image3 = RawImage::from_rgba(10, 10, pixels3);

        assert_eq!(image1, image2);
        assert_ne!(image1, image3);
    }

    #[test]
    fn test_video_metadata_creation() {
        let metadata = VideoMetadata::new(1920, 1080, 120.5, 30.0, true);

        assert_eq!(metadata.width, 1920);
        assert_eq!(metadata.height, 1080);
        assert!((metadata.duration_secs - 120.5).abs() < f64::EPSILON);
        assert!((metadata.fps - 30.0).abs() < f64::EPSILON);
        assert!(metadata.has_audio);
    }

    #[test]
    fn test_video_metadata_frame_count() {
        let metadata = VideoMetadata::new(1920, 1080, 10.0, 30.0, true);
        assert_eq!(metadata.frame_count(), 300);

        let metadata2 = VideoMetadata::new(1920, 1080, 10.5, 24.0, false);
        assert_eq!(metadata2.frame_count(), 252);
    }

    #[test]
    fn test_video_metadata_aspect_ratio() {
        let metadata_16_9 = VideoMetadata::new(1920, 1080, 10.0, 30.0, true);
        let expected = 1920.0 / 1080.0;
        assert!((metadata_16_9.aspect_ratio() - expected).abs() < 0.001);

        let metadata_4_3 = VideoMetadata::new(640, 480, 10.0, 30.0, true);
        let expected_4_3 = 640.0 / 480.0;
        assert!((metadata_4_3.aspect_ratio() - expected_4_3).abs() < 0.001);
    }

    #[test]
    fn test_video_metadata_zero_height() {
        let metadata = VideoMetadata::new(1920, 0, 10.0, 30.0, true);
        assert!((metadata.aspect_ratio() - 1.0).abs() < f64::EPSILON);
    }
}
