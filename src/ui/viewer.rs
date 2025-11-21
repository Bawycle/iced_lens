use crate::image_handler::ImageData;
use iced::widget::image;
use iced::{Element, Length};

pub fn view_image<'a, Message: 'a>(image_data: &'a ImageData) -> Element<'a, Message> {
    image::Image::new(image_data.handle.clone())
        .width(Length::Shrink)
        .height(Length::Shrink)
        .into()
}
