// SPDX-License-Identifier: MPL-2.0
//! Image viewer module responsible for rendering loaded images.

pub mod controls;

use crate::image_handler::ImageData;
use iced::{widget::Image, Element, Length};

pub fn view_image(
    image_data: &ImageData,
    zoom_percent: f32,
) -> Element<'_, super::super::app::Message> {
    let scale = (zoom_percent / 100.0).max(0.01);
    let width = (image_data.width as f32 * scale).max(1.0);
    let height = (image_data.height as f32 * scale).max(1.0);

    Image::new(image_data.handle.clone())
        .width(Length::Fixed(width))
        .height(Length::Fixed(height))
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::widget::image::Handle;

    #[test]
    fn view_image_produces_element() {
        let pixels = vec![0_u8, 0, 0, 255];
        let image_data = ImageData {
            handle: Handle::from_rgba(1, 1, pixels),
            width: 1,
            height: 1,
        };

        let _element = view_image(&image_data, 100.0);
        // Smoke test to ensure rendering succeeds.
    }
}
