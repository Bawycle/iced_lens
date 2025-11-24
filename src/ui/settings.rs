// SPDX-License-Identifier: MPL-2.0
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//! Settings UI module following a "state down, messages up" pattern.
//! The [`State`] struct owns the local UI state, while [`Event`] values
//! bubble up for the parent application to handle side effects.

use crate::config::{BackgroundTheme, DEFAULT_ZOOM_STEP_PERCENT};
use crate::i18n::fluent::I18n;
use crate::ui::state::zoom::{
    format_number, MAX_ZOOM_STEP_PERCENT, MIN_ZOOM_STEP_PERCENT, ZOOM_STEP_INVALID_KEY,
    ZOOM_STEP_RANGE_KEY,
};
use iced::{
    alignment::{Horizontal, Vertical},
    widget::{button, text, text_input, Button, Column, Container, Row, Text},
    Background, Color, Element, Length, Theme,
};
use unic_langid::LanguageIdentifier;

/// Contextual data needed to render the settings view.
pub struct ViewContext<'a> {
    pub i18n: &'a I18n,
}

/// Local UI state for the settings screen.
#[derive(Debug, Clone)]
pub struct State {
    background_theme: BackgroundTheme,
    zoom_step_percent: f32,
    zoom_step_input: String,
    zoom_step_input_dirty: bool,
    zoom_step_error_key: Option<&'static str>,
}

/// Messages emitted directly by the settings widgets.
#[derive(Debug, Clone)]
pub enum Message {
    LanguageSelected(LanguageIdentifier),
    ZoomStepInputChanged(String),
    ZoomStepSubmitted,
    BackgroundThemeSelected(BackgroundTheme),
}

/// Events propagated to the parent application for side effects.
#[derive(Debug, Clone)]
pub enum Event {
    None,
    LanguageSelected(LanguageIdentifier),
    ZoomStepChanged(f32),
    BackgroundThemeSelected(BackgroundTheme),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ZoomStepError {
    InvalidInput,
    OutOfRange,
}

impl Default for State {
    fn default() -> Self {
        Self::new(DEFAULT_ZOOM_STEP_PERCENT, BackgroundTheme::default())
    }
}

impl State {
    pub fn new(initial_zoom_step_percent: f32, background_theme: BackgroundTheme) -> Self {
        let clamped = initial_zoom_step_percent.clamp(MIN_ZOOM_STEP_PERCENT, MAX_ZOOM_STEP_PERCENT);
        Self {
            background_theme,
            zoom_step_percent: clamped,
            zoom_step_input: format_number(clamped),
            zoom_step_input_dirty: false,
            zoom_step_error_key: None,
        }
    }

    pub fn background_theme(&self) -> BackgroundTheme {
        self.background_theme
    }

    pub fn zoom_step_percent(&self) -> f32 {
        self.zoom_step_percent
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
            let error_color = Color::from_rgb8(229, 57, 53);
            helper_text = Text::new(ctx.i18n.tr(error_key))
                .size(14)
                .style(move |_theme: &iced::Theme| text::Style {
                    color: Some(error_color),
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

        let section_style = |theme: &Theme| {
            let palette = theme.extended_palette();
            let base = palette.background.base.color;
            let luminance = base.r + base.g + base.b;
            let (r, g, b) = if luminance < 1.5 {
                (
                    (base.r + 0.10).min(1.0),
                    (base.g + 0.10).min(1.0),
                    (base.b + 0.10).min(1.0),
                )
            } else {
                (
                    (base.r - 0.06).max(0.0),
                    (base.g - 0.06).max(0.0),
                    (base.b - 0.06).max(0.0),
                )
            };

            iced::widget::container::Style {
                background: Some(Background::Color(Color::from_rgba(r, g, b, 0.95))),
                border: iced::Border {
                    radius: 12.0.into(),
                    width: 0.0,
                    ..Default::default()
                },
                ..Default::default()
            }
        };

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

        Column::new()
            .width(Length::Fill)
            .spacing(24)
            .align_x(Horizontal::Left)
            .push(title)
            .push(language_section)
            .push(zoom_section)
            .push(background_section)
            .into()
    }

    /// Update the state and emit an [`Event`] for the parent when needed.
    pub fn update(&mut self, message: Message) -> Event {
        match message {
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
        let state = State::new(500.0, BackgroundTheme::Light);
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
        let mut state = State::default();
        state.zoom_step_input = "".into();
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
