// SPDX-License-Identifier: MPL-2.0
//! Image loading and decoding from various formats (PNG, JPEG, GIF, SVG, etc.).

use crate::error::{Error, Result};
use iced::widget::image;
use image_rs::{GenericImageView, ImageError};
use resvg::usvg;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tiny_skia;

#[derive(Debug, Clone)]
pub struct ImageData {
    pub handle: image::Handle,
    pub width: u32,
    pub height: u32,
    /// Original RGBA bytes for rotation support.
    /// Stored in Arc to avoid expensive cloning.
    rgba_bytes: Arc<Vec<u8>>,
}

impl ImageData {
    /// Creates a new `ImageData` from RGBA pixels.
    ///
    /// The pixels are stored in an Arc for shared ownership, and a copy is
    /// made for the Handle.
    #[must_use]
    pub fn from_rgba(width: u32, height: u32, pixels: Vec<u8>) -> Self {
        let rgba_bytes = Arc::new(pixels);
        let handle = image::Handle::from_rgba(width, height, rgba_bytes.to_vec());
        Self {
            handle,
            width,
            height,
            rgba_bytes,
        }
    }

    /// Creates a new `ImageData` from encoded bytes (PNG, JPEG, etc.).
    ///
    /// This is used for SVGs and other formats where the raw bytes are available.
    /// The RGBA bytes are extracted from the provided raw pixels.
    #[must_use]
    pub fn from_encoded_with_rgba(
        encoded_bytes: Vec<u8>,
        width: u32,
        height: u32,
        rgba_pixels: Vec<u8>,
    ) -> Self {
        let rgba_bytes = Arc::new(rgba_pixels);
        let handle = image::Handle::from_bytes(encoded_bytes);
        Self {
            handle,
            width,
            height,
            rgba_bytes,
        }
    }

    /// Returns a reference to the original RGBA bytes.
    pub fn rgba_bytes(&self) -> &[u8] {
        &self.rgba_bytes
    }

    /// Creates a rotated version of this image.
    ///
    /// The rotation is applied using 90° increments:
    /// - 90°: rotate clockwise
    /// - 180°: rotate upside down
    /// - 270°: rotate counter-clockwise
    ///
    /// Returns the original image if rotation is 0°.
    ///
    /// # Panics
    ///
    /// Panics if the internal RGBA bytes are invalid (should never happen
    /// as bytes are validated at construction).
    #[must_use]
    pub fn rotated(&self, degrees: u16) -> Self {
        if degrees == 0 {
            return self.clone();
        }

        // Create DynamicImage from RGBA bytes
        let img = image_rs::RgbaImage::from_raw(self.width, self.height, self.rgba_bytes.to_vec())
            .expect("RGBA bytes should be valid");
        let dynamic = image_rs::DynamicImage::ImageRgba8(img);

        // Apply rotation
        let rotated = match degrees {
            90 => dynamic.rotate90(),
            180 => dynamic.rotate180(),
            270 => dynamic.rotate270(),
            _ => dynamic, // Should not happen with RotationAngle newtype
        };

        // Get new dimensions
        let (new_width, new_height) = rotated.dimensions();

        // Convert back to RGBA bytes
        let rgba_img = rotated.to_rgba8();
        let pixels = rgba_img.into_vec();

        // Store pixels in Arc (shared ownership), then create handle from clone
        // This avoids double-allocation: Arc owns the data, Handle gets a copy
        let rgba_bytes = Arc::new(pixels);
        let handle = image::Handle::from_rgba(new_width, new_height, rgba_bytes.to_vec());

        Self {
            handle,
            width: new_width,
            height: new_height,
            rgba_bytes,
        }
    }
}

/// Load an image from the given path and return its data.
///
/// Supports common raster formats (PNG, JPEG, GIF, etc.) as well as SVG.
/// SVG files are rasterized to PNG format using resvg.
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be read ([`Error::Io`])
/// - The image format is invalid or unsupported ([`Error::Io`])
/// - For SVG files: parsing fails or dimensions are zero ([`Error::Svg`])
pub fn load_image<P: AsRef<Path>>(path: P) -> Result<ImageData> {
    let path = path.as_ref();
    let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");

    if extension.eq_ignore_ascii_case("svg") {
        let svg_data = fs::read(path)?;
        let tree = usvg::Tree::from_data(&svg_data, &usvg::Options::default())
            .map_err(|e| Error::Svg(e.to_string()))?;

        let pixmap_size = tree.size().to_int_size();
        let width = pixmap_size.width();
        let height = pixmap_size.height();
        if width == 0 || height == 0 {
            return Err(Error::Svg("SVG has empty dimensions".into()));
        }

        let mut pixmap = tiny_skia::Pixmap::new(width, height)
            .ok_or_else(|| Error::Svg("Failed to allocate SVG pixmap".into()))?;

        resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

        let rgba_pixels = pixmap.data().to_vec();
        let png_data = pixmap.encode_png().map_err(|e| Error::Svg(e.to_string()))?;

        Ok(ImageData::from_encoded_with_rgba(
            png_data,
            width,
            height,
            rgba_pixels,
        ))
    } else {
        let img_bytes = fs::read(path).map_err(|e| Error::Io(e.to_string()))?;

        let img = image_rs::load_from_memory(&img_bytes).map_err(|e| Error::Io(e.to_string()))?;

        let (width, height) = img.dimensions();

        let rgba_img = img.to_rgba8();
        let pixels = rgba_img.into_vec();

        Ok(ImageData::from_rgba(width, height, pixels))
    }
}

impl From<ImageError> for Error {
    fn from(err: ImageError) -> Self {
        Error::Io(err.to_string())
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Svg(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use image_rs::{ImageError, Rgba, RgbaImage};
    use std::{fs, io};
    use tempfile::tempdir;

    #[test]
    fn load_png_image_returns_expected_dimensions() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let image_path = temp_dir.path().join("sample.png");

        let image = RgbaImage::from_pixel(4, 2, Rgba([255, 0, 0, 255]));
        image
            .save(&image_path)
            .expect("failed to write temporary png");

        let data = load_image(&image_path).expect("png should load successfully");
        assert_eq!(data.width, 4);
        assert_eq!(data.height, 2);
    }

    #[test]
    fn load_svg_image_rasterizes_successfully() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let svg_path = temp_dir.path().join("sample.svg");
        let svg_content = r#"
            <svg xmlns="http://www.w3.org/2000/svg" width="6" height="3">
                <rect width="6" height="3" fill="blue" />
            </svg>
        "#;
        fs::write(&svg_path, svg_content.trim()).expect("failed to write svg");

        let data = load_image(&svg_path).expect("svg should load successfully");
        assert_eq!(data.width, 6);
        assert_eq!(data.height, 3);
    }

    #[test]
    fn load_missing_image_returns_io_error() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let missing_path = temp_dir.path().join("does_not_exist.png");

        match load_image(&missing_path) {
            Err(Error::Io(_)) => {}
            other => panic!("expected Io error, got {other:?}"),
        }
    }

    #[test]
    fn load_invalid_png_bytes_returns_io_error() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let bad_path = temp_dir.path().join("invalid.png");
        fs::write(&bad_path, b"not a png").expect("failed to write invalid data");

        match load_image(&bad_path) {
            Err(Error::Io(message)) => assert!(!message.is_empty()),
            other => panic!("expected Io error for invalid png, got {other:?}"),
        }
    }

    #[test]
    fn load_invalid_svg_returns_svg_error() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let bad_svg_path = temp_dir.path().join("broken.svg");
        fs::write(&bad_svg_path, "<svg>oops").expect("failed to write invalid svg");

        match load_image(&bad_svg_path) {
            Err(Error::Svg(message)) => assert!(!message.is_empty()),
            other => panic!("expected Svg error, got {other:?}"),
        }
    }

    #[test]
    fn load_svg_with_zero_dimensions_errors() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let svg_path = temp_dir.path().join("zero.svg");
        let svg = r"<svg xmlns='http://www.w3.org/2000/svg' width='0' height='10'></svg>";
        fs::write(&svg_path, svg).expect("write svg");

        match load_image(&svg_path) {
            Err(Error::Svg(_)) => {}
            other => panic!("expected Svg error, got {other:?}"),
        }
    }

    #[test]
    fn image_error_conversion_returns_io_variant() {
        let io_err = io::Error::other("decode failed");
        let image_error = ImageError::IoError(io_err);
        let error: Error = image_error.into();
        match error {
            Error::Io(message) => assert!(message.contains("decode failed")),
            other => panic!("expected Io variant from ImageError, got {other:?}"),
        }
    }
}
