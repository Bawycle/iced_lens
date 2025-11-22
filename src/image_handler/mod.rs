//! This module is responsible for loading and decoding image files from various
//! formats (e.g., PNG, JPEG, GIF, SVG). It provides a unified `ImageData`
//! structure that can be used by the UI to display images.
//!
//! # Examples
//!
//! ```no_run
//! use iced_lens::image_handler::{self, ImageData};
//! use iced_lens::error::Result;
//! use std::path::PathBuf;
//!
//! # fn main() -> Result<()> {
//! let image_path = PathBuf::from("./path/to/your/image.png");
//! let image_data: ImageData = image_handler::load_image(&image_path)?;
//!
//! println!("Loaded image: {}x{}", image_data.width, image_data.height);
//! # Ok(())
//! # }
//! ```

use crate::error::{Error, Result};
use iced::widget::image;
use image_rs::{GenericImageView, ImageError};
use resvg::usvg;
use std::fs;
use std::path::Path;
use tiny_skia;

#[derive(Debug, Clone)]
pub struct ImageData {
    pub handle: image::Handle,
    pub width: u32,
    pub height: u32,
}

pub fn load_image<P: AsRef<Path>>(path: P) -> Result<ImageData> {
    let path = path.as_ref();
    let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");

    match extension.to_lowercase().as_str() {
        "svg" => {
            let svg_data = fs::read(path)?;
            let tree = usvg::Tree::from_data(&svg_data, &usvg::Options::default())
                .map_err(|e| Error::Svg(e.to_string()))?;

            let pixmap_size = tree.size().to_int_size();
            let mut pixmap =
                tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
            resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

            let png_data = pixmap.encode_png().map_err(|e| Error::Svg(e.to_string()))?;
            let handle = image::Handle::from_memory(png_data);
            Ok(ImageData {
                handle,
                width: pixmap_size.width(),
                height: pixmap_size.height(),
            })
        }
        _ => {
            let img_bytes = fs::read(path).map_err(|e| Error::Io(e.to_string()))?;

            let img =
                image_rs::load_from_memory(&img_bytes).map_err(|e| Error::Io(e.to_string()))?;

            let (width, height) = img.dimensions();

            let rgba_img = img.to_rgba8();
            let pixels = rgba_img.into_vec();

            let handle = image::Handle::from_pixels(width, height, pixels);
            Ok(ImageData {
                handle,
                width,
                height,
            })
        }
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
            other => panic!("expected Io error, got {:?}", other),
        }
    }

    #[test]
    fn load_invalid_png_bytes_returns_io_error() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let bad_path = temp_dir.path().join("invalid.png");
        fs::write(&bad_path, b"not a png").expect("failed to write invalid data");

        match load_image(&bad_path) {
            Err(Error::Io(message)) => assert!(!message.is_empty()),
            other => panic!("expected Io error for invalid png, got {:?}", other),
        }
    }

    #[test]
    fn load_invalid_svg_returns_svg_error() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let bad_svg_path = temp_dir.path().join("broken.svg");
        fs::write(&bad_svg_path, "<svg>oops").expect("failed to write invalid svg");

        match load_image(&bad_svg_path) {
            Err(Error::Svg(message)) => assert!(!message.is_empty()),
            other => panic!("expected Svg error, got {:?}", other),
        }
    }

    #[test]
    fn image_error_conversion_returns_io_variant() {
        let io_err = io::Error::new(io::ErrorKind::Other, "decode failed");
        let image_error = ImageError::IoError(io_err);
        let error: Error = image_error.into();
        match error {
            Error::Io(message) => assert!(message.contains("decode failed")),
            other => panic!("expected Io variant from ImageError, got {:?}", other),
        }
    }
}
