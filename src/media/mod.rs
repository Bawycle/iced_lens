// SPDX-License-Identifier: MPL-2.0
//! Unified media handling for images and videos.
//!
//! This module provides a common interface for loading, displaying, and manipulating
//! both image and video files.

pub mod deblur;
pub mod filter;
pub mod frame_export;
pub mod image;
pub mod image_transform;
pub mod metadata;
pub mod metadata_operations;
pub mod metadata_writer;
pub mod navigator;
pub mod skip_attempts;
pub mod upscale;
pub mod video;
pub mod xmp;

use image_rs::AnimationDecoder;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

// Re-export commonly used types
pub use extensions::IMAGE_EXTENSIONS;
pub use filter::{DateFilterField, DateRangeFilter, MediaFilter, MediaTypeFilter};
pub use image::{load_image, ImageData};
pub use image_transform::ResizeScale;
pub use navigator::MediaNavigator;
pub use skip_attempts::MaxSkipAttempts;

/// Represents different types of media formats
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediaType {
    Image,
    Video,
}

/// Contains metadata and data for a loaded media file (image or video)
#[derive(Debug, Clone)]
pub enum MediaData {
    Image(ImageData),
    Video(VideoData),
}

/// Metadata and handles for video playback
#[derive(Debug, Clone)]
pub struct VideoData {
    /// Thumbnail image (first frame) for preview
    pub thumbnail: ImageData,
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

impl MediaData {
    /// Returns the media type (Image or Video)
    pub fn media_type(&self) -> MediaType {
        match self {
            MediaData::Image(_) => MediaType::Image,
            MediaData::Video(_) => MediaType::Video,
        }
    }

    /// Returns the width of the media
    pub fn width(&self) -> u32 {
        match self {
            MediaData::Image(data) => data.width,
            MediaData::Video(data) => data.width,
        }
    }

    /// Returns the height of the media
    pub fn height(&self) -> u32 {
        match self {
            MediaData::Image(data) => data.height,
            MediaData::Video(data) => data.height,
        }
    }
}

/// Supported media extensions
pub mod extensions {
    /// Image file extensions
    pub const IMAGE_EXTENSIONS: &[&str] = &[
        "jpg", "jpeg", "png", "gif", "tiff", "tif", "webp", "bmp", "ico", "svg",
    ];

    /// Video file extensions
    pub const VIDEO_EXTENSIONS: &[&str] = &["mp4", "m4v", "avi", "mov", "mkv", "webm"];

    /// All supported extensions (images + videos) for file dialogs
    pub const ALL_MEDIA_EXTENSIONS: &[&str] = &[
        "jpg", "jpeg", "png", "gif", "webp", "bmp", "tiff", "tif", "ico", "mp4", "avi", "mov",
        "mkv", "webm",
    ];

    /// Image format filters for save dialogs
    pub const IMAGE_SAVE_FILTERS: &[(&str, &[&str])] = &[
        ("JPEG", &["jpg", "jpeg"]),
        ("PNG", &["png"]),
        ("WebP", &["webp"]),
        ("TIFF", &["tiff", "tif"]),
    ];

    /// Extensions that support EXIF metadata writing.
    /// Includes formats from `IMAGE_EXTENSIONS` that support EXIF, plus additional formats.
    pub const EXIF_WRITE_EXTENSIONS: &[&str] = &[
        "jpg", "jpeg", "png", "webp", "tiff", "tif", "heic", "heif", "jxl", "avif",
    ];

    /// Extensions that support XMP metadata reading.
    pub const XMP_READ_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "tiff", "tif"];

    /// Extensions that support XMP metadata writing.
    pub const XMP_WRITE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "tiff", "tif"];

    /// All supported extensions (images + videos)
    #[must_use]
    pub fn all_supported_extensions() -> Vec<&'static str> {
        IMAGE_EXTENSIONS
            .iter()
            .chain(VIDEO_EXTENSIONS.iter())
            .copied()
            .collect()
    }

    /// Checks if a file extension supports XMP metadata reading.
    #[must_use]
    pub fn supports_xmp_read(ext: &str) -> bool {
        XMP_READ_EXTENSIONS.contains(&ext.to_lowercase().as_str())
    }

    /// Checks if a file extension supports XMP metadata writing.
    #[must_use]
    pub fn supports_xmp_write(ext: &str) -> bool {
        XMP_WRITE_EXTENSIONS.contains(&ext.to_lowercase().as_str())
    }

    /// Checks if a file path supports XMP metadata writing.
    #[must_use]
    pub fn path_supports_xmp_write<P: AsRef<std::path::Path>>(path: P) -> bool {
        path.as_ref()
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(supports_xmp_write)
    }
}

/// Counts the number of frames in a GIF file
fn count_gif_frames<P: AsRef<Path>>(path: P) -> crate::error::Result<usize> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let decoder = image_rs::codecs::gif::GifDecoder::new(reader)
        .map_err(|e| crate::error::Error::Io(e.to_string()))?;

    // Try to collect frames and count them
    let frames = decoder.into_frames();
    let count = frames.count();

    Ok(count)
}

/// Detects if a WebP file is animated by checking for ANMF chunk marker
///
/// Animated WebP files contain "ANMF" (Animation Frame) chunks in their structure.
/// This is a fast, reliable method that only reads the file header.
///
/// Reference: <https://stackoverflow.com/questions/45190469/how-to-identify-whether-webp-image-is-static-or-animated>
fn is_webp_animated_by_marker<P: AsRef<Path>>(path: P) -> crate::error::Result<bool> {
    let mut file = File::open(path)?;

    // Read first 1024 bytes (sufficient to find ANMF marker)
    // WebP header structure places animation info early in the file
    let mut buffer = vec![0u8; 1024];
    let bytes_read = file.read(&mut buffer)?;
    buffer.truncate(bytes_read);

    // Look for "ANMF" (Animation Frame chunk) marker
    // This is present in all animated WebP files
    let has_anmf = buffer.windows(4).any(|window| window == b"ANMF");

    Ok(has_anmf)
}

/// Counts the number of frames in a WebP file
///
/// For animated WebP, this uses marker detection to determine if animated.
/// Actual frame counting would require full decoding, so we return:
/// - 1 for static WebP (no ANMF marker)
/// - 2 for animated WebP (has ANMF marker, indicating >1 frame)
fn count_webp_frames<P: AsRef<Path>>(path: P) -> crate::error::Result<usize> {
    if is_webp_animated_by_marker(&path)? {
        Ok(2) // Animated (>1 frame)
    } else {
        Ok(1) // Static (1 frame)
    }
}

/// Detects if a GIF or WebP file is animated (has multiple frames)
fn is_animated<P: AsRef<Path>>(path: P) -> crate::error::Result<bool> {
    let path_ref = path.as_ref();
    let extension = path_ref
        .extension()
        .and_then(|s| s.to_str())
        .map(str::to_lowercase)
        .unwrap_or_default();

    match extension.as_str() {
        "gif" => {
            let count = count_gif_frames(path)?;
            Ok(count > 1)
        }
        "webp" => {
            let count = count_webp_frames(path)?;
            Ok(count > 1)
        }
        _ => Ok(false),
    }
}

/// Load media file (image or video) and return unified `MediaData`
///
/// Automatically detects the media type and loads it appropriately:
/// - Images are loaded directly using `load_image()`
/// - Videos are loaded as `VideoData` with thumbnail and metadata
/// - Animated WebP files use dedicated webp-animation decoder (`FFmpeg` doesn't support them well)
///
/// # Errors
/// Returns an error if:
/// - The file format is not supported
/// - The file cannot be read or decoded
pub fn load_media<P: AsRef<Path>>(path: P) -> crate::error::Result<MediaData> {
    let path_ref = path.as_ref();

    // Detect media type
    let media_type = detect_media_type(path_ref)
        .ok_or_else(|| crate::error::Error::Io("Unsupported file format".to_string()))?;

    match media_type {
        MediaType::Image => {
            // Load as image
            let image_data = image::load_image(path_ref)?;
            Ok(MediaData::Image(image_data))
        }
        MediaType::Video => {
            // Check if this is an animated WebP (requires special handling)
            let extension = path_ref
                .extension()
                .and_then(|s| s.to_str())
                .map(str::to_lowercase)
                .unwrap_or_default();

            if extension == "webp" {
                // Use dedicated WebP decoder for animated WebP files
                // FFmpeg doesn't support animated WebP well
                return load_animated_webp(path_ref);
            }

            // Try to load as video using FFmpeg
            match (
                video::extract_thumbnail(path_ref),
                video::extract_video_metadata(path_ref),
            ) {
                (Ok(thumbnail), Ok(metadata)) => {
                    let video_data = VideoData {
                        thumbnail,
                        width: metadata.width,
                        height: metadata.height,
                        duration_secs: metadata.duration_secs,
                        fps: metadata.fps,
                        has_audio: metadata.has_audio,
                    };
                    Ok(MediaData::Video(video_data))
                }
                (Err(e), _) | (_, Err(e)) => {
                    // FFmpeg failed - return error for regular videos
                    Err(e)
                }
            }
        }
    }
}

/// Load an animated WebP file using the dedicated webp-animation decoder.
fn load_animated_webp(path: &Path) -> crate::error::Result<MediaData> {
    use crate::video_player::WebpAnimDecoder;

    // Get metadata using our WebP decoder
    let metadata = WebpAnimDecoder::get_metadata(path)?;

    // Extract first frame as thumbnail
    let webp_data = std::fs::read(path)
        .map_err(|e| crate::error::Error::Io(format!("Failed to read WebP file: {e}")))?;

    let decoder = webp_animation::Decoder::new(&webp_data)
        .map_err(|e| crate::error::Error::Io(format!("Failed to decode WebP: {e:?}")))?;

    // Get first frame as thumbnail
    let first_frame = decoder
        .into_iter()
        .next()
        .ok_or_else(|| crate::error::Error::Io("No frames found in animated WebP".to_string()))?;

    let (width, height) = first_frame.dimensions();
    let rgba_data = first_frame.data().to_vec();

    // Create thumbnail ImageData
    let thumbnail = ImageData::from_rgba(width, height, rgba_data);

    let video_data = VideoData {
        thumbnail,
        width: metadata.width,
        height: metadata.height,
        duration_secs: metadata.duration_secs,
        fps: metadata.fps,
        has_audio: false, // WebP animations don't have audio
    };

    Ok(MediaData::Video(video_data))
}

/// Detects the media type from file extension with dynamic detection for GIF/WebP
pub fn detect_media_type<P: AsRef<Path>>(path: P) -> Option<MediaType> {
    let path_ref = path.as_ref();
    let extension = path_ref
        .extension()
        .and_then(|s| s.to_str())
        .map(str::to_lowercase)?;

    // For GIF and WebP, check if animated
    if extension == "gif" || extension == "webp" {
        match is_animated(path_ref) {
            Ok(true) => return Some(MediaType::Video), // Animated
            Ok(false) | Err(_) => return Some(MediaType::Image), // Static or error
        }
    }

    // Static detection for other formats
    if extensions::IMAGE_EXTENSIONS.contains(&extension.as_str()) {
        Some(MediaType::Image)
    } else if extensions::VIDEO_EXTENSIONS.contains(&extension.as_str()) {
        Some(MediaType::Video)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_detect_image_formats() {
        assert_eq!(detect_media_type("photo.jpg"), Some(MediaType::Image));
        assert_eq!(detect_media_type("image.PNG"), Some(MediaType::Image));
        assert_eq!(detect_media_type("graphic.svg"), Some(MediaType::Image));
    }

    #[test]
    fn test_detect_video_formats() {
        assert_eq!(detect_media_type("video.mp4"), Some(MediaType::Video));
        assert_eq!(detect_media_type("movie.AVI"), Some(MediaType::Video));
        assert_eq!(detect_media_type("clip.mkv"), Some(MediaType::Video));
    }

    #[test]
    fn test_detect_unsupported_format() {
        assert_eq!(detect_media_type("document.pdf"), None);
        assert_eq!(detect_media_type("archive.zip"), None);
    }

    #[test]
    fn test_case_insensitivity() {
        assert_eq!(detect_media_type("VIDEO.MP4"), Some(MediaType::Video));
        assert_eq!(detect_media_type("Image.JpEg"), Some(MediaType::Image));
    }

    #[test]
    fn test_path_with_directories() {
        let path = PathBuf::from("/home/user/videos/vacation.mp4");
        assert_eq!(detect_media_type(&path), Some(MediaType::Video));
    }

    #[test]
    fn test_all_extensions_unique() {
        let all = extensions::all_supported_extensions();
        let unique_count = all.iter().collect::<std::collections::HashSet<_>>().len();
        assert_eq!(all.len(), unique_count, "Extensions should be unique");
    }

    #[test]
    fn test_media_data_accessors() {
        // Create mock image data with 1920x1080 pixels
        let pixels = vec![0_u8; 1920 * 1080 * 4];
        let img_data = ImageData::from_rgba(1920, 1080, pixels);

        let media = MediaData::Image(img_data);

        assert_eq!(media.media_type(), MediaType::Image);
        assert_eq!(media.width(), 1920);
        assert_eq!(media.height(), 1080);
    }

    #[test]
    fn test_detect_static_gif() {
        // This test requires tests/data/test_static.gif
        let path = "tests/data/test_static.gif";
        if std::path::Path::new(path).exists() {
            assert_eq!(detect_media_type(path), Some(MediaType::Image));
        }
    }

    #[test]
    fn test_detect_animated_gif() {
        // This test requires tests/data/test_animated.gif
        let path = "tests/data/test_animated.gif";
        if std::path::Path::new(path).exists() {
            assert_eq!(detect_media_type(path), Some(MediaType::Video));
        }
    }

    #[test]
    fn test_detect_static_webp() {
        // This test requires tests/data/test_static.webp
        let path = "tests/data/test_static.webp";
        if std::path::Path::new(path).exists() {
            assert_eq!(detect_media_type(path), Some(MediaType::Image));
        }
    }

    #[test]
    fn test_detect_animated_webp() {
        // This test requires tests/data/test_animated.webp
        let path = "tests/data/test_animated.webp";
        if std::path::Path::new(path).exists() {
            assert_eq!(detect_media_type(path), Some(MediaType::Video));
        }
    }

    #[test]
    fn test_webp_anmf_marker_detection() {
        // Test ANMF marker detection directly
        let animated_path = "tests/data/test_animated.webp";
        let static_path = "tests/data/test_static.webp";

        if std::path::Path::new(animated_path).exists() {
            let is_animated = is_webp_animated_by_marker(animated_path).unwrap();
            assert!(is_animated, "Animated WebP should have ANMF marker");
        }

        if std::path::Path::new(static_path).exists() {
            let is_animated = is_webp_animated_by_marker(static_path).unwrap();
            assert!(!is_animated, "Static WebP should not have ANMF marker");
        }
    }

    #[test]
    fn test_count_gif_frames_static() {
        let path = "tests/data/test_static.gif";
        if std::path::Path::new(path).exists() {
            let count = count_gif_frames(path).unwrap();
            assert_eq!(count, 1);
        }
    }

    #[test]
    fn test_count_gif_frames_animated() {
        let path = "tests/data/test_animated.gif";
        if std::path::Path::new(path).exists() {
            let count = count_gif_frames(path).unwrap();
            assert!(count > 1, "Animated GIF should have more than 1 frame");
        }
    }

    #[test]
    fn test_load_media_image() {
        let path = "tests/data/sample.png";
        if !std::path::Path::new(path).exists() {
            return;
        }

        let result = super::load_media(path);
        assert!(result.is_ok(), "Should load image successfully");

        let media = result.unwrap();
        assert_eq!(media.media_type(), MediaType::Image);
        assert!(media.width() > 0);
        assert!(media.height() > 0);
    }

    #[test]
    fn test_load_media_video() {
        let path = "tests/data/sample.mp4";
        if !std::path::Path::new(path).exists() {
            return;
        }

        let result = super::load_media(path);
        assert!(result.is_ok(), "Should load video successfully");

        let media = result.unwrap();
        assert_eq!(media.media_type(), MediaType::Video);
        assert!(media.width() > 0);
        assert!(media.height() > 0);

        // Verify it's VideoData with metadata
        if let MediaData::Video(video_data) = media {
            assert!(video_data.duration_secs > 0.0);
            assert!(video_data.fps > 0.0);
        } else {
            panic!("Expected VideoData");
        }
    }

    #[test]
    fn test_load_media_unsupported_format() {
        let path = "tests/data/document.pdf";
        let result = super::load_media(path);
        assert!(result.is_err(), "Should fail on unsupported format");
    }

    #[test]
    fn test_supports_xmp_read() {
        assert!(extensions::supports_xmp_read("jpg"));
        assert!(extensions::supports_xmp_read("JPEG"));
        assert!(extensions::supports_xmp_read("png"));
        assert!(extensions::supports_xmp_read("PNG"));
        assert!(extensions::supports_xmp_read("webp"));
        assert!(extensions::supports_xmp_read("tiff"));
        assert!(extensions::supports_xmp_read("tif"));
        assert!(!extensions::supports_xmp_read("gif"));
        assert!(!extensions::supports_xmp_read("bmp"));
    }

    #[test]
    fn test_supports_xmp_write() {
        assert!(extensions::supports_xmp_write("jpg"));
        assert!(extensions::supports_xmp_write("JPEG"));
        assert!(extensions::supports_xmp_write("png"));
        assert!(extensions::supports_xmp_write("PNG"));
        assert!(extensions::supports_xmp_write("webp"));
        assert!(extensions::supports_xmp_write("tiff"));
        assert!(extensions::supports_xmp_write("tif"));
        assert!(!extensions::supports_xmp_write("gif"));
        assert!(!extensions::supports_xmp_write("bmp"));
    }

    #[test]
    fn test_path_supports_xmp_write() {
        assert!(extensions::path_supports_xmp_write("photo.jpg"));
        assert!(extensions::path_supports_xmp_write("image.PNG"));
        assert!(extensions::path_supports_xmp_write("/path/to/file.jpeg"));
        assert!(extensions::path_supports_xmp_write("image.webp"));
        assert!(extensions::path_supports_xmp_write("photo.tiff"));
        assert!(extensions::path_supports_xmp_write("photo.tif"));
        assert!(!extensions::path_supports_xmp_write("image.gif"));
        assert!(!extensions::path_supports_xmp_write("no_extension"));
    }
}
