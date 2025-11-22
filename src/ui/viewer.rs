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
    widget::{Image}, // Removed Container
    Element, Length,
};

pub fn view_image(image_data: &ImageData) -> Element<'_, super::super::app::Message> {
    Image::new(image_data.handle.clone())
        .width(Length::Fixed(image_data.width as f32))
        .height(Length::Fixed(image_data.height as f32))
        .into()
}
