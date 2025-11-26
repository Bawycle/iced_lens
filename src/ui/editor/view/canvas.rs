// SPDX-License-Identifier: MPL-2.0
//! Image canvas composition with overlays.

use crate::config::BackgroundTheme;
use crate::image_handler::ImageData;
use crate::ui::theme;
use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{container, image, Canvas, Scrollable, Stack};
use iced::{ContentFit, Element, Length};

use super::super::{
    overlay::{CropOverlayRenderer, ResizeOverlayRenderer},
    CropState, Message, ResizeState, State, ViewContext,
};

pub struct CanvasModel<'a> {
    pub display_image: &'a ImageData,
    pub crop_state: &'a CropState,
    pub resize_state: &'a ResizeState,
}

impl<'a> CanvasModel<'a> {
    pub fn from_state(state: &'a State) -> Self {
        Self {
            display_image: state.display_image(),
            crop_state: &state.crop_state,
            resize_state: &state.resize_state,
        }
    }
}

pub fn view<'a>(model: CanvasModel<'a>, ctx: &ViewContext<'a>) -> Element<'a, Message> {
    let current_display = model.display_image;
    let img_width = current_display.width;
    let img_height = current_display.height;

    let image_widget = image(current_display.handle.clone()).content_fit(ContentFit::Contain);

    let image_with_overlay: Element<'a, Message> = if model.crop_state.overlay.visible {
        Stack::new()
            .push(image_widget)
            .push(
                Canvas::new(CropOverlayRenderer {
                    crop_x: model.crop_state.x,
                    crop_y: model.crop_state.y,
                    crop_width: model.crop_state.width,
                    crop_height: model.crop_state.height,
                    img_width,
                    img_height,
                })
                .width(Length::Fill)
                .height(Length::Fill),
            )
            .into()
    } else if model.resize_state.overlay.visible {
        Stack::new()
            .push(image_widget)
            .push(
                Canvas::new(ResizeOverlayRenderer {
                    original_width: model.resize_state.overlay.original_width,
                    original_height: model.resize_state.overlay.original_height,
                    new_width: model.resize_state.width,
                    new_height: model.resize_state.height,
                })
                .width(Length::Fill)
                .height(Length::Fill),
            )
            .into()
    } else {
        image_widget.into()
    };

    let background_theme = ctx.background_theme;

    // Wrap image in scrollable for when preview exceeds available space
    let scrollable = Scrollable::new(image_with_overlay)
        .width(Length::Fill)
        .height(Length::Fill)
        .direction(Direction::Both {
            vertical: Scrollbar::new(),
            horizontal: Scrollbar::new(),
        });

    let build_image_surface = || {
        container(scrollable)
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
