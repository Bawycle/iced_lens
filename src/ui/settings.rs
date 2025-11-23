// SPDX-License-Identifier: MPL-2.0
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//! This module defines the UI components for the application's settings view.
//! It currently provides a language selection submenu, allowing users to
//! choose their preferred display language.
//!
//! # Examples
//!
//! ```no_run
//! use iced_lens::app::{App, Message};
//! use iced_lens::ui::settings;
//! use iced::{Element, widget::Container};
//!
//! // Assume `app` is your main application state
//! # fn dummy_app() -> App {
//! #     App::default()
//! # }
//! #
//! let app = dummy_app();
//! let settings_element: Element<'_, Message> = settings::view_settings(&app);
//!
//! let content = Container::new(settings_element);
//! // ... add to your application's view
//! ```

use crate::app::{App, Message};
use crate::config::BackgroundTheme;
use iced::{
    alignment::{Horizontal, Vertical},
    widget::{button, text, text_input, Button, Column, Container, Row, Text},
    Background, Color, Element, Length, Theme,
};

pub fn view_settings(app: &App) -> Element<'_, Message> {
    let title = Text::new(app.i18n.tr("settings-title")).size(30);

    let mut language_selection_column = Column::new().spacing(10);

    for locale in &app.i18n.available_locales {
        let display_name = locale.to_string(); // Fallback to string representation

        // Check for specific translation for the language name, e.g., "language-name-en-US"
        let translated_name_key = format!("language-name-{}", locale);
        let translated_name = app.i18n.tr(&translated_name_key);
        let button_text = if translated_name.starts_with("MISSING:") {
            display_name.clone() // Use raw locale if translation missing
        } else {
            format!("{} ({})", translated_name, display_name)
        };

        let is_current_locale = app.i18n.current_locale() == locale;
        let mut button =
            Button::new(Text::new(button_text)).on_press(Message::LanguageSelected(locale.clone()));

        if is_current_locale {
            button = button.style(button::primary); // Highlight current language
        } else {
            button = button.style(button::secondary);
        }

        language_selection_column = language_selection_column.push(button);
    }

    let zoom_step_label = Text::new(app.i18n.tr("settings-zoom-step-label"));
    let zoom_step_input = text_input(
        &app.i18n.tr("settings-zoom-step-placeholder"),
        app.zoom_step_input_value(),
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
    let mut helper_text: Element<'_, Message> = Text::new(app.i18n.tr("settings-zoom-step-hint"))
        .size(14)
        .into();

    if let Some(error_key) = app.zoom_step_error_key() {
        let error_color = Color::from_rgb8(229, 57, 53);
        helper_text = Text::new(app.i18n.tr(error_key))
            .size(14)
            .style(move |_theme: &iced::Theme| text::Style {
                color: Some(error_color),
                ..Default::default()
            })
            .into();
    }

    let zoom_content = Column::new()
        .spacing(8)
        .push(zoom_step_label)
        .push(zoom_input_element)
        .push(helper_text);

    let background_label = Text::new(app.i18n.tr("settings-background-label"));
    let mut background_row = Row::new().spacing(8);
    for (theme, key) in [
        (BackgroundTheme::Light, "settings-background-light"),
        (BackgroundTheme::Dark, "settings-background-dark"),
        (
            BackgroundTheme::Checkerboard,
            "settings-background-checkerboard",
        ),
    ] {
        let mut button = Button::new(Text::new(app.i18n.tr(key)))
            .on_press(Message::BackgroundThemeSelected(theme));
        if app.background_theme() == theme {
            button = button.style(button::primary);
        } else {
            button = button.style(button::secondary);
        }
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
            .push(Text::new(app.i18n.tr("select-language-label")).size(18))
            .push(language_selection_column),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_settings_returns_element() {
        let app = App::default();
        let _element = view_settings(&app);
        // Smoke test to ensure the view renders without panicking.
    }
}
