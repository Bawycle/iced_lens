// SPDX-License-Identifier: MPL-2.0
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//! Settings UI module following a "state down, messages up" pattern.
//! The [`State`] struct owns the local UI state, while [`Event`] values
//! bubble up for the parent application to handle side effects.

use crate::config::{
    BackgroundTheme, SortOrder, DEFAULT_FRAME_CACHE_MB, DEFAULT_OVERLAY_TIMEOUT_SECS,
    DEFAULT_ZOOM_STEP_PERCENT, MAX_FRAME_CACHE_MB, MAX_OVERLAY_TIMEOUT_SECS, MIN_FRAME_CACHE_MB,
    MIN_OVERLAY_TIMEOUT_SECS,
};
use crate::i18n::fluent::I18n;
use crate::ui::state::zoom::{
    format_number, MAX_ZOOM_STEP_PERCENT, MIN_ZOOM_STEP_PERCENT, ZOOM_STEP_INVALID_KEY,
    ZOOM_STEP_RANGE_KEY,
};
use crate::ui::styles;
use crate::ui::theme;
use crate::ui::theming::ThemeMode;
use iced::{
    alignment::{Horizontal, Vertical},
    widget::{button, scrollable, text, text_input, Button, Column, Container, Row, Slider, Text},
    Element, Length,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ZoomStepError {
    InvalidInput,
    OutOfRange,
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

        let mut language_selection_row = Row::new().spacing(10).align_y(Vertical::Center);
        for locale in &ctx.i18n.available_locales {
            let display_name = locale.to_string();
            let translated_name_key = format!("language-name-{}", locale);
            let translated_name = ctx.i18n.tr(&translated_name_key);
            let button_text = if translated_name.starts_with("MISSING:") {
                display_name.clone()
            } else {
                format!("{} ({})", translated_name, display_name)
            };

            let is_current_locale = ctx.i18n.current_locale() == locale;
            let mut button = Button::new(Text::new(button_text))
                .on_press(Message::LanguageSelected(locale.clone()));
            button = if is_current_locale {
                button.style(button::primary)
            } else {
                button.style(button::secondary)
            };
            language_selection_row = language_selection_row.push(button);
        }

        let zoom_step_label = Text::new(ctx.i18n.tr("settings-zoom-step-label"));
        let zoom_step_input = text_input(
            &ctx.i18n.tr("settings-zoom-step-placeholder"),
            self.zoom_step_input_value(),
        )
        .on_input(Message::ZoomStepInputChanged)
        .on_submit(Message::ZoomStepSubmitted)
        .padding(6)
        .width(Length::Fixed(120.0));

        let zoom_step_input_row = Row::new()
            .spacing(8)
            .align_y(Vertical::Center)
            .push(zoom_step_input)
            .push(Text::new("%"));

        let zoom_input_element: Element<'_, Message> = zoom_step_input_row.into();
        let mut helper_text: Element<'_, Message> =
            Text::new(ctx.i18n.tr("settings-zoom-step-hint"))
                .size(14)
                .into();

        if let Some(error_key) = self.zoom_step_error_key() {
            helper_text = Text::new(ctx.i18n.tr(error_key))
                .size(14)
                .style(move |_theme: &iced::Theme| text::Style {
                    color: Some(theme::error_text_color()),
                })
                .into();
        }

        let zoom_content = Column::new()
            .spacing(8)
            .push(zoom_step_label)
            .push(zoom_input_element)
            .push(helper_text);

        let background_label = Text::new(ctx.i18n.tr("settings-background-label"));
        let mut background_row = Row::new().spacing(8);
        for (theme, key) in [
            (BackgroundTheme::Light, "settings-background-light"),
            (BackgroundTheme::Dark, "settings-background-dark"),
            (
                BackgroundTheme::Checkerboard,
                "settings-background-checkerboard",
            ),
        ] {
            let mut button = Button::new(Text::new(ctx.i18n.tr(key)))
                .on_press(Message::BackgroundThemeSelected(theme));
            button = if self.background_theme == theme {
                button.style(button::primary)
            } else {
                button.style(button::secondary)
            };
            background_row = background_row.push(button);
        }

        let mut theme_mode_row = Row::new().spacing(8);
        for (mode, key) in [
            (ThemeMode::System, "settings-theme-system"),
            (ThemeMode::Light, "settings-theme-light"),
            (ThemeMode::Dark, "settings-theme-dark"),
        ] {
            let mut button =
                Button::new(Text::new(ctx.i18n.tr(key))).on_press(Message::ThemeModeSelected(mode));
            button = if self.theme_mode == mode {
                button.style(button::primary)
            } else {
                button.style(button::secondary)
            };
            theme_mode_row = theme_mode_row.push(button);
        }

        let section_style = styles::container::panel;

        let language_section = Container::new(
            Column::new()
                .spacing(12)
                .push(Text::new(ctx.i18n.tr("select-language-label")).size(18))
                .push(language_selection_row),
        )
        .padding(16)
        .width(Length::Fill)
        .style(section_style);

        let zoom_section = Container::new(zoom_content)
            .padding(16)
            .width(Length::Fill)
            .style(section_style);

        let background_section = Container::new(
            Column::new()
                .spacing(12)
                .push(background_label)
                .push(background_row),
        )
        .padding(16)
        .width(Length::Fill)
        .style(section_style);

        let theme_mode_label = Text::new(ctx.i18n.tr("settings-theme-mode-label"));
        let theme_mode_section = Container::new(
            Column::new()
                .spacing(12)
                .push(theme_mode_label)
                .push(theme_mode_row),
        )
        .padding(16)
        .width(Length::Fill)
        .style(section_style);

        let sort_order_label = Text::new(ctx.i18n.tr("settings-sort-order-label"));
        let mut sort_order_row = Row::new().spacing(8);
        for (order, key) in [
            (SortOrder::Alphabetical, "settings-sort-alphabetical"),
            (SortOrder::ModifiedDate, "settings-sort-modified"),
            (SortOrder::CreatedDate, "settings-sort-created"),
        ] {
            let mut button = Button::new(Text::new(ctx.i18n.tr(key)))
                .on_press(Message::SortOrderSelected(order));
            button = if self.sort_order == order {
                button.style(button::primary)
            } else {
                button.style(button::secondary)
            };
            sort_order_row = sort_order_row.push(button);
        }

        let sort_order_section = Container::new(
            Column::new()
                .spacing(12)
                .push(sort_order_label)
                .push(sort_order_row),
        )
        .padding(16)
        .width(Length::Fill)
        .style(section_style);

        let overlay_timeout_label = Text::new(ctx.i18n.tr("settings-overlay-timeout-label"));
        let overlay_timeout_slider = Slider::new(
            MIN_OVERLAY_TIMEOUT_SECS..=MAX_OVERLAY_TIMEOUT_SECS,
            self.overlay_timeout_secs,
            Message::OverlayTimeoutChanged,
        )
        .step(1u32);

        let overlay_timeout_value_text = Text::new(format!(
            "{} {}",
            self.overlay_timeout_secs,
            ctx.i18n.tr("seconds")
        ));

        let overlay_timeout_section = Container::new(
            Column::new()
                .spacing(12)
                .push(overlay_timeout_label)
                .push(overlay_timeout_slider)
                .push(overlay_timeout_value_text)
                .push(Text::new(ctx.i18n.tr("settings-overlay-timeout-hint")).size(14)),
        )
        .padding(16)
        .width(Length::Fill)
        .style(section_style);

        // Video autoplay toggle
        let video_autoplay_label = Text::new(ctx.i18n.tr("settings-video-autoplay-label"));
        let mut video_autoplay_row = Row::new().spacing(8);
        for (enabled, key) in [
            (false, "settings-video-autoplay-disabled"),
            (true, "settings-video-autoplay-enabled"),
        ] {
            let mut btn = Button::new(Text::new(ctx.i18n.tr(key)))
                .on_press(Message::VideoAutoplayChanged(enabled));
            btn = if self.video_autoplay == enabled {
                btn.style(button::primary)
            } else {
                btn.style(button::secondary)
            };
            video_autoplay_row = video_autoplay_row.push(btn);
        }

        let video_autoplay_section = Container::new(
            Column::new()
                .spacing(12)
                .push(video_autoplay_label)
                .push(video_autoplay_row)
                .push(Text::new(ctx.i18n.tr("settings-video-autoplay-hint")).size(14)),
        )
        .padding(16)
        .width(Length::Fill)
        .style(section_style);

        // Audio normalization toggle
        let audio_normalization_label =
            Text::new(ctx.i18n.tr("settings-audio-normalization-label"));
        let mut audio_normalization_row = Row::new().spacing(8);
        for (enabled, key) in [
            (false, "settings-audio-normalization-disabled"),
            (true, "settings-audio-normalization-enabled"),
        ] {
            let mut btn = Button::new(Text::new(ctx.i18n.tr(key)))
                .on_press(Message::AudioNormalizationChanged(enabled));
            btn = if self.audio_normalization == enabled {
                btn.style(button::primary)
            } else {
                btn.style(button::secondary)
            };
            audio_normalization_row = audio_normalization_row.push(btn);
        }

        let audio_normalization_section = Container::new(
            Column::new()
                .spacing(12)
                .push(audio_normalization_label)
                .push(audio_normalization_row)
                .push(Text::new(ctx.i18n.tr("settings-audio-normalization-hint")).size(14)),
        )
        .padding(16)
        .width(Length::Fill)
        .style(section_style);

        // Frame cache size slider
        let frame_cache_label = Text::new(ctx.i18n.tr("settings-frame-cache-label"));
        let frame_cache_slider = Slider::new(
            MIN_FRAME_CACHE_MB..=MAX_FRAME_CACHE_MB,
            self.frame_cache_mb,
            Message::FrameCacheMbChanged,
        )
        .step(16u32);

        let frame_cache_value_text = Text::new(format!(
            "{} {}",
            self.frame_cache_mb,
            ctx.i18n.tr("megabytes")
        ));

        let frame_cache_section = Container::new(
            Column::new()
                .spacing(12)
                .push(frame_cache_label)
                .push(frame_cache_slider)
                .push(frame_cache_value_text)
                .push(Text::new(ctx.i18n.tr("settings-frame-cache-hint")).size(14)),
        )
        .padding(16)
        .width(Length::Fill)
        .style(section_style);

        let content = Column::new()
            .width(Length::Fill)
            .spacing(24)
            .align_x(Horizontal::Left)
            .padding(10)
            .push(back_button)
            .push(title)
            .push(language_section)
            .push(zoom_section)
            .push(background_section)
            .push(theme_mode_section)
            .push(sort_order_section)
            .push(overlay_timeout_section)
            .push(video_autoplay_section)
            .push(audio_normalization_section)
            .push(frame_cache_section);

        scrollable(content).into()
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
                if self.background_theme == theme {
                    Event::None
                } else {
                    self.background_theme = theme;
                    Event::BackgroundThemeSelected(theme)
                }
            }
            Message::SortOrderSelected(order) => {
                if self.sort_order == order {
                    Event::None
                } else {
                    self.sort_order = order;
                    Event::SortOrderSelected(order)
                }
            }
            Message::OverlayTimeoutChanged(timeout) => {
                if self.overlay_timeout_secs == timeout {
                    Event::None
                } else {
                    self.overlay_timeout_secs = timeout;
                    Event::OverlayTimeoutChanged(timeout)
                }
            }
            Message::ThemeModeSelected(mode) => {
                if self.theme_mode == mode {
                    Event::None
                } else {
                    self.theme_mode = mode;
                    Event::ThemeModeSelected(mode)
                }
            }
            Message::VideoAutoplayChanged(enabled) => {
                if self.video_autoplay == enabled {
                    Event::None
                } else {
                    self.video_autoplay = enabled;
                    Event::VideoAutoplayChanged(enabled)
                }
            }
            Message::AudioNormalizationChanged(enabled) => {
                if self.audio_normalization == enabled {
                    Event::None
                } else {
                    self.audio_normalization = enabled;
                    Event::AudioNormalizationChanged(enabled)
                }
            }
            Message::FrameCacheMbChanged(mb) => {
                if self.frame_cache_mb == mb {
                    Event::None
                } else {
                    self.frame_cache_mb = mb;
                    Event::FrameCacheMbChanged(mb)
                }
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
