// SPDX-License-Identifier: MPL-2.0
//! Media loading port definition.
//!
//! This module defines the [`MediaLoader`] trait for loading media files.
//! Infrastructure adapters implement this trait to provide concrete loading logic.
//!
//! # Implementation Note
//!
//! This trait is defined but **not implemented** in this migration.
//! Media loading continues to use the existing code in `media/mod.rs`.
//! A concrete `FsMediaLoader` adapter will be added in a future migration
//! when filesystem abstraction is needed.

use crate::domain::media::{MediaType, RawImage, VideoMetadata};
use std::fmt;
use std::path::Path;
use std::time::Duration;

// =============================================================================
// MediaError
// =============================================================================

/// Errors that can occur during media loading.
#[derive(Debug, Clone)]
pub enum MediaError {
    /// The file format is not supported.
    UnsupportedFormat,

    /// The image dimensions are invalid (zero or negative).
    InvalidDimensions {
        /// The width that was detected.
        width: u32,
        /// The height that was detected.
        height: u32,
    },

    /// The media data is corrupted or cannot be decoded.
    CorruptedData(String),

    /// The file could not be read (I/O error).
    IoError(String),

    /// The file was not found.
    NotFound,
}

impl fmt::Display for MediaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MediaError::UnsupportedFormat => write!(f, "Unsupported media format"),
            MediaError::InvalidDimensions { width, height } => {
                write!(f, "Invalid dimensions: {width}x{height}")
            }
            MediaError::CorruptedData(msg) => write!(f, "Corrupted media data: {msg}"),
            MediaError::IoError(msg) => write!(f, "I/O error: {msg}"),
            MediaError::NotFound => write!(f, "File not found"),
        }
    }
}

impl std::error::Error for MediaError {}

// =============================================================================
// LoadedMedia
// =============================================================================

/// Result of successfully loading a media file.
///
/// This enum distinguishes between the different types of media that can be loaded.
#[derive(Debug, Clone)]
pub enum LoadedMedia {
    /// A static image (JPEG, PNG, BMP, etc.).
    Image(RawImage),

    /// A video file with metadata.
    Video(VideoMetadata),

    /// An animated image (GIF, animated WebP, etc.).
    AnimatedImage {
        /// Individual frames of the animation.
        frames: Vec<RawImage>,
        /// Duration of each frame.
        durations: Vec<Duration>,
    },
}

impl LoadedMedia {
    /// Returns the media type of this loaded media.
    #[must_use]
    pub fn media_type(&self) -> MediaType {
        match self {
            LoadedMedia::Image(_) => MediaType::Image,
            // Animated images are treated as video-like (both have temporal playback)
            LoadedMedia::Video(_) | LoadedMedia::AnimatedImage { .. } => MediaType::Video,
        }
    }

    /// Returns `true` if this is a static image.
    #[must_use]
    pub fn is_image(&self) -> bool {
        matches!(self, LoadedMedia::Image(_))
    }

    /// Returns `true` if this is a video.
    #[must_use]
    pub fn is_video(&self) -> bool {
        matches!(self, LoadedMedia::Video(_))
    }

    /// Returns `true` if this is an animated image.
    #[must_use]
    pub fn is_animated(&self) -> bool {
        matches!(self, LoadedMedia::AnimatedImage { .. })
    }
}

// =============================================================================
// MediaLoader Trait
// =============================================================================

/// Port for loading media files.
///
/// This trait defines the interface for loading media from the filesystem.
/// Infrastructure adapters implement this trait to provide concrete loading logic.
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` for use across threads.
///
/// # Example
///
/// ```ignore
/// use iced_lens::application::port::media::{MediaLoader, LoadedMedia};
/// use std::path::Path;
///
/// fn load_and_display(loader: &impl MediaLoader, path: &Path) {
///     if loader.supports(path) {
///         match loader.load(path) {
///             Ok(LoadedMedia::Image(raw)) => {
///                 println!("Loaded image: {}x{}", raw.width(), raw.height());
///             }
///             Ok(LoadedMedia::Video(meta)) => {
///                 println!("Loaded video: {:.1}s duration", meta.duration_secs);
///             }
///             Ok(LoadedMedia::AnimatedImage { frames, .. }) => {
///                 println!("Loaded animation: {} frames", frames.len());
///             }
///             Err(e) => eprintln!("Failed to load: {e}"),
///         }
///     }
/// }
/// ```
pub trait MediaLoader: Send + Sync {
    /// Loads media from a file path.
    ///
    /// Returns the loaded media data (image, video, or animated image).
    ///
    /// # Errors
    ///
    /// Returns a [`MediaError`] if:
    /// - The file cannot be read
    /// - The format is not supported
    /// - The data is corrupted
    fn load(&self, path: &Path) -> Result<LoadedMedia, MediaError>;

    /// Checks if a file path is a supported media format.
    ///
    /// This performs a quick check based on file extension only,
    /// without reading the file contents.
    fn supports(&self, path: &Path) -> bool;

    /// Probes a file to determine its media type without fully loading it.
    ///
    /// This is faster than `load()` when only the type is needed.
    ///
    /// # Errors
    ///
    /// Returns a [`MediaError`] if the file cannot be probed.
    fn probe(&self, path: &Path) -> Result<MediaType, MediaError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_error_display() {
        let err = MediaError::UnsupportedFormat;
        assert_eq!(format!("{err}"), "Unsupported media format");

        let err = MediaError::InvalidDimensions {
            width: 0,
            height: 100,
        };
        assert!(format!("{err}").contains("0x100"));

        let err = MediaError::CorruptedData("bad header".to_string());
        assert!(format!("{err}").contains("bad header"));

        let err = MediaError::IoError("permission denied".to_string());
        assert!(format!("{err}").contains("permission denied"));

        let err = MediaError::NotFound;
        assert!(format!("{err}").contains("not found"));
    }

    #[test]
    fn loaded_media_type_detection() {
        use std::sync::Arc;

        // Image
        let raw = RawImage::new(10, 10, Arc::new(vec![0u8; 400]));
        let media = LoadedMedia::Image(raw);
        assert_eq!(media.media_type(), MediaType::Image);
        assert!(media.is_image());
        assert!(!media.is_video());
        assert!(!media.is_animated());

        // Video
        let meta = VideoMetadata::new(1920, 1080, 120.0, 30.0, true);
        let media = LoadedMedia::Video(meta);
        assert_eq!(media.media_type(), MediaType::Video);
        assert!(!media.is_image());
        assert!(media.is_video());
        assert!(!media.is_animated());

        // Animated
        let media = LoadedMedia::AnimatedImage {
            frames: vec![],
            durations: vec![],
        };
        assert_eq!(media.media_type(), MediaType::Video);
        assert!(!media.is_image());
        assert!(!media.is_video());
        assert!(media.is_animated());
    }
}
