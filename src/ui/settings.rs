// SPDX-License-Identifier: MPL-2.0
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//! Settings UI module following a "state down, messages up" pattern.
//! The [`State`] struct owns the local UI state, while [`Event`] values
//! bubble up for the parent application to handle side effects.

use crate::config::{
    BackgroundTheme, SortOrder, DEFAULT_FRAME_CACHE_MB, DEFAULT_FRAME_HISTORY_MB,
    DEFAULT_OVERLAY_TIMEOUT_SECS, DEFAULT_ZOOM_STEP_PERCENT, MAX_FRAME_CACHE_MB,
    MAX_FRAME_HISTORY_MB, MAX_OVERLAY_TIMEOUT_SECS, MIN_FRAME_CACHE_MB, MIN_FRAME_HISTORY_MB,
    MIN_OVERLAY_TIMEOUT_SECS,
};
use crate::i18n::fluent::I18n;
use crate::ui::design_tokens::{radius, sizing, spacing};
use crate::ui::icons;
use crate::ui::state::zoom::{
    format_number, MAX_ZOOM_STEP_PERCENT, MIN_ZOOM_STEP_PERCENT, ZOOM_STEP_INVALID_KEY,
    ZOOM_STEP_RANGE_KEY,
};
use crate::ui::styles;
use crate::ui::theme;
use crate::ui::theming::ThemeMode;
use iced::{
    alignment::{Horizontal, Vertical},
    widget::{
        button, container, horizontal_rule, scrollable, svg::Svg, text, text_input, Button, Column,
        Container, Row, Slider, Text,
    },
    Border, Element, Length, Theme,
};
use unic_langid::LanguageIdentifier;

/// Contextual data needed to render the settings view.
pub struct ViewContext<'a> {
    pub i18n: &'a I18n,
}

/// Configuration parameters for initializing settings state.
///
/// This struct groups all configuration options to avoid functions with too many arguments.
/// Use `StateConfig::default()` for sensible defaults, then customize as needed.
#[derive(Debug, Clone)]
pub struct StateConfig {
    pub zoom_step_percent: f32,
    pub background_theme: BackgroundTheme,
    pub sort_order: SortOrder,
    pub overlay_timeout_secs: u32,
    pub theme_mode: ThemeMode,
    pub video_autoplay: bool,
    pub audio_normalization: bool,
    pub frame_cache_mb: u32,
    pub frame_history_mb: u32,
}

impl Default for StateConfig {
    fn default() -> Self {
        Self {
            zoom_step_percent: DEFAULT_ZOOM_STEP_PERCENT,
            background_theme: BackgroundTheme::default(),
            sort_order: SortOrder::default(),
            overlay_timeout_secs: DEFAULT_OVERLAY_TIMEOUT_SECS,
            theme_mode: ThemeMode::System,
            video_autoplay: false,
            audio_normalization: true,
            frame_cache_mb: DEFAULT_FRAME_CACHE_MB,
            frame_history_mb: DEFAULT_FRAME_HISTORY_MB,
        }
    }
}

/// Local UI state for the settings screen.
#[derive(Debug, Clone)]
pub struct State {
    background_theme: BackgroundTheme,
    sort_order: SortOrder,
    theme_mode: ThemeMode,
    zoom_step_percent: f32,
    zoom_step_input: String,
    zoom_step_input_dirty: bool,
    zoom_step_error_key: Option<&'static str>,
    overlay_timeout_secs: u32,
    video_autoplay: bool,
    audio_normalization: bool,
    frame_cache_mb: u32,
    frame_history_mb: u32,
}

/// Messages emitted directly by the settings widgets.
#[derive(Debug, Clone)]
pub enum Message {
    BackToViewer,
    LanguageSelected(LanguageIdentifier),
    ZoomStepInputChanged(String),
    ZoomStepSubmitted,
    BackgroundThemeSelected(BackgroundTheme),
    ThemeModeSelected(ThemeMode),
    SortOrderSelected(SortOrder),
    OverlayTimeoutChanged(u32),
    VideoAutoplayChanged(bool),
    AudioNormalizationChanged(bool),
    FrameCacheMbChanged(u32),
    FrameHistoryMbChanged(u32),
}

/// Events propagated to the parent application for side effects.
#[derive(Debug, Clone)]
pub enum Event {
    None,
    BackToViewer,
    BackToViewerWithZoomChange(f32),
    LanguageSelected(LanguageIdentifier),
    ZoomStepChanged(f32),
    BackgroundThemeSelected(BackgroundTheme),
    ThemeModeSelected(ThemeMode),
    SortOrderSelected(SortOrder),
    OverlayTimeoutChanged(u32),
    VideoAutoplayChanged(bool),
    AudioNormalizationChanged(bool),
    FrameCacheMbChanged(u32),
    FrameHistoryMbChanged(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ZoomStepError {
    InvalidInput,
    OutOfRange,
}

/// Helper to update a field and emit an event only if the value changed.
///
/// This reduces boilerplate in settings update handlers where we need to:
/// 1. Check if the new value differs from the current one
/// 2. Update the field if changed
/// 3. Return the appropriate event
fn update_if_changed<T: PartialEq + Clone>(
    current: &mut T,
    new_value: T,
    make_event: impl FnOnce(T) -> Event,
) -> Event {
    if *current == new_value {
        Event::None
    } else {
        *current = new_value.clone();
        make_event(new_value)
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new(StateConfig::default())
    }
}

impl State {
    /// Creates a new settings state from the given configuration.
    pub fn new(config: StateConfig) -> Self {
        let clamped = config
            .zoom_step_percent
            .clamp(MIN_ZOOM_STEP_PERCENT, MAX_ZOOM_STEP_PERCENT);
        let clamped_timeout = config
            .overlay_timeout_secs
            .clamp(MIN_OVERLAY_TIMEOUT_SECS, MAX_OVERLAY_TIMEOUT_SECS);
        let clamped_cache = config
            .frame_cache_mb
            .clamp(MIN_FRAME_CACHE_MB, MAX_FRAME_CACHE_MB);
        let clamped_history = config
            .frame_history_mb
            .clamp(MIN_FRAME_HISTORY_MB, MAX_FRAME_HISTORY_MB);
        Self {
            background_theme: config.background_theme,
            sort_order: config.sort_order,
            theme_mode: config.theme_mode,
            zoom_step_percent: clamped,
            zoom_step_input: format_number(clamped),
            zoom_step_input_dirty: false,
            zoom_step_error_key: None,
            overlay_timeout_secs: clamped_timeout,
            video_autoplay: config.video_autoplay,
            audio_normalization: config.audio_normalization,
            frame_cache_mb: clamped_cache,
            frame_history_mb: clamped_history,
        }
    }

    pub fn background_theme(&self) -> BackgroundTheme {
        self.background_theme
    }

    pub fn sort_order(&self) -> SortOrder {
        self.sort_order
    }

    pub fn theme_mode(&self) -> ThemeMode {
        self.theme_mode
    }

    pub fn zoom_step_percent(&self) -> f32 {
        self.zoom_step_percent
    }

    pub fn overlay_timeout_secs(&self) -> u32 {
        self.overlay_timeout_secs
    }

    pub fn video_autoplay(&self) -> bool {
        self.video_autoplay
    }

    pub fn audio_normalization(&self) -> bool {
        self.audio_normalization
    }

    pub fn frame_cache_mb(&self) -> u32 {
        self.frame_cache_mb
    }

    pub fn frame_history_mb(&self) -> u32 {
        self.frame_history_mb
    }

    pub(crate) fn zoom_step_input_value(&self) -> &str {
        &self.zoom_step_input
    }

    pub(crate) fn zoom_step_error_key(&self) -> Option<&'static str> {
        self.zoom_step_error_key
    }

    #[cfg(test)]
    pub(crate) fn zoom_step_input_dirty(&self) -> bool {
        self.zoom_step_input_dirty
    }

    /// Render the settings view.
    pub fn view<'a>(&'a self, ctx: ViewContext<'a>) -> Element<'a, Message> {
        let back_button = button(
            text(format!(
                "← {}",
                ctx.i18n.tr("settings-back-to-viewer-button")
            ))
            .size(14),
        )
        .on_press(Message::BackToViewer);

        let title = Text::new(ctx.i18n.tr("settings-title")).size(30);

        // =========================================================================
        // SECTION: General (Language, Theme)
        // =========================================================================
        let general_section = self.build_general_section(&ctx);

        // =========================================================================
        // SECTION: Display (Background, Zoom step, Sort order)
        // =========================================================================
        let display_section = self.build_display_section(&ctx);

        // =========================================================================
        // SECTION: Video (Autoplay, Audio normalization, Frame cache)
        // =========================================================================
        let video_section = self.build_video_section(&ctx);

        // =========================================================================
        // SECTION: Fullscreen (Overlay timeout)
        // =========================================================================
        let fullscreen_section = self.build_fullscreen_section(&ctx);

        let content = Column::new()
            .width(Length::Fill)
            .spacing(spacing::LG)
            .align_x(Horizontal::Left)
            .padding(spacing::MD)
            .push(back_button)
            .push(title)
            .push(general_section)
            .push(display_section)
            .push(video_section)
            .push(fullscreen_section);

        scrollable(content).into()
    }

    /// Build the General section (Language, Theme mode).
    fn build_general_section<'a>(&'a self, ctx: &ViewContext<'a>) -> Element<'a, Message> {
        // Language selection
        let mut language_row = Row::new().spacing(spacing::XS).align_y(Vertical::Center);
        for locale in &ctx.i18n.available_locales {
            let display_name = locale.to_string();
            let translated_name_key = format!("language-name-{}", locale);
            let translated_name = ctx.i18n.tr(&translated_name_key);
            let button_text = if translated_name.starts_with("MISSING:") {
                display_name.clone()
            } else {
                format!("{} ({})", translated_name, display_name)
            };

            let is_current = ctx.i18n.current_locale() == locale;
            let btn = Button::new(Text::new(button_text))
                .on_press(Message::LanguageSelected(locale.clone()))
                .style(if is_current {
                    button::primary
                } else {
                    button::secondary
                });
            language_row = language_row.push(btn);
        }

        let language_setting = self.build_setting_row(
            ctx.i18n.tr("select-language-label"),
            None,
            language_row.into(),
        );

        // Theme mode selection
        let mut theme_row = Row::new().spacing(spacing::XS);
        for (mode, key) in [
            (ThemeMode::System, "settings-theme-system"),
            (ThemeMode::Light, "settings-theme-light"),
            (ThemeMode::Dark, "settings-theme-dark"),
        ] {
            let btn = Button::new(Text::new(ctx.i18n.tr(key)))
                .on_press(Message::ThemeModeSelected(mode))
                .style(if self.theme_mode == mode {
                    button::primary
                } else {
                    button::secondary
                });
            theme_row = theme_row.push(btn);
        }

        let theme_setting = self.build_setting_row(
            ctx.i18n.tr("settings-theme-mode-label"),
            None,
            theme_row.into(),
        );

        let content = Column::new()
            .spacing(spacing::MD)
            .push(language_setting)
            .push(theme_setting);

        build_section(
            icons::globe(),
            ctx.i18n.tr("settings-section-general"),
            content.into(),
        )
    }

    /// Build the Display section (Background, Zoom step, Sort order).
    fn build_display_section<'a>(&'a self, ctx: &ViewContext<'a>) -> Element<'a, Message> {
        // Background selection
        let mut background_row = Row::new().spacing(spacing::XS);
        for (theme, key) in [
            (BackgroundTheme::Light, "settings-background-light"),
            (BackgroundTheme::Dark, "settings-background-dark"),
            (
                BackgroundTheme::Checkerboard,
                "settings-background-checkerboard",
            ),
        ] {
            let btn = Button::new(Text::new(ctx.i18n.tr(key)))
                .on_press(Message::BackgroundThemeSelected(theme))
                .style(if self.background_theme == theme {
                    button::primary
                } else {
                    button::secondary
                });
            background_row = background_row.push(btn);
        }

        let background_setting = self.build_setting_row(
            ctx.i18n.tr("settings-background-label"),
            None,
            background_row.into(),
        );

        // Zoom step input
        let zoom_input = text_input(
            &ctx.i18n.tr("settings-zoom-step-placeholder"),
            self.zoom_step_input_value(),
        )
        .on_input(Message::ZoomStepInputChanged)
        .on_submit(Message::ZoomStepSubmitted)
        .padding(6)
        .width(Length::Fixed(100.0));

        let zoom_input_row = Row::new()
            .spacing(spacing::XS)
            .align_y(Vertical::Center)
            .push(zoom_input)
            .push(Text::new("%"));

        let zoom_hint: Element<'_, Message> = if let Some(error_key) = self.zoom_step_error_key() {
            Text::new(ctx.i18n.tr(error_key))
                .size(13)
                .style(move |_theme: &Theme| text::Style {
                    color: Some(theme::error_text_color()),
                })
                .into()
        } else {
            Text::new(ctx.i18n.tr("settings-zoom-step-hint"))
                .size(13)
                .into()
        };

        let zoom_setting = self.build_setting_row(
            ctx.i18n.tr("settings-zoom-step-label"),
            Some(zoom_hint),
            zoom_input_row.into(),
        );

        // Sort order selection
        let mut sort_row = Row::new().spacing(spacing::XS);
        for (order, key) in [
            (SortOrder::Alphabetical, "settings-sort-alphabetical"),
            (SortOrder::ModifiedDate, "settings-sort-modified"),
            (SortOrder::CreatedDate, "settings-sort-created"),
        ] {
            let btn = Button::new(Text::new(ctx.i18n.tr(key)))
                .on_press(Message::SortOrderSelected(order))
                .style(if self.sort_order == order {
                    button::primary
                } else {
                    button::secondary
                });
            sort_row = sort_row.push(btn);
        }

        let sort_setting = self.build_setting_row(
            ctx.i18n.tr("settings-sort-order-label"),
            None,
            sort_row.into(),
        );

        let content = Column::new()
            .spacing(spacing::MD)
            .push(background_setting)
            .push(zoom_setting)
            .push(sort_setting);

        build_section(
            icons::image(),
            ctx.i18n.tr("settings-section-display"),
            content.into(),
        )
    }

    /// Build the Video section (Autoplay, Audio normalization, Frame cache).
    fn build_video_section<'a>(&'a self, ctx: &ViewContext<'a>) -> Element<'a, Message> {
        // Video autoplay toggle
        let mut autoplay_row = Row::new().spacing(spacing::XS);
        for (enabled, key) in [
            (false, "settings-video-autoplay-disabled"),
            (true, "settings-video-autoplay-enabled"),
        ] {
            let btn = Button::new(Text::new(ctx.i18n.tr(key)))
                .on_press(Message::VideoAutoplayChanged(enabled))
                .style(if self.video_autoplay == enabled {
                    button::primary
                } else {
                    button::secondary
                });
            autoplay_row = autoplay_row.push(btn);
        }

        let autoplay_setting = self.build_setting_row(
            ctx.i18n.tr("settings-video-autoplay-label"),
            Some(
                Text::new(ctx.i18n.tr("settings-video-autoplay-hint"))
                    .size(13)
                    .into(),
            ),
            autoplay_row.into(),
        );

        // Audio normalization toggle
        let mut normalization_row = Row::new().spacing(spacing::XS);
        for (enabled, key) in [
            (false, "settings-audio-normalization-disabled"),
            (true, "settings-audio-normalization-enabled"),
        ] {
            let btn = Button::new(Text::new(ctx.i18n.tr(key)))
                .on_press(Message::AudioNormalizationChanged(enabled))
                .style(if self.audio_normalization == enabled {
                    button::primary
                } else {
                    button::secondary
                });
            normalization_row = normalization_row.push(btn);
        }

        let normalization_setting = self.build_setting_row(
            ctx.i18n.tr("settings-audio-normalization-label"),
            Some(
                Text::new(ctx.i18n.tr("settings-audio-normalization-hint"))
                    .size(13)
                    .into(),
            ),
            normalization_row.into(),
        );

        // Frame cache slider
        let cache_slider = Slider::new(
            MIN_FRAME_CACHE_MB..=MAX_FRAME_CACHE_MB,
            self.frame_cache_mb,
            Message::FrameCacheMbChanged,
        )
        .step(16u32)
        .width(Length::Fixed(200.0));

        let cache_value = Text::new(format!(
            "{} {}",
            self.frame_cache_mb,
            ctx.i18n.tr("megabytes")
        ));

        let cache_control = Row::new()
            .spacing(spacing::SM)
            .align_y(Vertical::Center)
            .push(cache_slider)
            .push(cache_value);

        let cache_setting = self.build_setting_row(
            ctx.i18n.tr("settings-frame-cache-label"),
            Some(
                Text::new(ctx.i18n.tr("settings-frame-cache-hint"))
                    .size(13)
                    .into(),
            ),
            cache_control.into(),
        );

        // Frame history slider (for frame-by-frame backward stepping)
        let history_slider = Slider::new(
            MIN_FRAME_HISTORY_MB..=MAX_FRAME_HISTORY_MB,
            self.frame_history_mb,
            Message::FrameHistoryMbChanged,
        )
        .step(16u32)
        .width(Length::Fixed(200.0));

        let history_value = Text::new(format!(
            "{} {}",
            self.frame_history_mb,
            ctx.i18n.tr("megabytes")
        ));

        let history_control = Row::new()
            .spacing(spacing::SM)
            .align_y(Vertical::Center)
            .push(history_slider)
            .push(history_value);

        let history_setting = self.build_setting_row(
            ctx.i18n.tr("settings-frame-history-label"),
            Some(
                Text::new(ctx.i18n.tr("settings-frame-history-hint"))
                    .size(13)
                    .into(),
            ),
            history_control.into(),
        );

        let content = Column::new()
            .spacing(spacing::MD)
            .push(autoplay_setting)
            .push(normalization_setting)
            .push(cache_setting)
            .push(history_setting);

        build_section(
            icons::video_camera(),
            ctx.i18n.tr("settings-section-video"),
            content.into(),
        )
    }

    /// Build the Fullscreen section (Overlay timeout).
    fn build_fullscreen_section<'a>(&'a self, ctx: &ViewContext<'a>) -> Element<'a, Message> {
        let timeout_slider = Slider::new(
            MIN_OVERLAY_TIMEOUT_SECS..=MAX_OVERLAY_TIMEOUT_SECS,
            self.overlay_timeout_secs,
            Message::OverlayTimeoutChanged,
        )
        .step(1u32)
        .width(Length::Fixed(200.0));

        let timeout_value = Text::new(format!(
            "{} {}",
            self.overlay_timeout_secs,
            ctx.i18n.tr("seconds")
        ));

        let timeout_control = Row::new()
            .spacing(spacing::SM)
            .align_y(Vertical::Center)
            .push(timeout_slider)
            .push(timeout_value);

        let timeout_setting = self.build_setting_row(
            ctx.i18n.tr("settings-overlay-timeout-label"),
            Some(
                Text::new(ctx.i18n.tr("settings-overlay-timeout-hint"))
                    .size(13)
                    .into(),
            ),
            timeout_control.into(),
        );

        let content = Column::new().spacing(spacing::MD).push(timeout_setting);

        build_section(
            icons::fullscreen(),
            ctx.i18n.tr("settings-section-fullscreen"),
            content.into(),
        )
    }

    /// Build a single setting row with label, optional hint, and control.
    fn build_setting_row<'a>(
        &self,
        label: String,
        hint: Option<Element<'a, Message>>,
        control: Element<'a, Message>,
    ) -> Element<'a, Message> {
        let mut col = Column::new().spacing(spacing::XS);
        col = col.push(Text::new(label).size(14));
        col = col.push(control);
        if let Some(hint_element) = hint {
            col = col.push(hint_element);
        }
        col.into()
    }

    /// Update the state and emit an [`Event`] for the parent when needed.
    pub fn update(&mut self, message: Message) -> Event {
        match message {
            Message::BackToViewer => {
                // If zoom step input is dirty, validate and commit before leaving
                if self.zoom_step_input_dirty {
                    match self.commit_zoom_step() {
                        Ok(value) => {
                            // Valid zoom step, can proceed to viewer with zoom change
                            Event::BackToViewerWithZoomChange(value)
                        }
                        Err(_) => {
                            // Invalid zoom step, stay in settings
                            Event::None
                        }
                    }
                } else {
                    Event::BackToViewer
                }
            }
            Message::LanguageSelected(locale) => Event::LanguageSelected(locale),
            Message::ZoomStepInputChanged(value) => {
                let sanitized = value.replace('%', "").trim().to_string();
                self.zoom_step_input = sanitized;
                self.zoom_step_input_dirty = true;
                self.zoom_step_error_key = None;
                Event::None
            }
            Message::ZoomStepSubmitted => match self.commit_zoom_step() {
                Ok(value) => Event::ZoomStepChanged(value),
                Err(_) => Event::None,
            },
            Message::BackgroundThemeSelected(theme) => {
                update_if_changed(&mut self.background_theme, theme, Event::BackgroundThemeSelected)
            }
            Message::SortOrderSelected(order) => {
                update_if_changed(&mut self.sort_order, order, Event::SortOrderSelected)
            }
            Message::OverlayTimeoutChanged(timeout) => {
                update_if_changed(&mut self.overlay_timeout_secs, timeout, Event::OverlayTimeoutChanged)
            }
            Message::ThemeModeSelected(mode) => {
                update_if_changed(&mut self.theme_mode, mode, Event::ThemeModeSelected)
            }
            Message::VideoAutoplayChanged(enabled) => {
                update_if_changed(&mut self.video_autoplay, enabled, Event::VideoAutoplayChanged)
            }
            Message::AudioNormalizationChanged(enabled) => {
                update_if_changed(&mut self.audio_normalization, enabled, Event::AudioNormalizationChanged)
            }
            Message::FrameCacheMbChanged(mb) => {
                update_if_changed(&mut self.frame_cache_mb, mb, Event::FrameCacheMbChanged)
            }
            Message::FrameHistoryMbChanged(mb) => {
                update_if_changed(&mut self.frame_history_mb, mb, Event::FrameHistoryMbChanged)
            }
        }
    }

    /// Ensures any pending zoom step edits are validated before leaving the screen.
    pub(crate) fn ensure_zoom_step_committed(&mut self) -> Result<Option<f32>, ZoomStepError> {
        if self.zoom_step_input_dirty {
            self.commit_zoom_step().map(Some)
        } else {
            Ok(None)
        }
    }

    fn commit_zoom_step(&mut self) -> Result<f32, ZoomStepError> {
        if let Some(value) = parse_number(&self.zoom_step_input) {
            if !(MIN_ZOOM_STEP_PERCENT..=MAX_ZOOM_STEP_PERCENT).contains(&value) {
                self.zoom_step_error_key = Some(ZOOM_STEP_RANGE_KEY);
                self.zoom_step_input_dirty = true;
                return Err(ZoomStepError::OutOfRange);
            }

            self.zoom_step_percent = value;
            self.zoom_step_input = format_number(value);
            self.zoom_step_input_dirty = false;
            self.zoom_step_error_key = None;
            Ok(value)
        } else {
            self.zoom_step_error_key = Some(ZOOM_STEP_INVALID_KEY);
            self.zoom_step_input_dirty = true;
            Err(ZoomStepError::InvalidInput)
        }
    }
}

/// Build a settings section with icon, title, and content.
fn build_section<'a>(
    icon: Svg<'a>,
    title: String,
    content: Element<'a, Message>,
) -> Element<'a, Message> {
    let icon_sized = icons::sized(icon, sizing::ICON_MD).style(styles::tinted_svg);

    let header = Row::new()
        .spacing(spacing::SM)
        .align_y(Vertical::Center)
        .push(icon_sized)
        .push(Text::new(title).size(18));

    let inner = Column::new()
        .spacing(spacing::SM)
        .push(header)
        .push(horizontal_rule(1))
        .push(content);

    Container::new(inner)
        .padding(spacing::MD)
        .width(Length::Fill)
        .style(|theme: &Theme| container::Style {
            background: Some(theme.extended_palette().background.weak.color.into()),
            border: Border {
                radius: radius::MD.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

fn parse_number(input: &str) -> Option<f32> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    let without_percent = trimmed.trim_end_matches('%').trim();
    if without_percent.is_empty() {
        return None;
    }

    let normalized = without_percent.replace(',', ".");
    let value = normalized.trim().parse::<f32>().ok()?;
    if !value.is_finite() {
        return None;
    }

    Some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_state_clamps_zoom_step() {
        let config = StateConfig {
            zoom_step_percent: 500.0,
            background_theme: BackgroundTheme::Light,
            sort_order: SortOrder::Alphabetical,
            ..StateConfig::default()
        };
        let state = State::new(config);
        assert_eq!(state.zoom_step_percent, MAX_ZOOM_STEP_PERCENT);
        assert_eq!(state.zoom_step_input, format_number(MAX_ZOOM_STEP_PERCENT));
    }

    #[test]
    fn update_zoom_step_changes_dirty_flag() {
        let mut state = State::default();
        assert!(!state.zoom_step_input_dirty);
        state.update(Message::ZoomStepInputChanged("42".into()));
        assert!(state.zoom_step_input_dirty);
    }

    #[test]
    fn commit_zoom_step_rejects_invalid_input() {
        let mut state = State {
            zoom_step_input: "".into(),
            ..State::default()
        };
        assert_eq!(state.commit_zoom_step(), Err(ZoomStepError::InvalidInput));
        assert_eq!(state.zoom_step_error_key, Some(ZOOM_STEP_INVALID_KEY));
    }

    #[test]
    fn ensure_zoom_step_committed_returns_new_value() {
        let mut state = State::default();
        state.update(Message::ZoomStepInputChanged("15".into()));
        let result = state.ensure_zoom_step_committed().unwrap();
        assert_eq!(result, Some(15.0));
        assert_eq!(state.zoom_step_percent, 15.0);
    }
}
