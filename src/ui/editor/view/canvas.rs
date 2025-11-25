// SPDX-License-Identifier: MPL-2.0
//! Image canvas composition with overlays.

use crate::config::BackgroundTheme;
use crate::ui::theme;
use iced::widget::{center, container, image, Canvas, Stack};
use iced::{ContentFit, Element, Length};

use super::super::{CropOverlayRenderer, Message, ResizeOverlayRenderer, State, ViewContext};

pub fn view<'a>(state: &'a State, ctx: &ViewContext<'a>) -> Element<'a, Message> {
    let current_display = state.display_image();
    let img_width = current_display.width;
    let img_height = current_display.height;

    let image_widget = image(current_display.handle.clone()).content_fit(ContentFit::Contain);

    let image_with_overlay: Element<'a, Message> = if state.crop_state.overlay.visible {
        Stack::new()
            .push(image_widget)
            .push(
                Canvas::new(CropOverlayRenderer {
                    crop_x: state.crop_state.x,
                    crop_y: state.crop_state.y,
                    crop_width: state.crop_state.width,
                    crop_height: state.crop_state.height,
                    img_width,
                    img_height,
                })
                .width(Length::Fill)
                .height(Length::Fill),
            )
            .into()
    } else if state.resize_state.overlay.visible {
        Stack::new()
            .push(image_widget)
            .push(
                Canvas::new(ResizeOverlayRenderer {
                    original_width: state.resize_state.overlay.original_width,
                    original_height: state.resize_state.overlay.original_height,
                    new_width: state.resize_state.width,
                    new_height: state.resize_state.height,
                })
                .width(Length::Fill)
                .height(Length::Fill),
            )
            .into()
    } else {
        image_widget.into()
    };

    let background_theme = ctx.background_theme;

    let build_image_surface = || {
        container(center(image_with_overlay))
            .width(Length::Fill)
            .height(Length::Fill)
    };

    if theme::is_checkerboard(background_theme) {
        theme::wrap_with_checkerboard(build_image_surface())
    } else {
        let bg_color = match background_theme {
            BackgroundTheme::Light => theme::viewer_light_surface_color(),
            BackgroundTheme::Dark => theme::viewer_dark_surface_color(),
            BackgroundTheme::Checkerboard => unreachable!(),
        };

        build_image_surface()
            .style(move |_theme: &iced::Theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(bg_color)),
                ..Default::default()
            })
            .into()
    }
}
