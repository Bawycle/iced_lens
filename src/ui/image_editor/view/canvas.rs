// SPDX-License-Identifier: MPL-2.0
//! Image canvas composition with overlays.
#![allow(clippy::cast_precision_loss)]

use crate::config::BackgroundTheme;
use crate::media::ImageData;
use crate::ui::components::checkerboard;
use crate::ui::design_tokens::{opacity, radius, spacing, typography};
use crate::ui::theme;
use crate::ui::widgets::AnimatedSpinner;
use iced::alignment::Horizontal;
use iced::mouse;
use iced::widget::{
    container, image, mouse_area, responsive, text, Canvas, Column, Container, Stack,
};
use iced::{Background, Color, Element, Length, Padding, Size, Theme};

use super::super::{
    overlay::{CropOverlayRenderer, ResizeOverlayRenderer},
    CanvasMessage, CropState, DeblurState, Message, ResizeState, State, ViewContext,
};
use super::scrollable_canvas;

pub struct CanvasModel<'a> {
    pub display_image: &'a ImageData,
    pub crop_state: &'a CropState,
    pub resize_state: &'a ResizeState,
    pub deblur_state: &'a DeblurState,
    /// Zoom scale factor (1.0 = 100%)
    pub zoom_scale: f32,
    /// Whether the user is currently dragging to pan
    pub is_dragging: bool,
    /// Whether crop tool is active (disables pan cursor)
    pub crop_active: bool,
    /// Whether AI upscale processing is in progress
    pub upscale_processing: bool,
}

impl<'a> CanvasModel<'a> {
    pub fn from_state(state: &'a State) -> Self {
        let display_image = state.display_image();
        Self {
            display_image,
            crop_state: &state.crop_state,
            resize_state: &state.resize_state,
            deblur_state: &state.deblur_state,
            zoom_scale: state.zoom.zoom_percent / 100.0,
            is_dragging: state.is_dragging(),
            crop_active: state.crop_state.overlay.visible,
            upscale_processing: state.resize_state.is_upscale_processing,
        }
    }
}

/// Calculate padding to center content within available space.
fn calculate_centering_padding(content_size: Size, available: Size) -> Padding {
    let horizontal = ((available.width - content_size.width) / 2.0).max(0.0);
    let vertical = ((available.height - content_size.height) / 2.0).max(0.0);

    Padding {
        top: vertical,
        right: horizontal,
        bottom: vertical,
        left: horizontal,
    }
}

/// Builds the processing overlay with spinner (used during deblur/upscale).
fn build_processing_overlay<'a>(
    scaled_width: f32,
    scaled_height: f32,
    spinner_rotation: f32,
    processing_text: String,
) -> Container<'a, Message> {
    let spinner =
        AnimatedSpinner::new(theme::overlay_arrow_light_color(), spinner_rotation).into_element();

    let loading_text = text(processing_text).size(typography::BODY_LG);

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

    container(loading_overlay)
        .width(Length::Fixed(scaled_width))
        .height(Length::Fixed(scaled_height))
        .align_x(Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
}

/// Determines cursor interaction based on current state.
fn determine_cursor_interaction(crop_active: bool, is_dragging: bool) -> mouse::Interaction {
    if crop_active {
        mouse::Interaction::default()
    } else if is_dragging {
        mouse::Interaction::Grabbing
    } else {
        mouse::Interaction::Grab
    }
}

/// Applies the background theme to the canvas surface.
fn apply_background<'a>(
    canvas_content: impl Into<Element<'a, Message>>,
    background_theme: BackgroundTheme,
) -> Element<'a, Message> {
    let surface = container(canvas_content)
        .width(Length::Fill)
        .height(Length::Fill);

    if theme::is_checkerboard(background_theme) {
        checkerboard::wrap(surface)
    } else {
        let bg_color = match background_theme {
            BackgroundTheme::Light => theme::viewer_light_surface_color(),
            BackgroundTheme::Dark => theme::viewer_dark_surface_color(),
            BackgroundTheme::Checkerboard => unreachable!(),
        };

        surface.style(theme::editor_canvas_style(bg_color)).into()
    }
}

pub fn view<'a>(model: &CanvasModel<'a>, ctx: &ViewContext<'a>) -> Element<'a, Message> {
    let background_theme = ctx.background_theme;

    // Clone/copy values needed inside responsive closure
    let image_handle = model.display_image.handle.clone();
    let img_width = model.display_image.width;
    let img_height = model.display_image.height;
    let zoom_scale = model.zoom_scale;

    // Capture overlay state
    let deblur_processing = model.deblur_state.is_processing;
    let upscale_processing = model.upscale_processing;
    let spinner_rotation = model.deblur_state.spinner_rotation;
    let processing_text = if deblur_processing {
        ctx.i18n.tr("image-editor-deblur-processing").clone()
    } else if upscale_processing {
        ctx.i18n.tr("image-editor-upscale-processing").clone()
    } else {
        String::new()
    };
    let is_processing = deblur_processing || upscale_processing;

    let crop_visible = model.crop_state.overlay.visible;
    let crop_x = model.crop_state.x;
    let crop_y = model.crop_state.y;
    let crop_width = model.crop_state.width;
    let crop_height = model.crop_state.height;

    let resize_visible = model.resize_state.overlay.visible;
    let resize_original_width = model.resize_state.overlay.original_width;
    let resize_original_height = model.resize_state.overlay.original_height;
    let resize_width = model.resize_state.width;
    let resize_height = model.resize_state.height;

    // Capture drag state for cursor interaction
    let is_dragging = model.is_dragging;
    let crop_active = model.crop_active;

    // Use responsive to get available size for centering
    let canvas_content = responsive(move |available_size: Size| {
        // Apply zoom scale to image dimensions
        let scaled_width = (img_width as f32 * zoom_scale).round();
        let scaled_height = (img_height as f32 * zoom_scale).round();
        let scaled_size = Size::new(scaled_width, scaled_height);

        // Calculate centering padding
        let centering_padding = calculate_centering_padding(scaled_size, available_size);

        // Render image at zoomed size
        let image_widget = image(image_handle.clone())
            .width(Length::Fixed(scaled_width))
            .height(Length::Fixed(scaled_height));

        let image_with_overlay: Element<'_, Message> = if is_processing {
            let overlay =
                build_processing_overlay(scaled_width, scaled_height, spinner_rotation, processing_text.clone());
            Stack::new().push(image_widget).push(overlay).into()
        } else if crop_visible {
            Stack::new()
                .push(image_widget)
                .push(
                    Canvas::new(CropOverlayRenderer {
                        crop_x,
                        crop_y,
                        crop_width,
                        crop_height,
                        img_width,
                        img_height,
                    })
                    .width(Length::Fill)
                    .height(Length::Fill),
                )
                .into()
        } else if resize_visible {
            Stack::new()
                .push(image_widget)
                .push(
                    Canvas::new(ResizeOverlayRenderer {
                        original_width: resize_original_width,
                        original_height: resize_original_height,
                        new_width: resize_width,
                        new_height: resize_height,
                    })
                    .width(Length::Fill)
                    .height(Length::Fill),
                )
                .into()
        } else {
            image_widget.into()
        };

        // Wrap in container with centering padding, then in scrollable
        let centered_content = Container::new(image_with_overlay).padding(centering_padding);

        scrollable_canvas::scrollable_canvas(centered_content.into(), scaled_width, scaled_height)
    });

    let cursor_interaction = determine_cursor_interaction(crop_active, is_dragging);

    // Wrap canvas in mouse_area for cursor feedback and tracking
    let canvas_with_cursor = mouse_area(canvas_content)
        .interaction(cursor_interaction)
        .on_move(|position| Message::Canvas(CanvasMessage::CursorMoved { position }))
        .on_exit(Message::Canvas(CanvasMessage::CursorLeft));

    apply_background(canvas_with_cursor, background_theme)
}
