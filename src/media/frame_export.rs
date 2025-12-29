// SPDX-License-Identifier: MPL-2.0
//! Frame export functionality for video playback.
//!
//! This module provides functions to export video frames to various image formats
//! (PNG, JPEG, WebP) using the `image` crate.

use crate::error::{Error, Result};
use image_rs::{ImageBuffer, ImageFormat, Rgba};
use std::path::Path;
use std::sync::Arc;

/// Supported export formats for frame capture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExportFormat {
    /// PNG format (lossless, best quality).
    #[default]
    Png,
    /// JPEG format (lossy, smaller file size).
    Jpeg,
    /// WebP format (modern, good compression).
    WebP,
}

impl ExportFormat {
    /// Returns the file extension for this format.
    #[must_use] 
    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Png => "png",
            ExportFormat::Jpeg => "jpg",
            ExportFormat::WebP => "webp",
        }
    }

    /// Returns the image format for the `image` crate.
    fn image_format(self) -> ImageFormat {
        match self {
            ExportFormat::Png => ImageFormat::Png,
            ExportFormat::Jpeg => ImageFormat::Jpeg,
            ExportFormat::WebP => ImageFormat::WebP,
        }
    }

    /// Returns a human-readable description.
    #[must_use] 
    pub fn description(&self) -> &'static str {
        match self {
            ExportFormat::Png => "PNG (Lossless)",
            ExportFormat::Jpeg => "JPEG (Lossy)",
            ExportFormat::WebP => "WebP (Modern)",
        }
    }

    /// Returns all supported formats.
    #[must_use] 
    pub fn all() -> &'static [ExportFormat] {
        &[ExportFormat::Png, ExportFormat::Jpeg, ExportFormat::WebP]
    }

    /// Detects format from file extension.
    #[must_use] 
    pub fn from_extension(ext: &str) -> Option<ExportFormat> {
        match ext.to_lowercase().as_str() {
            "png" => Some(ExportFormat::Png),
            "jpg" | "jpeg" => Some(ExportFormat::Jpeg),
            "webp" => Some(ExportFormat::WebP),
            _ => None,
        }
    }

    /// Detects format from file path extension.
    pub fn from_path(path: &Path) -> Option<ExportFormat> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(Self::from_extension)
    }
}

/// Data for a frame ready to be exported.
///
/// Uses `Arc<Vec<u8>>` to avoid expensive clones when passing frame data around.
/// The actual data is only cloned when necessary (e.g., for `ImageBuffer::from_raw`).
#[derive(Debug, Clone, PartialEq)]
pub struct ExportableFrame {
    /// RGBA pixel data (shared reference to avoid expensive clones).
    pub rgba_data: Arc<Vec<u8>>,
    /// Frame width in pixels.
    pub width: u32,
    /// Frame height in pixels.
    pub height: u32,
}

impl ExportableFrame {
    /// Creates a new exportable frame from RGBA data.
    #[must_use] 
    pub fn new(rgba_data: Arc<Vec<u8>>, width: u32, height: u32) -> Self {
        Self {
            rgba_data,
            width,
            height,
        }
    }

    /// Converts to a `DynamicImage` for use with image editing.
    ///
    /// Note: This clones the underlying pixel data since `ImageBuffer::from_raw`
    /// requires ownership of the data.
    pub fn to_dynamic_image(&self) -> Option<image_rs::DynamicImage> {
        ImageBuffer::<Rgba<u8>, _>::from_raw(self.width, self.height, (*self.rgba_data).clone())
            .map(image_rs::DynamicImage::ImageRgba8)
    }

    /// Converts to `ImageData` for display in the UI.
    ///
    /// Note: This clones the underlying pixel data since `from_rgba`
    /// requires ownership of the data.
    #[must_use] 
    pub fn to_image_data(&self) -> crate::media::ImageData {
        crate::media::ImageData::from_rgba(self.width, self.height, (*self.rgba_data).clone())
    }

    /// Exports the frame to a file.
    ///
    /// The format is determined by the file extension if not specified.
    ///
    /// Note: This clones the underlying pixel data since `ImageBuffer::from_raw`
    /// requires ownership of the data.
    ///
    /// # Errors
    ///
    /// Returns an error if the image cannot be encoded or written to disk.
    pub fn save_to_file<P: AsRef<Path>>(
        &self,
        path: P,
        format: Option<ExportFormat>,
    ) -> Result<()> {
        let path = path.as_ref();

        // Determine format from extension if not specified
        let format = format.unwrap_or_else(|| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .and_then(ExportFormat::from_extension)
                .unwrap_or_default()
        });

        // Create image buffer from RGBA data (requires cloning the Arc's contents)
        let img: ImageBuffer<Rgba<u8>, _> =
            ImageBuffer::from_raw(self.width, self.height, (*self.rgba_data).clone()).ok_or_else(
                || Error::Io("Failed to create image buffer from frame data".to_string()),
            )?;

        // For JPEG, convert to RGB (JPEG doesn't support alpha)
        if format == ExportFormat::Jpeg {
            let rgb_img = image_rs::DynamicImage::ImageRgba8(img).to_rgb8();
            rgb_img
                .save_with_format(path, format.image_format())
                .map_err(|e| Error::Io(format!("Failed to save frame: {e}")))?;
        } else {
            img.save_with_format(path, format.image_format())
                .map_err(|e| Error::Io(format!("Failed to save frame: {e}")))?;
        }

        Ok(())
    }
}

/// Generates a default filename for frame export.
///
/// Format: `{video_name}_frame_{position}.{ext}`
#[must_use] 
pub fn generate_default_filename(
    video_path: &Path,
    position_secs: f64,
    format: ExportFormat,
) -> String {
    let video_name = video_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("video");

    // Format position as MM-SS-mmm (minutes-seconds-milliseconds)
    // Video positions are practically bounded (years of video fit in u64 ms), so cast is safe
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let total_ms = (position_secs.max(0.0) * 1000.0).round() as u64;
    let minutes = total_ms / 60000;
    let seconds = (total_ms % 60000) / 1000;
    let millis = total_ms % 1000;

    format!(
        "{}_frame_{:02}-{:02}-{:03}.{}",
        video_name,
        minutes,
        seconds,
        millis,
        format.extension()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn export_format_extensions() {
        assert_eq!(ExportFormat::Png.extension(), "png");
        assert_eq!(ExportFormat::Jpeg.extension(), "jpg");
        assert_eq!(ExportFormat::WebP.extension(), "webp");
    }

    #[test]
    fn export_format_from_extension() {
        assert_eq!(ExportFormat::from_extension("png"), Some(ExportFormat::Png));
        assert_eq!(ExportFormat::from_extension("PNG"), Some(ExportFormat::Png));
        assert_eq!(
            ExportFormat::from_extension("jpg"),
            Some(ExportFormat::Jpeg)
        );
        assert_eq!(
            ExportFormat::from_extension("jpeg"),
            Some(ExportFormat::Jpeg)
        );
        assert_eq!(
            ExportFormat::from_extension("webp"),
            Some(ExportFormat::WebP)
        );
        assert_eq!(ExportFormat::from_extension("bmp"), None);
    }

    #[test]
    fn export_format_all_returns_three_formats() {
        assert_eq!(ExportFormat::all().len(), 3);
    }

    #[test]
    fn generate_default_filename_formats_correctly() {
        let path = PathBuf::from("/videos/my_video.mp4");
        let filename = generate_default_filename(&path, 125.456, ExportFormat::Png);
        assert_eq!(filename, "my_video_frame_02-05-456.png");
    }

    #[test]
    fn generate_default_filename_handles_zero() {
        let path = PathBuf::from("video.mkv");
        let filename = generate_default_filename(&path, 0.0, ExportFormat::Jpeg);
        assert_eq!(filename, "video_frame_00-00-000.jpg");
    }

    #[test]
    fn exportable_frame_can_be_created() {
        let rgba = Arc::new(vec![255u8; 4 * 10 * 10]); // 10x10 white image
        let frame = ExportableFrame::new(rgba, 10, 10);
        assert_eq!(frame.width, 10);
        assert_eq!(frame.height, 10);
        assert_eq!(frame.rgba_data.len(), 400);
    }

    #[test]
    fn export_format_default_is_png() {
        assert_eq!(ExportFormat::default(), ExportFormat::Png);
    }
}
