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
                .map_err(|e| {
                    Error::Svg(e.to_string())
                })?;

            let pixmap_size = tree.size().to_int_size();
            let mut pixmap =
                tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
            resvg::render(
                &tree,
                tiny_skia::Transform::default(),
                &mut pixmap.as_mut(),
            );

            let png_data = pixmap.encode_png().map_err(|e| {
                Error::Svg(e.to_string())
            })?;
            let handle = image::Handle::from_memory(png_data);
            Ok(ImageData {
                handle,
                width: pixmap_size.width(),
                height: pixmap_size.height(),
            })
        }
        _ => {
            let img_bytes = fs::read(path).map_err(|e| {
                Error::Io(e.to_string())
            })?;

            let img = image_rs::load_from_memory(&img_bytes).map_err(|e| {
                Error::Io(e.to_string())
            })?;

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
    // Tests will be updated later when we have sample data
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
