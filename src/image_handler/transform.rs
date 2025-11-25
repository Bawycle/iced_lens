// SPDX-License-Identifier: MPL-2.0
//! Image transformation functions for rotate, crop, and resize operations.

use crate::error::Result;
use crate::image_handler::ImageData;
use image_rs::{imageops::FilterType, DynamicImage, GenericImageView};

/// Rotate an image 90 degrees counter-clockwise (left).
pub fn rotate_left(image: &DynamicImage) -> DynamicImage {
    image.rotate270()
}

/// Rotate an image 90 degrees clockwise (right).
pub fn rotate_right(image: &DynamicImage) -> DynamicImage {
    image.rotate90()
}

/// Convert DynamicImage back to ImageData for display.
pub fn dynamic_to_image_data(dynamic: &DynamicImage) -> Result<ImageData> {
    let (width, height) = dynamic.dimensions();
    let rgba_img = dynamic.to_rgba8();
    let pixels = rgba_img.into_vec();

    let handle = iced::widget::image::Handle::from_rgba(width, height, pixels);

    Ok(ImageData {
        handle,
        width,
        height,
    })
}

/// Resize the image to the provided dimensions using a high-quality filter.
pub fn resize(image: &DynamicImage, width: u32, height: u32) -> DynamicImage {
    let width = width.max(1);
    let height = height.max(1);
    image.resize_exact(width, height, FilterType::Lanczos3)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image_rs::DynamicImage;
    use image_rs::ImageBuffer;

    fn create_test_image(width: u32, height: u32) -> DynamicImage {
        let buffer = ImageBuffer::from_pixel(width, height, image_rs::Rgba([0, 0, 0, 0]));
        DynamicImage::ImageRgba8(buffer)
    }

    #[test]
    fn rotate_left_swaps_dimensions() {
        let img = create_test_image(4, 3);
        let rotated = rotate_left(&img);
        assert_eq!(rotated.width(), 3);
        assert_eq!(rotated.height(), 4);
    }

    #[test]
    fn rotate_right_swaps_dimensions() {
        let img = create_test_image(4, 3);
        let rotated = rotate_right(&img);
        assert_eq!(rotated.width(), 3);
        assert_eq!(rotated.height(), 4);
    }

    #[test]
    fn resize_changes_dimensions() {
        let img = create_test_image(8, 4);
        let resized = resize(&img, 4, 2);
        assert_eq!(resized.width(), 4);
        assert_eq!(resized.height(), 2);
    }
}
