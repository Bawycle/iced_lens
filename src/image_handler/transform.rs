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

/// Crop the image to the specified rectangle.
///
/// The rectangle coordinates are clamped to the image boundaries.
/// If the resulting crop area is invalid (zero width or height), returns None.
pub fn crop(image: &DynamicImage, x: u32, y: u32, width: u32, height: u32) -> Option<DynamicImage> {
    let img_width = image.width();
    let img_height = image.height();

    // Clamp coordinates to image boundaries
    let x = x.min(img_width.saturating_sub(1));
    let y = y.min(img_height.saturating_sub(1));

    // Calculate available width and height from the crop start point
    let max_width = img_width.saturating_sub(x);
    let max_height = img_height.saturating_sub(y);

    let width = width.min(max_width).max(1);
    let height = height.min(max_height).max(1);

    // Ensure we have a valid crop area
    if width == 0 || height == 0 {
        return None;
    }

    Some(image.crop_imm(x, y, width, height))
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

    #[test]
    fn crop_within_bounds() {
        let img = create_test_image(10, 8);
        let cropped = crop(&img, 2, 2, 4, 3);
        assert!(cropped.is_some());
        let result = cropped.unwrap();
        assert_eq!(result.width(), 4);
        assert_eq!(result.height(), 3);
    }

    #[test]
    fn crop_clamps_to_boundaries() {
        let img = create_test_image(10, 8);
        // Request crop that extends beyond image
        let cropped = crop(&img, 8, 6, 10, 10);
        assert!(cropped.is_some());
        let result = cropped.unwrap();
        // Should be clamped to available area (10-8=2, 8-6=2)
        assert_eq!(result.width(), 2);
        assert_eq!(result.height(), 2);
    }

    #[test]
    fn crop_at_origin() {
        let img = create_test_image(10, 8);
        let cropped = crop(&img, 0, 0, 5, 5);
        assert!(cropped.is_some());
        let result = cropped.unwrap();
        assert_eq!(result.width(), 5);
        assert_eq!(result.height(), 5);
    }

    #[test]
    fn crop_entire_image() {
        let img = create_test_image(10, 8);
        let cropped = crop(&img, 0, 0, 10, 8);
        assert!(cropped.is_some());
        let result = cropped.unwrap();
        assert_eq!(result.width(), 10);
        assert_eq!(result.height(), 8);
    }
}
