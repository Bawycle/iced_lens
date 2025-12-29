// SPDX-License-Identifier: MPL-2.0
//! Empty state view displayed when no media is loaded.
//!
//! This component provides a welcoming UI with:
//! - An icon and message explaining the empty state
//! - A button to open a file or folder via system dialog
//! - Visual indication that files can be dropped on the window

use super::component::Message;
use crate::i18n::fluent::I18n;
use crate::ui::design_tokens::{palette, sizing, spacing, typography};
use crate::ui::icons;
use crate::ui::styles;
use iced::widget::{button, Column, Container, Row, Text};
use iced::{alignment, Color, Element, Length};

/// Renders the empty state view.
///
/// This view is displayed when the application starts without a file argument
/// or when no media is currently loaded. It provides a welcoming interface
/// with instructions and a button to open files.
pub fn view(i18n: &I18n) -> Element<'_, Message> {
    // Large icon
    let icon = icons::sized(icons::image(), sizing::ICON_XL * 2.0);

    // Title
    let title = Text::new(i18n.tr("empty-state-title"))
        .size(typography::TITLE_LG)
        .color(palette::GRAY_400);

    // Subtitle with drop hint
    let subtitle = Text::new(i18n.tr("empty-state-subtitle"))
        .size(typography::BODY)
        .color(palette::GRAY_400);

    // Open button
    let button_content = Row::new()
        .spacing(spacing::SM)
        .align_y(alignment::Vertical::Center)
        .push(icons::sized(icons::image(), sizing::ICON_SM))
        .push(Text::new(i18n.tr("empty-state-button")));

    let open_button = button(button_content)
        .padding([spacing::SM, spacing::LG])
        .style(styles::button::primary)
        .on_press(Message::OpenFileRequested);

    // Drop zone hint
    let drop_hint = Text::new(i18n.tr("empty-state-drop-hint"))
        .size(typography::CAPTION)
        .color(Color {
            a: 0.5,
            ..palette::GRAY_400
        });

    // Assemble the content
    let content = Column::new()
        .spacing(spacing::LG)
        .align_x(alignment::Horizontal::Center)
        .push(icon)
        .push(title)
        .push(subtitle)
        .push(open_button)
        .push(drop_hint);

    // Center everything in the container
    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .into()
}
