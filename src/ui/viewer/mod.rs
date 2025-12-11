// SPDX-License-Identifier: MPL-2.0
//! Image viewer module responsible for rendering loaded images and related UI.

pub mod component;
pub mod controls;
pub mod pane;
pub mod shared_styles;
pub mod state;
pub mod video_controls;

use self::component::Message;
use crate::i18n::fluent::I18n;
use crate::media::MediaData;
use crate::ui::components::error_display::{centered_error_view, ErrorDisplay, ErrorSeverity};
use crate::ui::design_tokens::{sizing, spacing};
use crate::ui::state::ZoomState;
use crate::ui::styles;
use crate::ui::theme;
use crate::ui::widgets::AnimatedSpinner;
use iced::widget::{button, Column, Container, Image, Row, Stack, Text};
use iced::{alignment, Element, Length};

/// Kind of icon to display for a HUD line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HudIconKind {
    Position,
    Zoom,
    Video { has_audio: bool },
}

/// A single HUD entry combining an icon kind and descriptive text.
#[derive(Debug, Clone)]
pub struct HudLine {
    pub icon: HudIconKind,
    pub text: String,
}

pub fn view_media(media_data: &MediaData, zoom_percent: f32) -> Element<'_, Message> {
    // Extract the image handle (either from Image or Video thumbnail)
    let (handle, width, height) = match media_data {
        MediaData::Image(image_data) => (
            image_data.handle.clone(),
            image_data.width,
            image_data.height,
        ),
        MediaData::Video(video_data) => (
            video_data.thumbnail.handle.clone(),
            video_data.thumbnail.width,
            video_data.thumbnail.height,
        ),
    };

    let scale = (zoom_percent / 100.0).max(0.01);
    let scaled_width = (width as f32 * scale).max(1.0);
    let scaled_height = (height as f32 * scale).max(1.0);

    Image::new(handle)
        .width(Length::Fixed(scaled_width))
        .height(Length::Fixed(scaled_height))
        .into()
}

pub struct ViewContext<'a> {
    pub i18n: &'a I18n,
    pub error: Option<ErrorContext<'a>>,
    pub image: Option<ImageContext<'a>>,
    pub is_loading: bool,
    pub spinner_rotation: f32,
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
    /// Effective fit-to-window state (may differ from zoom.fit_to_window for videos).
    pub effective_fit_to_window: bool,
    pub pane_context: pane::ViewContext<'a>,
    pub pane_model: pane::ViewModel<'a>,
    pub controls_visible: bool,
    pub is_fullscreen: bool,
    pub video_playback_state: Option<video_controls::PlaybackState>,
    /// True if the current media is a video (used to disable Edit button).
    pub is_video: bool,
}

pub fn view(ctx: ViewContext<'_>) -> Element<'_, Message> {
    if let Some(error) = ctx.error {
        return error_view(ctx.i18n, error);
    }

    if let Some(image) = ctx.image {
        return image_view(image);
    }

    // Show loading spinner if loading
    if ctx.is_loading {
        return loading_view(ctx.i18n, ctx.spinner_rotation);
    }

    Text::new(ctx.i18n.tr("hello-message")).into()
}

fn error_view<'a>(i18n: &'a I18n, error: ErrorContext<'a>) -> Element<'a, Message> {
    let error_display = ErrorDisplay::new(ErrorSeverity::Error)
        .title(i18n.tr("error-load-image-heading"))
        .message(error.friendly_text)
        .details(error.details)
        .details_visible(error.show_details)
        .on_toggle_details(Message::ToggleErrorDetails)
        .details_labels(
            i18n.tr("error-details-show"),
            i18n.tr("error-details-hide"),
            i18n.tr("error-details-technical-heading"),
        );

    centered_error_view(error_display)
}

fn loading_view<'a>(i18n: &'a I18n, rotation: f32) -> Element<'a, Message> {
    let spinner = AnimatedSpinner::new(theme::overlay_arrow_light_color(), rotation).into_element();

    let loading_text = Text::new(i18n.tr("media-loading")).size(sizing::ICON_SM);

    let loading_content = Column::new()
        .spacing(spacing::SM)
        .align_x(alignment::Horizontal::Center)
        .push(spinner)
        .push(loading_text);

    Container::new(loading_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .into()
}

fn image_view(ctx: ImageContext<'_>) -> Element<'_, Message> {
    let pane_view = pane::view(ctx.pane_context, ctx.pane_model);

    // Build video controls if video is playing/paused
    let video_controls_view = if let Some(ref playback_state) = ctx.video_playback_state {
        let video_ctx = video_controls::ViewContext { i18n: ctx.i18n };
        Some(video_controls::view(video_ctx, playback_state).map(Message::VideoControls))
    } else {
        None
    };

    // Fullscreen mode: overlay controls on top of pane
    if ctx.is_fullscreen {
        let mut stack = Stack::new().width(Length::Fill).height(Length::Fill);

        // Layer 1: Pane (image + navigation arrows)
        stack = stack.push(pane_view);

        // Layer 2: Overlay controls (if visible)
        if ctx.controls_visible {
            let controls_view =
                controls::view(ctx.controls_context, ctx.zoom, ctx.effective_fit_to_window)
                    .map(Message::Controls);

            let mut controls_column = Column::new().spacing(8);
            controls_column = controls_column.push(controls_view);

            // Add video controls if video is playing/paused
            if let Some(ref video_playback_state) = ctx.video_playback_state {
                let video_ctx = video_controls::ViewContext { i18n: ctx.i18n };
                let video_controls_view = video_controls::view(video_ctx, video_playback_state)
                    .map(Message::VideoControls);
                controls_column = controls_column.push(video_controls_view);
            }

            let overlay_container = Container::new(controls_column)
                .width(Length::Fill)
                .padding(10)
                .style(styles::overlay::controls_container);

            stack = stack.push(overlay_container);
        }

        stack.into()
    } else {
        // Windowed mode: normal column layout
        let mut column = Column::new()
            .spacing(16)
            .width(Length::Fill)
            .height(Length::Fill);

        // Add top navigation bar with Settings and Edit buttons
        if ctx.controls_visible {
            let mut top_bar = Row::new().spacing(10).padding(10);

            let settings_button = button(Text::new(ctx.i18n.tr("open-settings-button")))
                .on_press(Message::OpenSettings);
            top_bar = top_bar.push(settings_button);

            // Edit button is disabled for videos (editing not supported in v0.2)
            let edit_button = if ctx.is_video {
                button(Text::new("✏ Edit")).style(styles::button::disabled())
            } else {
                button(Text::new("✏ Edit")).on_press(Message::EnterEditor)
            };
            top_bar = top_bar.push(edit_button);

            column = column.push(
                Container::new(top_bar)
                    .width(Length::Fill)
                    .align_x(alignment::Horizontal::Left)
                    .style(styles::editor::toolbar),
            );

            let controls_view =
                controls::view(ctx.controls_context, ctx.zoom, ctx.effective_fit_to_window)
                    .map(Message::Controls);
            column = column.push(controls_view);

            // Add video controls if video is playing/paused
            if let Some(video_controls) = video_controls_view {
                column = column.push(video_controls);
            }
        }

        column.push(pane_view).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::{ImageData, MediaData};
    use iced::widget::image::Handle;

    #[test]
    fn view_media_produces_element() {
        let pixels = vec![0_u8, 0, 0, 255];
        let image_data = ImageData {
            handle: Handle::from_rgba(1, 1, pixels),
            width: 1,
            height: 1,
        };
        let media_data = MediaData::Image(image_data);

        let _element = view_media(&media_data, 100.0);
        // Smoke test to ensure rendering succeeds.
    }
}
