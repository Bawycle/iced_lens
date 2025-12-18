// SPDX-License-Identifier: MPL-2.0
//! Image canvas composition with overlays.

use crate::config::BackgroundTheme;
use crate::media::ImageData;
use crate::ui::components::checkerboard;
use crate::ui::design_tokens::{opacity, radius, spacing, typography};
use crate::ui::theme;
use crate::ui::widgets::AnimatedSpinner;
use iced::alignment::Horizontal;
use iced::widget::{container, image, text, Canvas, Column, Stack};
use iced::{Background, Color, Element, Length, Theme};

use super::super::{
    overlay::{CropOverlayRenderer, ResizeOverlayRenderer},
    CropState, DeblurState, Message, ResizeState, State, ViewContext,
};
use super::scrollable_canvas;

pub struct CanvasModel<'a> {
    pub display_image: &'a ImageData,
    pub crop_state: &'a CropState,
    pub resize_state: &'a ResizeState,
    pub deblur_state: &'a DeblurState,
    pub image_width: f32,
    pub image_height: f32,
}

impl<'a> CanvasModel<'a> {
    pub fn from_state(state: &'a State) -> Self {
        let display_image = state.display_image();
        Self {
            display_image,
            crop_state: &state.crop_state,
            resize_state: &state.resize_state,
            deblur_state: &state.deblur_state,
            image_width: display_image.width as f32,
            image_height: display_image.height as f32,
        }
    }
}

pub fn view<'a>(model: CanvasModel<'a>, ctx: &ViewContext<'a>) -> Element<'a, Message> {
    let current_display = model.display_image;
    let img_width = current_display.width;
    let img_height = current_display.height;

    // Render image at natural size (will be centered by center() widget)
    let image_widget = image(current_display.handle.clone())
        .width(Length::Fixed(img_width as f32))
        .height(Length::Fixed(img_height as f32));

    let image_with_overlay: Element<'a, Message> = if model.deblur_state.is_processing {
        // Deblur processing overlay: animated spinner with text (consistent with media loading)
        let spinner = AnimatedSpinner::new(
            theme::overlay_arrow_light_color(),
            model.deblur_state.spinner_rotation,
        )
        .into_element();

        let loading_text = text(ctx.i18n.tr("image-editor-deblur-processing"))
            .size(typography::BODY_LG);

        let loading_content = Column::new()
            .spacing(spacing::SM)
            .align_x(Horizontal::Center)
            .push(spinner)
            .push(loading_text);

        let loading_overlay = container(loading_content)
            .padding(spacing::MD)
            .style(move |_theme: &Theme| container::Style {
                background: Some(Background::Color(Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: opacity::OVERLAY_MEDIUM,
                })),
                border: iced::Border {
                    radius: radius::MD.into(),
                    ..Default::default()
                },
                text_color: Some(theme::overlay_arrow_light_color()),
                ..Default::default()
            });

        let overlay = container(loading_overlay)
            .width(Length::Fixed(img_width as f32))
            .height(Length::Fixed(img_height as f32))
            .align_x(Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center);

        Stack::new()
            .push(image_widget)
            .push(overlay)
            .into()
    } else if model.crop_state.overlay.visible {
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

    // Wrap image in scrollable canvas (centered for small, scrollable for large)
    let scrollable = scrollable_canvas::scrollable_canvas(
        image_with_overlay,
        model.image_width,
        model.image_height,
    );

    let build_image_surface = || {
        container(scrollable)
            .width(Length::Fill)
            .height(Length::Fill)
    };

    if theme::is_checkerboard(background_theme) {
        checkerboard::wrap(build_image_surface())
    } else {
        let bg_color = match background_theme {
            BackgroundTheme::Light => theme::viewer_light_surface_color(),
            BackgroundTheme::Dark => theme::viewer_dark_surface_color(),
            BackgroundTheme::Checkerboard => unreachable!(),
        };

        build_image_surface()
            .style(theme::editor_canvas_style(bg_color))
            .into()
    }
}
