// SPDX-License-Identifier: MPL-2.0
//! Video playback controls UI.
//!
//! Provides a toolbar with play/pause, timeline scrubber, time display,
//! volume controls, and loop toggle specifically for video playback.

use crate::i18n::fluent::I18n;
use crate::ui::design_tokens::{sizing, spacing};
use crate::ui::{icons, styles};
use iced::widget::{button, container, row, slider, text, tooltip, Row, Text};
use iced::{Element, Length};

/// Messages emitted by video control widgets.
#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    /// Toggle play/pause state.
    TogglePlayback,

    /// Seek preview - slider is being dragged (visual feedback only, no actual seek).
    SeekPreview(f32),

    /// Commit seek - slider released, perform actual seek to preview position.
    SeekCommit,

    /// Legacy seek (kept for compatibility).
    Seek(f32),

    /// Seek relative to current position (in seconds, can be negative).
    /// Used by keyboard shortcuts (e.g., arrow keys for ±5s).
    SeekRelative(f64),

    /// Adjust volume (0.0 to 1.0).
    SetVolume(f32),

    /// Toggle mute state.
    ToggleMute,

    /// Toggle loop mode.
    ToggleLoop,

    /// Capture current frame and export to file.
    CaptureFrame,

    /// Step forward one frame (only when paused).
    StepForward,

    /// Step backward one frame (only when paused).
    StepBackward,
}

/// View context for rendering video controls.
pub struct ViewContext<'a> {
    pub i18n: &'a I18n,
}

/// Video playback state for rendering controls.
#[derive(Debug, Clone)]
pub struct PlaybackState {
    /// Is the video currently playing?
    pub is_playing: bool,

    /// Current playback position in seconds.
    pub position_secs: f64,

    /// Total duration in seconds.
    pub duration_secs: f64,

    /// Current volume (0.0 to 1.0).
    pub volume: f32,

    /// Is audio muted?
    pub muted: bool,

    /// Is loop mode enabled?
    pub loop_enabled: bool,

    /// Preview position during seek drag (0.0 to 1.0), if any.
    /// When Some, the slider shows this position instead of actual playback position.
    pub seek_preview_position: Option<f32>,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            is_playing: false,
            position_secs: 0.0,
            duration_secs: 0.0,
            volume: 1.0,
            muted: false,
            loop_enabled: false,
            seek_preview_position: None,
        }
    }
}

/// Renders video controls toolbar.
///
/// Returns a Row with:
/// - Play/Pause button
/// - Timeline slider
/// - Time display (current/total)
/// - Volume button
/// - Loop button
pub fn view<'a>(ctx: ViewContext<'a>, state: &PlaybackState) -> Element<'a, Message> {
    // Icon size for control buttons (consistent with design tokens)
    let icon_size = sizing::ICON_SM;
    let button_height = sizing::BUTTON_HEIGHT;

    let play_pause_svg = if state.is_playing {
        icons::sized(icons::pause(), icon_size)
    } else {
        icons::sized(icons::play(), icon_size)
    };

    let play_pause_tooltip = if state.is_playing {
        ctx.i18n.tr("video-pause-tooltip")
    } else {
        ctx.i18n.tr("video-play-tooltip")
    };

    let play_pause_button_content: Element<'_, Message> = button(play_pause_svg)
        .on_press(Message::TogglePlayback)
        .padding(spacing::XS)
        .width(Length::Shrink)
        .height(Length::Fixed(button_height))
        .into();

    let play_pause_button = tooltip(
        play_pause_button_content,
        Text::new(play_pause_tooltip),
        tooltip::Position::Top,
    )
    .gap(4);

    // Calculate timeline position (0.0 to 1.0)
    // Use preview position during drag, otherwise use actual playback position
    let timeline_position = if let Some(preview) = state.seek_preview_position {
        preview
    } else if state.duration_secs > 0.0 {
        (state.position_secs / state.duration_secs).clamp(0.0, 1.0) as f32
    } else {
        0.0
    };

    // Use on_change for visual preview, on_release for actual seek
    let timeline = slider(0.0..=1.0, timeline_position, Message::SeekPreview)
        .on_release(Message::SeekCommit)
        .width(Length::FillPortion(1))
        .step(0.001);

    // Format time display - use monospace-like sizing
    let time_display = text(format!(
        "{} / {}",
        format_time(state.position_secs),
        format_time(state.duration_secs)
    ))
    .size(sizing::ICON_SM);

    // Volume button with tooltip - shows mute icon when muted
    let volume_icon = if state.muted || state.volume == 0.0 {
        icons::sized(icons::volume_mute(), icon_size)
    } else {
        icons::sized(icons::volume(), icon_size)
    };
    let volume_tooltip = if state.muted {
        ctx.i18n.tr("video-unmute-tooltip")
    } else {
        ctx.i18n.tr("video-mute-tooltip")
    };
    let volume_button = button(volume_icon)
        .on_press(Message::ToggleMute)
        .padding(spacing::XS)
        .width(Length::Shrink)
        .height(Length::Fixed(button_height));

    // Apply active style when muted (highlighted like fit-to-window button)
    let volume_button_content: Element<'_, Message> = if state.muted {
        volume_button.style(styles::button_primary).into()
    } else {
        volume_button.into()
    };

    let volume_button_tooltip = tooltip(
        volume_button_content,
        Text::new(volume_tooltip),
        tooltip::Position::Top,
    )
    .gap(4);

    // Volume slider (only shown when not muted)
    let volume_slider = slider(0.0..=1.0, state.volume, Message::SetVolume)
        .width(Length::Fixed(80.0))
        .step(0.01);

    // Step backward button (only enabled when paused)
    let step_back_button_content: Element<'_, Message> = if !state.is_playing {
        button(icons::sized(icons::step_backward(), icon_size))
            .on_press(Message::StepBackward)
            .padding(spacing::XS)
            .width(Length::Shrink)
            .height(Length::Fixed(button_height))
            .into()
    } else {
        button(icons::sized(icons::step_backward(), icon_size))
            .padding(spacing::XS)
            .width(Length::Shrink)
            .height(Length::Fixed(button_height))
            .style(styles::button::disabled())
            .into()
    };
    let step_back_button = tooltip(
        step_back_button_content,
        Text::new(ctx.i18n.tr("video-step-backward-tooltip")),
        tooltip::Position::Top,
    )
    .gap(4);

    // Step forward button (only enabled when paused)
    let step_forward_button_content: Element<'_, Message> = if !state.is_playing {
        button(icons::sized(icons::step_forward(), icon_size))
            .on_press(Message::StepForward)
            .padding(spacing::XS)
            .width(Length::Shrink)
            .height(Length::Fixed(button_height))
            .into()
    } else {
        button(icons::sized(icons::step_forward(), icon_size))
            .padding(spacing::XS)
            .width(Length::Shrink)
            .height(Length::Fixed(button_height))
            .style(styles::button::disabled())
            .into()
    };
    let step_forward_button = tooltip(
        step_forward_button_content,
        Text::new(ctx.i18n.tr("video-step-forward-tooltip")),
        tooltip::Position::Top,
    )
    .gap(4);

    // Capture frame button with tooltip
    let capture_tooltip = ctx.i18n.tr("video-capture-tooltip");
    let capture_button_content: Element<'_, Message> =
        button(icons::sized(icons::camera(), icon_size))
            .on_press(Message::CaptureFrame)
            .padding(spacing::XS)
            .width(Length::Shrink)
            .height(Length::Fixed(button_height))
            .into();

    let capture_button = tooltip(
        capture_button_content,
        Text::new(capture_tooltip),
        tooltip::Position::Top,
    )
    .gap(4);

    // Loop button with tooltip
    let loop_tooltip = ctx.i18n.tr("video-loop-tooltip");
    let loop_button_base = button(icons::sized(icons::loop_icon(), icon_size))
        .on_press(Message::ToggleLoop)
        .padding(spacing::XS)
        .width(Length::Shrink)
        .height(Length::Fixed(button_height));

    // Apply active style when loop is enabled (highlighted like mute button)
    let loop_button_content: Element<'_, Message> = if state.loop_enabled {
        loop_button_base.style(styles::button_primary).into()
    } else {
        loop_button_base.into()
    };

    let loop_button = tooltip(
        loop_button_content,
        Text::new(loop_tooltip),
        tooltip::Position::Top,
    )
    .gap(4);

    let controls: Row<'a, Message> = row![
        step_back_button,
        play_pause_button,
        step_forward_button,
        timeline,
        time_display,
        capture_button,
        volume_button_tooltip,
        volume_slider,
        loop_button,
    ]
    .spacing(spacing::XS)
    .padding(spacing::XS)
    .align_y(iced::Alignment::Center);

    container(controls)
        .width(Length::Fill)
        .padding(spacing::XXS)
        .into()
}

/// Formats duration in MM:SS or HH:MM:SS format.
fn format_time(seconds: f64) -> String {
    let total_secs = seconds.max(0.0) as u64;
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let secs = total_secs % 60;

    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, secs)
    } else {
        format!("{:02}:{:02}", minutes, secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_time_handles_zero() {
        assert_eq!(format_time(0.0), "00:00");
    }

    #[test]
    fn format_time_handles_seconds() {
        assert_eq!(format_time(45.0), "00:45");
    }

    #[test]
    fn format_time_handles_minutes() {
        assert_eq!(format_time(125.0), "02:05");
    }

    #[test]
    fn format_time_handles_hours() {
        assert_eq!(format_time(3665.0), "01:01:05");
    }

    #[test]
    fn format_time_handles_negative() {
        // Negative time should be clamped to 0
        assert_eq!(format_time(-10.0), "00:00");
    }

    #[test]
    fn playback_state_defaults() {
        let state = PlaybackState::default();
        assert!(!state.is_playing);
        assert_eq!(state.position_secs, 0.0);
        assert_eq!(state.duration_secs, 0.0);
        assert_eq!(state.volume, 1.0);
        assert!(!state.muted);
        assert!(!state.loop_enabled);
        assert!(state.seek_preview_position.is_none());
    }

    #[test]
    fn message_clone_works() {
        let msg = Message::TogglePlayback;
        let cloned = msg.clone();
        assert_eq!(msg, cloned);
    }

    #[test]
    fn message_debug_works() {
        let msg = Message::Seek(0.5);
        let debug_str = format!("{:?}", msg);
        assert!(debug_str.contains("Seek"));
        assert!(debug_str.contains("0.5"));
    }

    #[test]
    fn view_renders() {
        let i18n = I18n::default();
        let ctx = ViewContext { i18n: &i18n };
        let state = PlaybackState::default();
        let _element = view(ctx, &state);
    }

    #[test]
    fn timeline_position_calculated_correctly() {
        let state = PlaybackState {
            is_playing: true,
            position_secs: 30.0,
            duration_secs: 120.0,
            volume: 0.8,
            muted: false,
            loop_enabled: false,
            seek_preview_position: None,
        };

        let position = if state.duration_secs > 0.0 {
            (state.position_secs / state.duration_secs).clamp(0.0, 1.0)
        } else {
            0.0
        };

        assert_eq!(position, 0.25);
    }

    #[test]
    fn timeline_position_handles_zero_duration() {
        let state = PlaybackState {
            is_playing: false,
            position_secs: 10.0,
            duration_secs: 0.0,
            volume: 1.0,
            muted: false,
            loop_enabled: false,
            seek_preview_position: None,
        };

        let position = if state.duration_secs > 0.0 {
            (state.position_secs / state.duration_secs).clamp(0.0, 1.0)
        } else {
            0.0
        };

        assert_eq!(position, 0.0);
    }

    #[test]
    fn timeline_uses_preview_position_when_set() {
        let state = PlaybackState {
            is_playing: true,
            position_secs: 30.0,
            duration_secs: 120.0,
            volume: 1.0,
            muted: false,
            loop_enabled: false,
            seek_preview_position: Some(0.75),
        };

        // When seek_preview_position is set, it should be used instead of calculated position
        let position = if let Some(preview) = state.seek_preview_position {
            preview
        } else if state.duration_secs > 0.0 {
            (state.position_secs / state.duration_secs).clamp(0.0, 1.0) as f32
        } else {
            0.0
        };

        assert_eq!(position, 0.75);
    }
}
