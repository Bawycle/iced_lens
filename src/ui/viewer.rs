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
//! let image_viewer_element: Element<'_, Message> = viewer::view_image(&image_data);
//!
//! let content = Container::new(image_viewer_element);
//! // ... add to your application's view
//! ```

use crate::image_handler::ImageData;
use iced::{
    widget::Image, // Removed Container
    Element,
    Length,
};

pub fn view_image(image_data: &ImageData) -> Element<'_, super::super::app::Message> {
    Image::new(image_data.handle.clone())
        .width(Length::Fixed(image_data.width as f32))
        .height(Length::Fixed(image_data.height as f32))
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

        let _element = view_image(&image_data);
        // The assertion is implicit: we simply ensure the helper does not panic and returns.
    }
}
