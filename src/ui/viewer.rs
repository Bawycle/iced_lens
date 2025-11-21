use crate::image_handler::ImageData;
use iced::widget::{image, Column};
use iced::Element;

pub fn view_image<'a, Message: 'a>(image_data: &'a ImageData) -> Element<'a, Message> {
    Column::new().push(image::Image::new(image_data.handle.clone())).into()
}
