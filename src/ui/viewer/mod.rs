// SPDX-License-Identifier: MPL-2.0
//! Image viewer module responsible for rendering loaded images and related UI.

pub mod component;
pub mod controls;
pub mod pane;
pub mod state;

use self::component::Message;
use crate::i18n::fluent::I18n;
use crate::image_handler::ImageData;
use crate::ui::state::ZoomState;
use iced::widget::{button, Column, Container, Image, Row, Text};
use iced::{alignment, Element, Length};

pub fn view_image(image_data: &ImageData, zoom_percent: f32) -> Element<'_, Message> {
    let scale = (zoom_percent / 100.0).max(0.01);
    let width = (image_data.width as f32 * scale).max(1.0);
    let height = (image_data.height as f32 * scale).max(1.0);

    Image::new(image_data.handle.clone())
        .width(Length::Fixed(width))
        .height(Length::Fixed(height))
        .into()
}

pub struct ViewContext<'a> {
    pub i18n: &'a I18n,
    pub error: Option<ErrorContext<'a>>,
    pub image: Option<ImageContext<'a>>,
}

pub struct ErrorContext<'a> {
    pub friendly_text: &'a str,
    pub details: &'a str,
    pub show_details: bool,
}

pub struct ImageContext<'a> {
    pub i18n: &'a I18n,
    pub controls_context: controls::ViewContext<'a>,
    pub zoom: &'a ZoomState,
    pub pane_context: pane::ViewContext,
    pub pane_model: pane::ViewModel<'a>,
    pub controls_visible: bool,
}

pub fn view(ctx: ViewContext<'_>) -> Element<'_, Message> {
    if let Some(error) = ctx.error {
        return error_view(ctx.i18n, error);
    }

    if let Some(image) = ctx.image {
        return image_view(image);
    }

    Text::new(ctx.i18n.tr("hello-message")).into()
}

fn error_view<'a>(i18n: &'a I18n, error: ErrorContext<'a>) -> Element<'a, Message> {
    let heading = Container::new(Text::new(i18n.tr("error-load-image-heading")).size(24))
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Center);

    let summary = Container::new(Text::new(error.friendly_text).width(Length::Fill))
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Center);

    let toggle_label = if error.show_details {
        i18n.tr("error-details-hide")
    } else {
        i18n.tr("error-details-show")
    };

    let toggle_button =
        Container::new(button(Text::new(toggle_label)).on_press(Message::ToggleErrorDetails))
            .align_x(alignment::Horizontal::Center);

    let mut error_content = Column::new()
        .spacing(12)
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .push(heading)
        .push(summary)
        .push(toggle_button);

    if error.show_details {
        let details_heading =
            Container::new(Text::new(i18n.tr("error-details-technical-heading")).size(16))
                .width(Length::Fill)
                .align_x(alignment::Horizontal::Center);

        let details_body = Container::new(Text::new(error.details).width(Length::Fill))
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Left);

        let details_column = Column::new()
            .spacing(8)
            .width(Length::Fill)
            .push(details_heading)
            .push(details_body);

        error_content = error_content.push(
            Container::new(details_column)
                .width(Length::Fill)
                .padding(16),
        );
    }

    Container::new(error_content)
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .into()
}

fn image_view(ctx: ImageContext<'_>) -> Element<'_, Message> {
    let pane_view = pane::view(ctx.pane_context, ctx.pane_model);

    let mut column = Column::new()
        .spacing(16)
        .width(Length::Fill)
        .height(Length::Fill);

    // Add top navigation bar with Settings and Edit buttons
    if ctx.controls_visible {
        let mut top_bar = Row::new().spacing(10).padding(10);

        let settings_button =
            button(Text::new(ctx.i18n.tr("open-settings-button"))).on_press(Message::OpenSettings);
        top_bar = top_bar.push(settings_button);

        let edit_button = button(Text::new("✏ Edit")).on_press(Message::EnterEditor);
        top_bar = top_bar.push(edit_button);

        let delete_button = button(Text::new(ctx.i18n.tr("viewer-delete-button")))
            .on_press(Message::DeleteCurrentImage);
        top_bar = top_bar.push(delete_button);

        column = column.push(
            Container::new(top_bar)
                .width(Length::Fill)
                .align_x(alignment::Horizontal::Left),
        );

        let controls_view = controls::view(ctx.controls_context, ctx.zoom).map(Message::Controls);
        column = column.push(controls_view);
    }

    column.push(pane_view).into()
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
