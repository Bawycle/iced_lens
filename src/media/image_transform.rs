// SPDX-License-Identifier: MPL-2.0
//! Image transformation functions for rotate, crop, and resize operations.

use crate::error::Result;
use crate::media::ImageData;
use image_rs::{imageops::FilterType, DynamicImage, GenericImageView};

/// Rotate an image 90 degrees counter-clockwise (left).
pub fn rotate_left(image: &DynamicImage) -> DynamicImage {
    image.rotate270()
}

/// Rotate an image 90 degrees clockwise (right).
pub fn rotate_right(image: &DynamicImage) -> DynamicImage {
    image.rotate90()
}

/// Flip an image horizontally (mirror left-to-right).
pub fn flip_horizontal(image: &DynamicImage) -> DynamicImage {
    image.fliph()
}

/// Flip an image vertically (mirror top-to-bottom).
pub fn flip_vertical(image: &DynamicImage) -> DynamicImage {
    image.flipv()
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

/// Adjust brightness of an image.
///
/// The `value` parameter ranges from -100 to +100:
/// - Negative values darken the image
/// - Positive values brighten the image
/// - Zero returns the image unchanged
pub fn adjust_brightness(image: &DynamicImage, value: i32) -> DynamicImage {
    if value == 0 {
        return image.clone();
    }
    image.brighten(value)
}

/// Adjust contrast of an image.
///
/// The `value` parameter ranges from -100 to +100:
/// - Negative values reduce contrast (flatten toward gray)
/// - Positive values increase contrast (more separation between light/dark)
/// - Zero returns the image unchanged
///
/// Internally converts the -100..+100 range to a multiplier for the image crate's
/// contrast function which expects a float (-100 = 0.0x, 0 = 1.0x, +100 = 2.0x).
pub fn adjust_contrast(image: &DynamicImage, value: i32) -> DynamicImage {
    if value == 0 {
        return image.clone();
    }
    // Convert -100..+100 to a factor: -100 -> -100.0, 0 -> 0.0, +100 -> +100.0
    // The image crate's contrast function interprets this as percentage adjustment
    let factor = value as f32;
    image.adjust_contrast(factor)
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

    #[test]
    fn flip_horizontal_preserves_dimensions() {
        let img = create_test_image(8, 6);
        let flipped = flip_horizontal(&img);
        assert_eq!(flipped.width(), 8);
        assert_eq!(flipped.height(), 6);
    }

    #[test]
    fn flip_vertical_preserves_dimensions() {
        let img = create_test_image(8, 6);
        let flipped = flip_vertical(&img);
        assert_eq!(flipped.width(), 8);
        assert_eq!(flipped.height(), 6);
    }

    #[test]
    fn flip_horizontal_mirrors_pixels_left_to_right() {
        // Create an image with distinct left and right sides
        let mut buffer = ImageBuffer::from_pixel(4, 2, image_rs::Rgba([0, 0, 0, 255]));
        // Fill right half with white
        for x in 2..4 {
            for y in 0..2 {
                buffer.put_pixel(x, y, image_rs::Rgba([255, 255, 255, 255]));
            }
        }
        let img = DynamicImage::ImageRgba8(buffer);

        let flipped = flip_horizontal(&img);
        let flipped_rgba = flipped.to_rgba8();

        // After horizontal flip, left should be white, right should be black
        assert_eq!(
            flipped_rgba.get_pixel(0, 0).0,
            [255, 255, 255, 255],
            "Top-left should be white after flip"
        );
        assert_eq!(
            flipped_rgba.get_pixel(3, 0).0,
            [0, 0, 0, 255],
            "Top-right should be black after flip"
        );
    }

    #[test]
    fn flip_vertical_mirrors_pixels_top_to_bottom() {
        // Create an image with distinct top and bottom halves
        let mut buffer = ImageBuffer::from_pixel(2, 4, image_rs::Rgba([0, 0, 0, 255]));
        // Fill bottom half with white
        for x in 0..2 {
            for y in 2..4 {
                buffer.put_pixel(x, y, image_rs::Rgba([255, 255, 255, 255]));
            }
        }
        let img = DynamicImage::ImageRgba8(buffer);

        let flipped = flip_vertical(&img);
        let flipped_rgba = flipped.to_rgba8();

        // After vertical flip, top should be white, bottom should be black
        assert_eq!(
            flipped_rgba.get_pixel(0, 0).0,
            [255, 255, 255, 255],
            "Top-left should be white after flip"
        );
        assert_eq!(
            flipped_rgba.get_pixel(0, 3).0,
            [0, 0, 0, 255],
            "Bottom-left should be black after flip"
        );
    }

    #[test]
    fn brightness_zero_returns_unchanged() {
        let img = create_test_image(4, 4);
        let result = adjust_brightness(&img, 0);
        assert_eq!(result.width(), 4);
        assert_eq!(result.height(), 4);
    }

    #[test]
    fn brightness_positive_lightens_image() {
        let buffer = ImageBuffer::from_pixel(2, 2, image_rs::Rgba([100, 100, 100, 255]));
        let img = DynamicImage::ImageRgba8(buffer);

        let brightened = adjust_brightness(&img, 50);
        let rgba = brightened.to_rgba8();
        let pixel = rgba.get_pixel(0, 0).0;

        // Brightness should increase (but not overflow 255)
        assert!(pixel[0] > 100, "Red channel should be brighter");
        assert!(pixel[1] > 100, "Green channel should be brighter");
        assert!(pixel[2] > 100, "Blue channel should be brighter");
    }

    #[test]
    fn brightness_negative_darkens_image() {
        let buffer = ImageBuffer::from_pixel(2, 2, image_rs::Rgba([100, 100, 100, 255]));
        let img = DynamicImage::ImageRgba8(buffer);

        let darkened = adjust_brightness(&img, -50);
        let rgba = darkened.to_rgba8();
        let pixel = rgba.get_pixel(0, 0).0;

        // Brightness should decrease (but not underflow 0)
        assert!(pixel[0] < 100, "Red channel should be darker");
        assert!(pixel[1] < 100, "Green channel should be darker");
        assert!(pixel[2] < 100, "Blue channel should be darker");
    }

    #[test]
    fn contrast_zero_returns_unchanged() {
        let img = create_test_image(4, 4);
        let result = adjust_contrast(&img, 0);
        assert_eq!(result.width(), 4);
        assert_eq!(result.height(), 4);
    }

    #[test]
    fn contrast_preserves_dimensions() {
        let img = create_test_image(8, 6);
        let result = adjust_contrast(&img, 50);
        assert_eq!(result.width(), 8);
        assert_eq!(result.height(), 6);
    }
}
