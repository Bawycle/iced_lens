// SPDX-License-Identifier: MPL-2.0
//! Viewer pane that renders the image inside the scrollable area with proper
//! background, cursor interaction, and position indicator.

use crate::config::BackgroundTheme;
use crate::media::MediaData;
use crate::ui::components::checkerboard;
use crate::ui::design_tokens::{opacity, radius, sizing, spacing, typography};
use crate::ui::icons;
use crate::ui::styles;
use crate::ui::theme;
use crate::ui::viewer::{component::Message, HudIconKind, HudLine};
use crate::ui::widgets::{wheel_blocking_scrollable::wheel_blocking_scrollable, AnimatedSpinner};
use iced::mouse;
use iced::widget::{
    button, mouse_area, responsive, Column, Container, Row, Scrollable, Stack, Text,
};
use iced::{
    alignment::{Horizontal, Vertical},
    widget::scrollable::{Direction, Scrollbar, Viewport},
    widget::Id,
    Background, Element, Length, Padding, Size, Theme,
};

pub struct ViewContext<'a> {
    pub background_theme: BackgroundTheme,
    pub hud_lines: Vec<HudLine>,
    pub scrollable_id: &'static str,
    pub i18n: &'a crate::i18n::fluent::I18n,
}

pub struct ViewModel<'a> {
    pub media: &'a MediaData,
    pub zoom_percent: f32,
    /// Manual zoom percentage (used when fit_to_window is disabled).
    pub manual_zoom_percent: f32,
    /// Whether fit-to-window mode is enabled.
    pub fit_to_window: bool,
    pub is_dragging: bool,
    pub cursor_over_media: bool,
    pub arrows_visible: bool,
    pub overlay_visible: bool,
    pub has_next: bool,
    pub has_previous: bool,
    pub at_first: bool,
    pub at_last: bool,
    pub current_index: Option<usize>,
    pub total_count: usize,
    pub position_counter_visible: bool,
    pub hud_visible: bool,
    pub video_shader: Option<&'a crate::ui::widgets::VideoShader<super::component::Message>>,
    pub is_video_playing: bool,
    pub is_loading_media: bool,
    pub spinner_rotation: f32,
    pub video_error: Option<&'a str>,
    /// Whether metadata editor has unsaved changes (disables navigation).
    pub metadata_editor_has_changes: bool,
}

pub fn view<'a>(ctx: ViewContext<'a>, model: ViewModel<'a>) -> Element<'a, Message> {
    // Use responsive widget to get the available size and calculate fit-to-window zoom
    responsive(move |available_size: Size| view_inner(&ctx, &model, available_size)).into()
}

/// Calculate the zoom percentage needed to fit media within available space.
fn calculate_fit_zoom(media_width: u32, media_height: u32, available: Size) -> f32 {
    if media_width == 0 || media_height == 0 || available.width <= 0.0 || available.height <= 0.0 {
        return crate::ui::state::zoom::DEFAULT_ZOOM_PERCENT;
    }

    let scale_x = available.width / media_width as f32;
    let scale_y = available.height / media_height as f32;
    let scale = scale_x.min(scale_y);

    if !scale.is_finite() || scale <= 0.0 {
        return crate::ui::state::zoom::DEFAULT_ZOOM_PERCENT;
    }

    crate::ui::state::zoom::clamp_zoom(scale * 100.0)
}

/// Calculate padding to center media within available space.
fn calculate_centering_padding(media_size: Size, available: Size) -> Padding {
    let horizontal = ((available.width - media_size.width) / 2.0).max(0.0);
    let vertical = ((available.height - media_size.height) / 2.0).max(0.0);

    Padding {
        top: vertical,
        right: horizontal,
        bottom: vertical,
        left: horizontal,
    }
}

fn view_inner<'a>(
    ctx: &ViewContext<'a>,
    model: &ViewModel<'a>,
    available_size: Size,
) -> Element<'a, Message> {
    // Calculate effective zoom: use fit-to-window calculation or manual zoom
    let effective_zoom = if model.fit_to_window {
        calculate_fit_zoom(model.media.width(), model.media.height(), available_size)
    } else {
        model.manual_zoom_percent
    };

    // Calculate scaled media size
    let scale = effective_zoom / 100.0;
    let scaled_width = model.media.width() as f32 * scale;
    let scaled_height = model.media.height() as f32 * scale;
    let scaled_size = Size::new(scaled_width, scaled_height);

    // Calculate padding based on current available size (from responsive widget)
    // This ensures proper centering even when layout changes
    let effective_padding = calculate_centering_padding(scaled_size, available_size);

    // Determine arrow colors based on background theme for optimal visibility
    // Following UX best practices: semi-transparent backgrounds with strong shadows
    let (arrow_text_color, arrow_bg_alpha_normal, arrow_bg_alpha_hover) = match ctx.background_theme
    {
        BackgroundTheme::Light => {
            // Light background: dark arrows with light background on hover
            (theme::overlay_arrow_dark_color(), 0.0, 0.2)
        }
        BackgroundTheme::Dark | BackgroundTheme::Checkerboard => {
            // Dark/checkerboard: white arrows with dark background on hover
            (theme::overlay_arrow_light_color(), 0.0, 0.5)
        }
    };

    // Use video shader if it has a frame (playing OR paused with frame),
    // otherwise show static media (image or video thumbnail before playback starts)
    let media_viewer = if let Some(shader) = model.video_shader {
        if shader.has_frame() {
            // Show the shader frame (whether playing or paused)
            shader.view()
        } else {
            // No frame yet, show thumbnail
            super::view_media(model.media, effective_zoom)
        }
    } else {
        // Not a video or no shader, show static media
        super::view_media(model.media, effective_zoom)
    };

    let media_container = Container::new(media_viewer).padding(effective_padding);

    let scrollable = Scrollable::new(media_container)
        .id(Id::new(ctx.scrollable_id))
        .width(Length::Fill)
        .height(Length::Fill)
        .direction(Direction::Both {
            vertical: Scrollbar::hidden(),
            horizontal: Scrollbar::hidden(),
        })
        .on_scroll(|viewport: Viewport| {
            let bounds = viewport.bounds();
            Message::ViewportChanged {
                bounds,
                offset: viewport.absolute_offset(),
            }
        });

    let wheel_blocked_scrollable = wheel_blocking_scrollable(scrollable);

    let cursor_interaction = if model.is_dragging {
        mouse::Interaction::Grabbing
    } else if model.cursor_over_media {
        mouse::Interaction::Grab
    } else {
        mouse::Interaction::default()
    };

    let scrollable_with_cursor =
        mouse_area(wheel_blocked_scrollable).interaction(cursor_interaction);

    let scrollable_container = Container::new(scrollable_with_cursor)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center);

    let base_surface: Element<'_, Message> = match ctx.background_theme {
        BackgroundTheme::Light => {
            let color = theme::viewer_light_surface_color();
            scrollable_container
                .style(move |_theme: &Theme| iced::widget::container::Style {
                    background: Some(Background::Color(color)),
                    ..Default::default()
                })
                .into()
        }
        BackgroundTheme::Dark => {
            let color = theme::viewer_dark_surface_color();
            scrollable_container
                .style(move |_theme: &Theme| iced::widget::container::Style {
                    background: Some(Background::Color(color)),
                    ..Default::default()
                })
                .into()
        }
        BackgroundTheme::Checkerboard => checkerboard::wrap(scrollable_container),
    };

    let mut stack = Stack::new().push(base_surface);

    // Add navigation arrows if visible

    // Navigation is disabled when metadata editor has unsaved changes
    let nav_enabled = !model.metadata_editor_has_changes;

    if model.arrows_visible {
        if model.has_previous {
            // Show loop icon at boundaries to indicate wrap-around behavior
            // Choose icon color based on background for optimal visibility
            let button_content: Element<'_, Message> = if model.at_first {
                let loop_icon = match ctx.background_theme {
                    BackgroundTheme::Light => icons::sized(icons::loop_icon(), 16.0),
                    BackgroundTheme::Dark | BackgroundTheme::Checkerboard => {
                        icons::sized(icons::overlay::loop_icon(), 16.0)
                    }
                };
                Row::new()
                    .spacing(spacing::XS)
                    .align_y(Vertical::Center)
                    .push(loop_icon)
                    .push(Text::new("◀").size(typography::TITLE_MD))
                    .into()
            } else {
                Text::new("◀").size(typography::TITLE_LG).into()
            };
            let left_arrow =
                button(button_content)
                    .padding(spacing::SM)
                    .style(styles::button_overlay(
                        arrow_text_color,
                        arrow_bg_alpha_normal,
                        arrow_bg_alpha_hover,
                    ));
            let left_arrow = if nav_enabled {
                left_arrow.on_press(Message::NavigatePrevious)
            } else {
                left_arrow
            };

            // Create a clickable zone that contains the button
            // The zone has a minimum width but can expand to fit the button content
            let left_zone = Container::new(left_arrow)
                .height(Length::Fill)
                .padding(spacing::MD)
                .align_x(Horizontal::Left)
                .align_y(Vertical::Center);

            // Wrap in mouse_area to capture clicks outside the button but within the zone
            let left_zone_clickable = mouse_area(left_zone);
            let left_zone_clickable = if nav_enabled {
                left_zone_clickable.on_release(Message::NavigatePrevious)
            } else {
                left_zone_clickable
            };

            // Outer container fills width so content has room to display fully
            stack = stack.push(
                Container::new(left_zone_clickable)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Horizontal::Left),
            );
        }

        if model.has_next {
            // Show loop icon at boundaries to indicate wrap-around behavior
            // Choose icon color based on background for optimal visibility
            let button_content: Element<'_, Message> = if model.at_last {
                let loop_icon = match ctx.background_theme {
                    BackgroundTheme::Light => icons::sized(icons::loop_icon(), 16.0),
                    BackgroundTheme::Dark | BackgroundTheme::Checkerboard => {
                        icons::sized(icons::overlay::loop_icon(), 16.0)
                    }
                };
                Row::new()
                    .spacing(spacing::XS)
                    .align_y(Vertical::Center)
                    .push(Text::new("▶").size(typography::TITLE_MD))
                    .push(loop_icon)
                    .into()
            } else {
                Text::new("▶").size(typography::TITLE_LG).into()
            };
            let right_arrow =
                button(button_content)
                    .padding(spacing::SM)
                    .style(styles::button_overlay(
                        arrow_text_color,
                        arrow_bg_alpha_normal,
                        arrow_bg_alpha_hover,
                    ));
            let right_arrow = if nav_enabled {
                right_arrow.on_press(Message::NavigateNext)
            } else {
                right_arrow
            };

            // Create a clickable zone that contains the button
            // The zone has a minimum width but can expand to fit the button content
            let right_zone = Container::new(right_arrow)
                .height(Length::Fill)
                .padding(spacing::MD)
                .align_x(Horizontal::Right)
                .align_y(Vertical::Center);

            // Wrap in mouse_area to capture clicks outside the button but within the zone
            let right_zone_clickable = mouse_area(right_zone);
            let right_zone_clickable = if nav_enabled {
                right_zone_clickable.on_release(Message::NavigateNext)
            } else {
                right_zone_clickable
            };

            // Outer container fills width so align_x positions content at right edge
            stack = stack.push(
                Container::new(right_zone_clickable)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Horizontal::Right),
            );
        }
    }

    // Add loading overlay if media is being loaded
    if model.is_loading_media {
        let spinner =
            AnimatedSpinner::new(theme::overlay_arrow_light_color(), model.spinner_rotation)
                .into_element();

        let loading_text = Text::new(ctx.i18n.tr("media-loading")).size(sizing::ICON_SM);

        let loading_content = Column::new()
            .spacing(spacing::SM)
            .align_x(Horizontal::Center)
            .push(spinner)
            .push(loading_text);

        let loading_overlay =
            Container::new(loading_content)
                .padding(spacing::MD)
                .style(move |_theme: &Theme| iced::widget::container::Style {
                    background: Some(Background::Color(iced::Color {
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

        stack = stack.push(
            Container::new(loading_overlay)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center),
        );
    }

    // Add video error overlay if there's an error
    if let Some(error_msg) = model.video_error {
        use crate::error::VideoError;

        // Parse error message to get appropriate i18n key and user-friendly message
        let video_error = VideoError::from_message(error_msg);
        let args = video_error.i18n_args();
        let args_refs: Vec<(&str, &str)> = args.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let error_text = ctx.i18n.tr_with_args(video_error.i18n_key(), &args_refs);
        let heading = ctx.i18n.tr("error-load-video-heading");

        let error_icon = icons::sized(icons::overlay::warning(), 32.0);

        let error_content = Column::new()
            .spacing(spacing::SM)
            .align_x(Horizontal::Center)
            .push(error_icon)
            .push(Text::new(heading).size(sizing::ICON_MD))
            .push(Text::new(error_text).size(sizing::ICON_SM));

        let error_overlay = Container::new(error_content)
            .padding(spacing::LG)
            .max_width(400.0)
            .style(move |_theme: &Theme| iced::widget::container::Style {
                background: Some(Background::Color(iced::Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: opacity::OVERLAY_STRONG,
                })),
                border: iced::Border {
                    radius: radius::MD.into(),
                    ..Default::default()
                },
                text_color: Some(theme::overlay_arrow_light_color()),
                ..Default::default()
            });

        stack = stack.push(
            Container::new(error_overlay)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center),
        );
    }

    // Add play/pause button overlay for videos
    if let MediaData::Video(_) = model.media {
        // Show play button when paused and not loading and no error
        if model.overlay_visible
            && !model.is_loading_media
            && !model.is_video_playing
            && model.video_error.is_none()
        {
            let play_icon = icons::sized(icons::overlay::play(), 32.0);

            let play_button = button(play_icon)
                .on_press(Message::InitiatePlayback)
                .style(styles::button::video_play_overlay())
                .width(Length::Fixed(64.0))
                .height(Length::Fixed(64.0));

            stack = stack.push(
                Container::new(play_button)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center),
            );
        }
        // Show pause button when playing and overlay is visible and no error
        else if model.overlay_visible && model.is_video_playing && model.video_error.is_none() {
            let pause_icon = icons::sized(icons::overlay::pause(), 32.0);

            // This should send a message to pause the video
            // For now, we'll reuse InitiatePlayback which should toggle
            let pause_button = button(pause_icon)
                .on_press(Message::InitiatePlayback)
                .style(styles::button::video_play_overlay())
                .width(Length::Fixed(64.0))
                .height(Length::Fixed(64.0));

            stack = stack.push(
                Container::new(pause_button)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center),
            );
        }
    }

    // Add HUD indicator if present and visible
    if model.hud_visible && !ctx.hud_lines.is_empty() {
        const HUD_ICON_SIZE: f32 = 14.0;

        let mut hud_column: Column<'_, Message> = Column::new().spacing(spacing::XXS);
        for hud_line in &ctx.hud_lines {
            let icon = match hud_line.icon {
                HudIconKind::Position => icons::overlay::crosshair(),
                HudIconKind::Zoom => icons::overlay::magnifier(),
                HudIconKind::Video { has_audio } => {
                    if has_audio {
                        icons::overlay::video_camera_audio()
                    } else {
                        icons::overlay::video_camera()
                    }
                }
            };

            let styled_icon = icons::sized(icon, HUD_ICON_SIZE);

            let line_row = Row::new()
                .spacing(spacing::XXS)
                .align_y(Vertical::Center)
                .push(styled_icon)
                .push(Text::new(hud_line.text.clone()).size(typography::CAPTION));

            hud_column = hud_column.push(line_row);
        }

        let indicator = Container::new(hud_column)
            .padding(spacing::XXS)
            .style(styles::overlay::indicator(4.0));

        stack = stack.push(
            Container::new(indicator)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(spacing::SM)
                .align_x(Horizontal::Right)
                .align_y(Vertical::Bottom),
        );
    }

    // Add position counter at bottom center if there are multiple images and it should be visible
    if model.position_counter_visible && model.total_count > 1 {
        if let Some(current) = model.current_index {
            let position_text = format!("{}/{}", current + 1, model.total_count);
            let position_indicator =
                Container::new(Text::new(position_text).size(typography::BODY))
                    .padding(Padding {
                        top: spacing::XXS,
                        right: spacing::XS,
                        bottom: spacing::XXS,
                        left: spacing::XS,
                    })
                    .style(styles::overlay::indicator(12.0));

            stack = stack.push(
                Container::new(position_indicator)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(spacing::SM)
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Bottom),
            );
        }
    }

    stack.into()
}
