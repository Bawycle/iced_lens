//! This module defines the UI component responsible for displaying images within
//! the IcedLens application. It takes processed `ImageData` and renders it
//! using Iced's `Image` widget.
//!
//! # Examples
//!
//! ```no_run
//! use iced_lens::image_handler::ImageData;
//! use iced_lens::ui::viewer;
//! use iced_lens::app::Message; // Import Message
//! use iced::{Element, widget::Container};
//!
//! # fn dummy_image_data() -> ImageData {
//! #     ImageData {
//! #         handle: iced::widget::image::Handle::from_path("dummy.png"),
//! #         width: 100,
//! #         height: 100,
//! #     }
//! # }
//! #
//! // Assume `image_data` is obtained from image_handler::load_image
//! let image_data = dummy_image_data();
//! let image_viewer_element: Element<'_, Message> = viewer::view_image(&image_data, 100.0);
//!
//! let content = Container::new(image_viewer_element);
//! // ... add to your application's view
//! ```

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

    #[test]
    fn view_image_produces_element() {
        let pixels = vec![0_u8, 0, 0, 255];
        let image_data = ImageData {
            handle: iced::widget::image::Handle::from_rgba(1, 1, pixels),
            width: 1,
            height: 1,
        };

        let _element = view_image(&image_data, 100.0);
        // The assertion is implicit: we simply ensure the helper does not panic and returns.
    }
}
