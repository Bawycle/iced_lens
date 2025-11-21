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
use iced::{
    theme, // Added
    widget::{Button, Column, Text},
    Element,
    Length,
};
use unic_langid::LanguageIdentifier; // Added

pub fn view_settings(app: &App) -> Element<'_, Message> {
    let title = Text::new(app.i18n.tr("settings-title")).size(30);

    let mut language_selection_column = Column::new()
        .push(Text::new(app.i18n.tr("select-language-label")))
        .spacing(10);

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
            button = button.style(theme::Button::Primary); // Highlight current language
        } else {
            button = button.style(theme::Button::Secondary);
        }

        language_selection_column = language_selection_column.push(button);
    }

    Column::new()
        .push(title)
        .push(language_selection_column)
        .spacing(20)
        .width(Length::Fill)
        .align_items(iced::alignment::Horizontal::Center.into())
        .into()
}
