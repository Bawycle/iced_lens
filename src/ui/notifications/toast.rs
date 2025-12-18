// SPDX-License-Identifier: MPL-2.0
//! Toast widget for rendering individual notifications.
//!
//! Toasts are the visual representation of notifications, appearing as
//! small cards with severity-colored accents and optional dismiss buttons.

use super::manager::{Manager, Message};
use super::notification::{Notification, Severity};
use crate::i18n::fluent::I18n;
use crate::ui::design_tokens::{
    border, opacity, palette, radius, shadow, sizing, spacing, typography,
};
use crate::ui::icons;
use iced::widget::image::{Handle, Image};
use iced::widget::{button, container, text, Column, Container, Row, Text};
use iced::{alignment, Color, Element, Length, Theme};

/// Toast widget configuration.
pub struct Toast;

impl Toast {
    /// Renders a single toast notification.
    pub fn view<'a>(notification: &'a Notification, i18n: &'a I18n) -> Element<'a, Message> {
        let severity = notification.severity();
        let accent_color = severity.color();

        // Resolve the message text using i18n with optional arguments
        let message_text = if notification.message_args().is_empty() {
            i18n.tr(notification.message_key())
        } else {
            // Convert (String, String) to (&str, &str) for tr_with_args
            let args: Vec<(&str, &str)> = notification
                .message_args()
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            i18n.tr_with_args(notification.message_key(), &args)
        };

        // Severity icon (PNG icons have fixed colors)
        let icon = Self::severity_icon(severity);
        let icon_widget = icons::sized(icon, sizing::ICON_MD);

        // Message text
        let message_widget =
            Text::new(message_text)
                .size(typography::BODY)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.palette().text),
                });

        // Dismiss button (always visible, uses main text color for good contrast)
        let notification_id = notification.id();
        let dismiss_button = button(icons::sized(icons::cross(), sizing::ICON_SM))
            .on_press(Message::Dismiss(notification_id))
            .padding(spacing::XXS)
            .style(dismiss_button_style);

        // Layout: [icon] [message] [dismiss]
        let content = Row::new()
            .spacing(spacing::SM)
            .align_y(alignment::Vertical::Center)
            .push(Container::new(icon_widget).padding(spacing::XXS))
            .push(
                Container::new(message_widget)
                    .width(Length::Fill)
                    .align_x(alignment::Horizontal::Left),
            )
            .push(dismiss_button);

        // Toast container with accent border
        Container::new(content)
            .width(Length::Fixed(sizing::TOAST_WIDTH))
            .padding(spacing::SM)
            .style(move |theme: &Theme| toast_container_style(theme, accent_color))
            .into()
    }

    /// Renders the toast overlay with all visible notifications.
    ///
    /// Positions toasts in the bottom-right corner, stacked vertically.
    pub fn view_overlay<'a>(manager: &'a Manager, i18n: &'a I18n) -> Element<'a, Message> {
        let toasts: Vec<Element<'a, Message>> = manager
            .visible()
            .map(|notification| Self::view(notification, i18n))
            .collect();

        if toasts.is_empty() {
            // Return an empty container that takes no space
            Container::new(text(""))
                .width(Length::Shrink)
                .height(Length::Shrink)
                .into()
        } else {
            let toast_column = Column::with_children(toasts)
                .spacing(spacing::XS)
                .align_x(alignment::Horizontal::Right);

            // Position in bottom-right with padding
            Container::new(toast_column)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(alignment::Horizontal::Right)
                .align_y(alignment::Vertical::Bottom)
                .padding(spacing::MD)
                .into()
        }
    }

    /// Returns the appropriate icon for the severity level.
    fn severity_icon(severity: Severity) -> Image<Handle> {
        match severity {
            Severity::Success => icons::checkmark(),
            Severity::Info => icons::info(),
            Severity::Warning => icons::warning(),
            Severity::Error => icons::warning(),
        }
    }
}

/// Style function for the toast container.
fn toast_container_style(theme: &Theme, accent_color: Color) -> container::Style {
    let bg_color = theme.extended_palette().background.base.color;

    container::Style {
        background: Some(iced::Background::Color(bg_color)),
        border: iced::Border {
            color: accent_color,
            width: border::WIDTH_MD,
            radius: radius::MD.into(),
        },
        shadow: shadow::MD,
        text_color: Some(theme.palette().text),
        ..Default::default()
    }
}

/// Style function for the dismiss button.
fn dismiss_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let base = theme.extended_palette().background.base;

    match status {
        button::Status::Active => button::Style {
            background: None,
            text_color: base.text,
            border: iced::Border::default(),
            shadow: shadow::NONE,
            snap: true,
        },
        button::Status::Hovered => button::Style {
            background: Some(iced::Background::Color(Color {
                a: opacity::OVERLAY_SUBTLE,
                ..palette::GRAY_400
            })),
            text_color: base.text,
            border: iced::Border {
                radius: radius::SM.into(),
                ..Default::default()
            },
            shadow: shadow::NONE,
            snap: true,
        },
        button::Status::Pressed => button::Style {
            background: Some(iced::Background::Color(Color {
                a: opacity::OVERLAY_MEDIUM,
                ..palette::GRAY_400
            })),
            text_color: base.text,
            border: iced::Border {
                radius: radius::SM.into(),
                ..Default::default()
            },
            shadow: shadow::NONE,
            snap: true,
        },
        button::Status::Disabled => button::Style {
            background: None,
            text_color: Color {
                a: opacity::OVERLAY_MEDIUM,
                ..base.text
            },
            border: iced::Border::default(),
            shadow: shadow::NONE,
            snap: true,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toast_container_style_uses_accent_color() {
        let theme = Theme::Dark;
        let accent = palette::SUCCESS_500;
        let style = toast_container_style(&theme, accent);

        assert_eq!(style.border.color, accent);
        assert!(style.background.is_some());
    }

    #[test]
    fn severity_icons_are_defined() {
        // Just verify icons don't panic when created
        let _ = Toast::severity_icon(Severity::Success);
        let _ = Toast::severity_icon(Severity::Info);
        let _ = Toast::severity_icon(Severity::Warning);
        let _ = Toast::severity_icon(Severity::Error);
    }
}
