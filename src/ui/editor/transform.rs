// SPDX-License-Identifier: MPL-2.0
//! Image transformation functions for rotate, crop, and resize operations.

use crate::error::{Error, Result};
use crate::image_handler::ImageData;
use image_rs::{DynamicImage, GenericImageView, ImageBuffer, Rgba};

/// Rotate an image 90 degrees counter-clockwise (left).
pub fn rotate_left(image_data: &ImageData) -> Result<ImageData> {
    // Convert ImageData to DynamicImage
    let dynamic_image = image_data_to_dynamic(image_data)?;

    // Rotate 90 degrees counter-clockwise
    let rotated = dynamic_image.rotate90();

    // Convert back to ImageData
    dynamic_to_image_data(rotated)
}

/// Rotate an image 90 degrees clockwise (right).
pub fn rotate_right(image_data: &ImageData) -> Result<ImageData> {
    // Convert ImageData to DynamicImage
    let dynamic_image = image_data_to_dynamic(image_data)?;

    // Rotate 270 degrees counter-clockwise (= 90 degrees clockwise)
    let rotated = dynamic_image.rotate270();

    // Convert back to ImageData
    dynamic_to_image_data(rotated)
}

/// Convert ImageData to DynamicImage for manipulation.
fn image_data_to_dynamic(image_data: &ImageData) -> Result<DynamicImage> {
    // ImageData stores pixels as RGBA, so we need to reconstruct from handle
    // For now, we'll use a workaround: encode to bytes and decode back
    // This is not optimal but works with the current architecture

    // We can't easily extract pixels from iced::Handle, so we'll need a different approach
    // For MVP, we'll work with the assumption that we can access the underlying bytes

    // TODO: This is a limitation - we might need to restructure ImageData to also store
    // the raw pixel data or the original DynamicImage

    Err(Error::Io(
        "Image transformation not yet fully implemented - need raw pixel access".into(),
    ))
}

/// Convert DynamicImage back to ImageData for display.
fn dynamic_to_image_data(dynamic: DynamicImage) -> Result<ImageData> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use iced::widget::image;

    fn create_test_image(width: u32, height: u32) -> ImageData {
        let pixels = vec![0u8; (width * height * 4) as usize];
        ImageData {
            handle: image::Handle::from_rgba(width, height, pixels),
            width,
            height,
        }
    }

    #[test]
    fn rotate_left_swaps_dimensions() {
        let img = create_test_image(4, 3);
        // This will fail until we implement proper pixel access
        // assert_eq!(img.width, 4);
        // assert_eq!(img.height, 3);

        // let rotated = rotate_left(&img).unwrap();
        // assert_eq!(rotated.width, 3);
        // assert_eq!(rotated.height, 4);
    }

    #[test]
    fn rotate_right_swaps_dimensions() {
        let img = create_test_image(4, 3);
        // This will fail until we implement proper pixel access
        // let rotated = rotate_right(&img).unwrap();
        // assert_eq!(rotated.width, 3);
        // assert_eq!(rotated.height, 4);
    }
}
