// SPDX-License-Identifier: MPL-2.0
//! Video playback controls UI.
//!
//! Provides a toolbar with play/pause, timeline scrubber, time display,
//! volume controls, and loop toggle specifically for video playback.

use crate::config;
use crate::i18n::fluent::I18n;
use crate::ui::design_tokens::{sizing, spacing};
use crate::ui::{action_icons, icons, styles};
use crate::video_player::Volume;
use iced::widget::{button, column, container, row, slider, text, tooltip, Column, Row, Space};
use iced::{Element, Length, Theme};

/// Helper to create a styled tooltip positioned above the element.
fn tip<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    text: impl Into<String>,
) -> tooltip::Tooltip<'a, Message, Theme, iced::Renderer> {
    styles::tooltip::styled(content, text, tooltip::Position::Top)
}

/// Slider step in seconds (1ms precision).
/// f64 has ~15 significant digits, so even for 24h videos (86400s),
/// we have plenty of precision for millisecond accuracy.
const SLIDER_STEP_SECS: f64 = 0.001;

/// Messages emitted by video control widgets.
#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    /// Toggle play/pause state.
    TogglePlayback,

    /// Seek preview - slider is being dragged (visual feedback only, no actual seek).
    /// Position in seconds.
    SeekPreview(f64),

    /// Commit seek - slider released, perform actual seek to preview position.
    SeekCommit,

    /// Seek relative to current position (in seconds, can be negative).
    /// Used by keyboard shortcuts (e.g., arrow keys for ±5s).
    SeekRelative(f64),

    /// Adjust volume (guaranteed to be within 0.0–1.0 by Volume type).
    SetVolume(Volume),

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

    /// Toggle the overflow menu (advanced controls).
    ToggleOverflowMenu,

    /// Increase playback speed to next preset.
    IncreasePlaybackSpeed,

    /// Decrease playback speed to previous preset.
    DecreasePlaybackSpeed,
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

    /// Current volume (0.0 to 1.5, where 1.0 = 100%).
    pub volume: f32,

    /// Is audio muted?
    pub muted: bool,

    /// Is loop mode enabled?
    pub loop_enabled: bool,

    /// Preview position during seek drag in seconds, if any.
    /// When Some, the slider shows this position instead of actual playback position.
    pub seek_preview_position: Option<f64>,

    /// Is the overflow menu (advanced controls) open?
    pub overflow_menu_open: bool,

    /// Can step backward (in stepping mode with frame history available)?
    /// When false, the step backward button is disabled.
    pub can_step_backward: bool,

    /// Can step forward (paused and not at end of video)?
    /// When false, the step forward button is disabled.
    pub can_step_forward: bool,

    /// Current playback speed (1.0 = normal).
    pub playback_speed: f64,

    /// Whether audio is auto-muted due to high speed (>2x).
    pub speed_auto_muted: bool,

    /// Whether this media has an audio track.
    /// When false, audio controls (mute button, volume slider) are disabled.
    pub has_audio: bool,
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
            overflow_menu_open: false,
            can_step_backward: false,
            can_step_forward: false,
            playback_speed: 1.0,
            speed_auto_muted: false,
            has_audio: true,
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
        icons::sized(action_icons::video::toolbar::pause(), icon_size)
    } else {
        icons::sized(action_icons::video::toolbar::play(), icon_size)
    };

    let play_pause_tooltip = if state.is_playing {
        ctx.i18n.tr("video-pause-tooltip")
    } else {
        ctx.i18n.tr("video-play-tooltip")
    };

    let play_pause_button = tip(
        button(play_pause_svg)
            .on_press(Message::TogglePlayback)
            .padding(spacing::XS)
            .width(Length::Shrink)
            .height(Length::Fixed(button_height)),
        play_pause_tooltip,
    );

    // Timeline position in seconds
    // Use preview position during drag, otherwise use actual playback position
    let timeline_position = state.seek_preview_position.unwrap_or(state.position_secs);

    // Use on_change for visual preview, on_release for actual seek
    let timeline = slider(
        0.0..=state.duration_secs,
        timeline_position,
        Message::SeekPreview,
    )
    .on_release(Message::SeekCommit)
    .width(Length::FillPortion(1))
    .step(SLIDER_STEP_SECS);

    // Format time display - use monospace-like sizing
    let time_display = text(format!(
        "{} / {}",
        format_time(state.position_secs),
        format_time(state.duration_secs)
    ))
    .size(sizing::ICON_SM);

    // Volume button with tooltip - shows mute icon when muted, disabled when no audio
    let volume_icon = if state.muted || state.volume == 0.0 {
        icons::sized(action_icons::video::toolbar::volume_muted(), icon_size)
    } else {
        icons::sized(action_icons::video::toolbar::volume(), icon_size)
    };

    let volume_button_content: Element<'_, Message> = if state.has_audio {
        // Has audio: normal volume button with mute/unmute functionality
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
        let styled_button: Element<'_, Message> = if state.muted {
            volume_button.style(styles::button::selected).into()
        } else {
            volume_button.into()
        };
        tip(styled_button, volume_tooltip).into()
    } else {
        // No audio: disabled button with explanatory tooltip
        let volume_button = button(volume_icon)
            .padding(spacing::XS)
            .width(Length::Shrink)
            .height(Length::Fixed(button_height))
            .style(styles::button::disabled());
        tip(volume_button, ctx.i18n.tr("video-no-audio-tooltip")).into()
    };

    // Volume slider with percentage display
    // Disabled when no audio track
    let current_volume = state.volume; // Copy for closure capture
    let volume_slider: Element<'_, Message> = if state.has_audio {
        slider(0.0..=config::MAX_VOLUME, current_volume, |v| {
            Message::SetVolume(Volume::new(v))
        })
        .width(Length::Fixed(80.0))
        .step(0.01)
        .into()
    } else {
        slider(0.0..=config::MAX_VOLUME, current_volume, move |_v| {
            // No-op: slider is disabled, but we need a message for type inference
            Message::SetVolume(Volume::new(current_volume))
        })
        .width(Length::Fixed(80.0))
        .step(0.01)
        .style(styles::slider::disabled())
        .into()
    };

    // Volume percentage text - grayed when no audio
    let volume_percent: Element<'_, Message> = if state.has_audio {
        text(format_volume_percent(state.volume))
            .size(sizing::ICON_SM)
            .width(Length::Fixed(40.0))
            .into()
    } else {
        text(format_volume_percent(state.volume))
            .size(sizing::ICON_SM)
            .width(Length::Fixed(40.0))
            .style(styles::slider::disabled_text_style)
            .into()
    };

    // More button (overflow menu toggle)
    let more_button_base = button(icons::sized(
        action_icons::video::toolbar::more_options(),
        icon_size,
    ))
    .on_press(Message::ToggleOverflowMenu)
    .padding(spacing::XS)
    .width(Length::Shrink)
    .height(Length::Fixed(button_height));

    let more_button_content: Element<'_, Message> = if state.overflow_menu_open {
        more_button_base.style(styles::button::selected).into()
    } else {
        more_button_base.into()
    };

    let more_button = tip(more_button_content, ctx.i18n.tr("video-more-tooltip"));

    // Loop button with tooltip
    let loop_button_base = button(icons::sized(
        action_icons::video::toolbar::toggle_loop(),
        icon_size,
    ))
    .on_press(Message::ToggleLoop)
    .padding(spacing::XS)
    .width(Length::Shrink)
    .height(Length::Fixed(button_height));

    // Apply active style when loop is enabled (highlighted like mute button)
    let loop_button_content: Element<'_, Message> = if state.loop_enabled {
        loop_button_base.style(styles::button::selected).into()
    } else {
        loop_button_base.into()
    };

    let loop_button = tip(loop_button_content, ctx.i18n.tr("video-loop-tooltip"));

    // Main controls row (simplified - advanced controls in overflow menu)
    let controls: Row<'a, Message> = row![
        play_pause_button,
        timeline,
        time_display,
        volume_button_content,
        volume_slider,
        volume_percent,
        loop_button,
        more_button,
    ]
    .spacing(spacing::XS)
    .padding(spacing::XS)
    .align_y(iced::Alignment::Center);

    // Build overflow menu content if open
    if state.overflow_menu_open {
        let overflow_content = build_overflow_menu(ctx, state, icon_size, button_height);

        // Stack: overflow menu above main controls
        let stacked: Column<'a, Message> = column![overflow_content, controls]
            .spacing(spacing::XXS)
            .width(Length::Fill);

        container(stacked)
            .width(Length::Fill)
            .padding(spacing::XXS)
            .into()
    } else {
        container(controls)
            .width(Length::Fill)
            .padding(spacing::XXS)
            .into()
    }
}

/// Builds the overflow menu with advanced controls.
fn build_overflow_menu<'a>(
    ctx: ViewContext<'a>,
    state: &PlaybackState,
    icon_size: f32,
    button_height: f32,
) -> Element<'a, Message> {
    // Speed down button (disabled at minimum speed)
    let at_min_speed = state.playback_speed <= config::MIN_PLAYBACK_SPEED;
    let speed_down_content: Element<'_, Message> = if at_min_speed {
        button(icons::sized(
            action_icons::video::toolbar::speed_down(),
            icon_size,
        ))
        .padding(spacing::XS)
        .width(Length::Shrink)
        .height(Length::Fixed(button_height))
        .style(styles::button::disabled())
        .into()
    } else {
        button(icons::sized(
            action_icons::video::toolbar::speed_down(),
            icon_size,
        ))
        .on_press(Message::DecreasePlaybackSpeed)
        .padding(spacing::XS)
        .width(Length::Shrink)
        .height(Length::Fixed(button_height))
        .into()
    };
    let speed_down_button = tip(speed_down_content, ctx.i18n.tr("video-speed-down-tooltip"));

    // Speed label (text showing current speed)
    let speed_label = text(format_playback_speed(state.playback_speed))
        .size(sizing::ICON_SM)
        .width(Length::Shrink);

    // Speed up button (disabled at maximum speed)
    let at_max_speed = state.playback_speed >= config::MAX_PLAYBACK_SPEED;
    let speed_up_content: Element<'_, Message> = if at_max_speed {
        button(icons::sized(
            action_icons::video::toolbar::speed_up(),
            icon_size,
        ))
        .padding(spacing::XS)
        .width(Length::Shrink)
        .height(Length::Fixed(button_height))
        .style(styles::button::disabled())
        .into()
    } else {
        button(icons::sized(
            action_icons::video::toolbar::speed_up(),
            icon_size,
        ))
        .on_press(Message::IncreasePlaybackSpeed)
        .padding(spacing::XS)
        .width(Length::Shrink)
        .height(Length::Fixed(button_height))
        .into()
    };
    let speed_up_button = tip(speed_up_content, ctx.i18n.tr("video-speed-up-tooltip"));

    // Step backward button (only enabled when paused AND in stepping mode)
    let step_back_content: Element<'_, Message> = if !state.is_playing && state.can_step_backward {
        button(icons::sized(
            action_icons::video::toolbar::step_backward(),
            icon_size,
        ))
        .on_press(Message::StepBackward)
        .padding(spacing::XS)
        .width(Length::Shrink)
        .height(Length::Fixed(button_height))
        .into()
    } else {
        button(icons::sized(
            action_icons::video::toolbar::step_backward(),
            icon_size,
        ))
        .padding(spacing::XS)
        .width(Length::Shrink)
        .height(Length::Fixed(button_height))
        .style(styles::button::disabled())
        .into()
    };
    let step_back_button = tip(
        step_back_content,
        ctx.i18n.tr("video-step-backward-tooltip"),
    );

    // Step forward button (only enabled when paused AND not at end of video)
    let step_forward_content: Element<'_, Message> = if state.can_step_forward {
        button(icons::sized(
            action_icons::video::toolbar::step_forward(),
            icon_size,
        ))
        .on_press(Message::StepForward)
        .padding(spacing::XS)
        .width(Length::Shrink)
        .height(Length::Fixed(button_height))
        .into()
    } else {
        button(icons::sized(
            action_icons::video::toolbar::step_forward(),
            icon_size,
        ))
        .padding(spacing::XS)
        .width(Length::Shrink)
        .height(Length::Fixed(button_height))
        .style(styles::button::disabled())
        .into()
    };
    let step_forward_button = tip(
        step_forward_content,
        ctx.i18n.tr("video-step-forward-tooltip"),
    );

    // Capture frame button
    let capture_content: Element<'_, Message> = button(icons::sized(
        action_icons::video::toolbar::capture_frame(),
        icon_size,
    ))
    .on_press(Message::CaptureFrame)
    .padding(spacing::XS)
    .width(Length::Shrink)
    .height(Length::Fixed(button_height))
    .into();
    let capture_button = tip(capture_content, ctx.i18n.tr("video-capture-tooltip"));

    // Overflow menu container - align to the right
    // Layout: [Speed Down] [1x] [Speed Up] | [Step Back] [Step Fwd] [Capture]
    let menu_content: Row<'a, Message> = row![
        Space::new().width(Length::Fill),
        speed_down_button,
        speed_label,
        speed_up_button,
        step_back_button,
        step_forward_button,
        capture_button,
    ]
    .spacing(spacing::XS)
    .padding(spacing::XS)
    .align_y(iced::Alignment::Center);

    container(menu_content).width(Length::Fill).into()
}

/// Formats duration in MM:SS or HH:MM:SS format.
fn format_time(seconds: f64) -> String {
    let total_secs = seconds.max(0.0) as u64;
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let secs = total_secs % 60;

    if hours > 0 {
        format!("{hours:02}:{minutes:02}:{secs:02}")
    } else {
        format!("{minutes:02}:{secs:02}")
    }
}

/// Formats playback speed for display.
/// Always shows 2 decimal places for consistent UI width.
fn format_playback_speed(speed: f64) -> String {
    format!("{speed:.2}x")
}

/// Formats volume as percentage for display.
/// Rounds to integer for cleaner UI (e.g., "75%" not "75.00%").
fn format_volume_percent(volume: f32) -> String {
    format!("{}%", (volume * 100.0).round() as u32)
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
        let msg = Message::SeekPreview(30.5);
        let debug_str = format!("{msg:?}");
        assert!(debug_str.contains("SeekPreview"));
        assert!(debug_str.contains("30.5"));
    }

    #[test]
    fn view_renders() {
        let i18n = I18n::default();
        let ctx = ViewContext { i18n: &i18n };
        let state = PlaybackState::default();
        let _element = view(ctx, &state);
    }

    #[test]
    fn timeline_position_uses_seconds() {
        let state = PlaybackState {
            is_playing: true,
            position_secs: 30.0,
            duration_secs: 120.0,
            volume: 0.8,
            muted: false,
            loop_enabled: false,
            seek_preview_position: None,
            overflow_menu_open: false,
            can_step_backward: false,
            can_step_forward: false,
            playback_speed: 1.0,
            speed_auto_muted: false,
            has_audio: true,
        };

        // Position is in seconds
        let position = state.seek_preview_position.unwrap_or(state.position_secs);

        assert_eq!(position, 30.0);
        assert_eq!(state.duration_secs, 120.0);
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
            overflow_menu_open: false,
            can_step_backward: false,
            can_step_forward: false,
            playback_speed: 1.0,
            speed_auto_muted: false,
            has_audio: true,
        };

        // When duration is zero, position is still valid
        let position = state.seek_preview_position.unwrap_or(state.position_secs);

        // Position is 10 seconds, but slider range is 0..=0
        // so the slider will clamp to 0
        assert_eq!(position, 10.0);
        assert_eq!(state.duration_secs, 0.0);
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
            seek_preview_position: Some(90.0), // Preview at 90 seconds
            overflow_menu_open: false,
            can_step_backward: false,
            can_step_forward: false,
            playback_speed: 1.0,
            speed_auto_muted: false,
            has_audio: true,
        };

        // When seek_preview_position is set, it should be used instead of playback position
        let position = state.seek_preview_position.unwrap_or(state.position_secs);

        // Should use preview position (90s) not playback position (30s)
        assert_eq!(position, 90.0);
    }

    #[test]
    fn format_playback_speed_always_two_decimals() {
        // Integer values show .00
        assert_eq!(format_playback_speed(1.0), "1.00x");
        assert_eq!(format_playback_speed(2.0), "2.00x");
        assert_eq!(format_playback_speed(4.0), "4.00x");
        assert_eq!(format_playback_speed(8.0), "8.00x");

        // One decimal values show trailing 0
        assert_eq!(format_playback_speed(0.1), "0.10x");
        assert_eq!(format_playback_speed(0.5), "0.50x");
        assert_eq!(format_playback_speed(1.5), "1.50x");

        // Two decimal values shown as-is
        assert_eq!(format_playback_speed(0.15), "0.15x");
        assert_eq!(format_playback_speed(0.25), "0.25x");
        assert_eq!(format_playback_speed(0.33), "0.33x");
        assert_eq!(format_playback_speed(1.25), "1.25x");
    }

    #[test]
    fn playback_state_default_speed() {
        let state = PlaybackState::default();
        assert_eq!(state.playback_speed, 1.0);
        assert!(!state.speed_auto_muted);
    }

    #[test]
    fn format_volume_percent_rounds_to_integer() {
        // Standard values
        assert_eq!(format_volume_percent(0.0), "0%");
        assert_eq!(format_volume_percent(0.5), "50%");
        assert_eq!(format_volume_percent(1.0), "100%");
        assert_eq!(format_volume_percent(1.5), "150%");

        // Fractional values round correctly
        assert_eq!(format_volume_percent(0.754), "75%");
        assert_eq!(format_volume_percent(0.756), "76%");
        assert_eq!(format_volume_percent(1.25), "125%");
    }
}
