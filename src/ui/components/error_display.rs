// SPDX-License-Identifier: MPL-2.0
//! Reusable error display component with consistent styling.
//!
//! This component displays errors, warnings, and info messages with:
//! - An icon appropriate to the severity
//! - A title describing the issue
//! - A detailed message explaining what went wrong
//! - Optional action button (e.g., "Retry", "Choose another file")
//! - Optional collapsible technical details
//!
//! # Usage
//!
//! ```ignore
//! use crate::ui::components::error_display::{ErrorDisplay, ErrorSeverity};
//!
//! ErrorDisplay::new(ErrorSeverity::Error)
//!     .title("Unable to load image")
//!     .message("The file format is not supported.")
//!     .details("Unsupported codec: HEVC")
//!     .action("Try another file", Message::OpenFile)
//!     .view()
//! ```

use crate::ui::design_tokens::{palette, radius, sizing, spacing};
use crate::ui::icons;
use crate::ui::styles::button as button_styles;
use iced::widget::image::{Handle, Image};
use iced::widget::{button, container, rule, text, Column, Container, Row, Text};
use iced::{alignment, Color, Element, Length, Theme};

/// Severity level determines the color scheme and default icon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ErrorSeverity {
    /// Critical error - prevents operation (red)
    #[default]
    Error,
    /// Warning - operation degraded but possible (orange)
    Warning,
    /// Informational - no action required (blue)
    Info,
}

impl ErrorSeverity {
    /// Returns the primary color for this severity level.
    pub fn color(&self) -> Color {
        match self {
            ErrorSeverity::Error => palette::ERROR_500,
            ErrorSeverity::Warning => palette::WARNING_500,
            ErrorSeverity::Info => palette::INFO_500,
        }
    }

    /// Returns the appropriate icon for this severity level.
    pub fn icon(&self) -> Image<Handle> {
        // Warning icon is used for all severity levels as it's the most recognizable
        // The color differentiation handles the severity communication
        icons::warning()
    }
}

/// Configuration for the ErrorDisplay component.
#[derive(Debug, Clone)]
pub struct ErrorDisplay<Message> {
    severity: ErrorSeverity,
    title: Option<String>,
    message: Option<String>,
    details: Option<String>,
    show_details: bool,
    action_label: Option<String>,
    action_message: Option<Message>,
    toggle_details_message: Option<Message>,
    show_details_label: String,
    hide_details_label: String,
    details_heading_label: String,
}

impl<Message> Default for ErrorDisplay<Message> {
    fn default() -> Self {
        Self {
            severity: ErrorSeverity::default(),
            title: None,
            message: None,
            details: None,
            show_details: false,
            action_label: None,
            action_message: None,
            toggle_details_message: None,
            show_details_label: "Show details".to_string(),
            hide_details_label: "Hide details".to_string(),
            details_heading_label: "Technical details".to_string(),
        }
    }
}

impl<Message: Clone + 'static> ErrorDisplay<Message> {
    /// Creates a new error display with the given severity.
    pub fn new(severity: ErrorSeverity) -> Self {
        Self {
            severity,
            ..Self::default()
        }
    }

    /// Sets the title (main heading).
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the message (user-friendly explanation).
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Sets the technical details (collapsible).
    pub fn details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// Sets whether details are currently shown.
    pub fn details_visible(mut self, visible: bool) -> Self {
        self.show_details = visible;
        self
    }

    /// Sets the action button label and message.
    pub fn action(mut self, label: impl Into<String>, message: Message) -> Self {
        self.action_label = Some(label.into());
        self.action_message = Some(message);
        self
    }

    /// Sets the message to emit when toggling details visibility.
    pub fn on_toggle_details(mut self, message: Message) -> Self {
        self.toggle_details_message = Some(message);
        self
    }

    /// Sets the localized labels for the details toggle.
    pub fn details_labels(
        mut self,
        show_label: impl Into<String>,
        hide_label: impl Into<String>,
        heading_label: impl Into<String>,
    ) -> Self {
        self.show_details_label = show_label.into();
        self.hide_details_label = hide_label.into();
        self.details_heading_label = heading_label.into();
        self
    }

    /// Renders the error display component.
    pub fn view(self) -> Element<'static, Message> {
        let accent_color = self.severity.color();

        // Icon with severity color (PNG icons have fixed colors)
        let icon = icons::sized(self.severity.icon(), sizing::ICON_XL);

        let icon_container = Container::new(icon)
            .width(Length::Shrink)
            .align_x(alignment::Horizontal::Center);

        // Build content column
        let mut content = Column::new()
            .spacing(spacing::SM)
            .align_x(alignment::Horizontal::Center)
            .width(Length::Fill);

        // Title
        if let Some(title_text) = self.title {
            let title = Text::new(title_text)
                .size(20)
                .style(move |_theme: &Theme| text::Style {
                    color: Some(accent_color),
                });
            content = content.push(title);
        }

        // Message
        if let Some(message_text) = self.message {
            let message = Text::new(message_text).size(14);
            content = content.push(
                Container::new(message)
                    .width(Length::Fill)
                    .align_x(alignment::Horizontal::Center),
            );
        }

        // Action button
        if let (Some(label), Some(msg)) = (self.action_label, self.action_message) {
            let action_btn = button(Text::new(label))
                .on_press(msg)
                .style(button_styles::selected);
            content = content.push(
                Container::new(action_btn)
                    .padding(spacing::SM)
                    .align_x(alignment::Horizontal::Center),
            );
        }

        // Details toggle and content
        if self.details.is_some() {
            let toggle_label = if self.show_details {
                self.hide_details_label
            } else {
                self.show_details_label
            };

            if let Some(toggle_msg) = self.toggle_details_message {
                let toggle_btn = button(Text::new(toggle_label).size(13)).on_press(toggle_msg);
                content = content.push(
                    Container::new(toggle_btn)
                        .padding(spacing::XS)
                        .align_x(alignment::Horizontal::Center),
                );
            }

            if self.show_details {
                if let Some(details_text) = self.details {
                    // Use theme-aware secondary text color for details
                    let details_heading =
                        Text::new(self.details_heading_label)
                            .size(14)
                            .style(|theme: &Theme| text::Style {
                                color: Some(theme.extended_palette().secondary.base.text),
                            });

                    let details_body =
                        Text::new(details_text)
                            .size(12)
                            .style(|theme: &Theme| text::Style {
                                color: Some(theme.extended_palette().secondary.base.text),
                            });

                    let details_column = Column::new()
                        .spacing(spacing::XS)
                        .width(Length::Fill)
                        .push(rule::horizontal(1))
                        .push(details_heading)
                        .push(details_body);

                    content = content.push(
                        Container::new(details_column)
                            .width(Length::Fill)
                            .padding(spacing::SM),
                    );
                }
            }
        }

        // Combine icon and content
        let main_row = Row::new()
            .spacing(spacing::MD)
            .align_y(alignment::Vertical::Top)
            .push(icon_container)
            .push(content);

        // Outer container with neutral background (Option A: subtle, non-aggressive)
        Container::new(main_row)
            .width(Length::Fill)
            .max_width(500.0)
            .padding(spacing::LG)
            .style(move |theme: &Theme| {
                // Use theme's weak background for a subtle, neutral appearance
                let bg_color = theme.extended_palette().background.weak.color;
                let border_color = theme.extended_palette().background.strong.color;
                container::Style {
                    background: Some(iced::Background::Color(bg_color)),
                    border: iced::Border {
                        color: border_color,
                        width: 1.0,
                        radius: radius::MD.into(),
                    },
                    text_color: Some(theme.palette().text),
                    ..Default::default()
                }
            })
            .into()
    }
}

/// Simplified error view for common use cases.
///
/// Creates a centered error display that fills its container.
pub fn centered_error_view<Message: Clone + 'static>(
    error_display: ErrorDisplay<Message>,
) -> Element<'static, Message> {
    Container::new(error_display.view())
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .padding(spacing::LG)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    enum TestMessage {
        Retry,
        ToggleDetails,
    }

    #[test]
    fn error_severity_colors_are_distinct() {
        let error_color = ErrorSeverity::Error.color();
        let warning_color = ErrorSeverity::Warning.color();
        let info_color = ErrorSeverity::Info.color();

        assert_ne!(error_color.r, warning_color.r);
        assert_ne!(warning_color.r, info_color.r);
        assert_ne!(error_color.r, info_color.r);
    }

    #[test]
    fn error_display_builder_works() {
        let display: ErrorDisplay<TestMessage> = ErrorDisplay::new(ErrorSeverity::Error)
            .title("Test Error")
            .message("Something went wrong")
            .details("Stack trace here")
            .details_visible(false)
            .action("Retry", TestMessage::Retry)
            .on_toggle_details(TestMessage::ToggleDetails);

        assert_eq!(display.severity, ErrorSeverity::Error);
        assert_eq!(display.title, Some("Test Error".to_string()));
        assert_eq!(display.message, Some("Something went wrong".to_string()));
        assert_eq!(display.details, Some("Stack trace here".to_string()));
        assert!(!display.show_details);
    }

    #[test]
    fn default_severity_is_error() {
        let display: ErrorDisplay<TestMessage> = ErrorDisplay::default();
        assert_eq!(display.severity, ErrorSeverity::Error);
    }

    #[test]
    fn details_labels_can_be_customized() {
        let display: ErrorDisplay<TestMessage> = ErrorDisplay::new(ErrorSeverity::Warning)
            .details_labels("Afficher", "Masquer", "Technique");

        assert_eq!(display.show_details_label, "Afficher");
        assert_eq!(display.hide_details_label, "Masquer");
        assert_eq!(display.details_heading_label, "Technique");
    }
}
