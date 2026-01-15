// SPDX-License-Identifier: MPL-2.0
//! Diagnostics screen module for viewing and exporting diagnostic data.
//!
//! This module provides a UI for accessing diagnostic controls, viewing
//! collection status, and exporting diagnostic reports.

use std::time::Duration;

use crate::diagnostics::CollectionStatus;

/// Documentation URL for diagnostics.
const DOCS_URL: &str =
    "https://codeberg.org/Bawycle/iced_lens/src/branch/master/docs/USER_GUIDE.md";
use crate::i18n::fluent::I18n;
use crate::ui::action_icons;
use crate::ui::design_tokens::{palette, radius, sizing, spacing, typography};
use crate::ui::icons;
use iced::{
    alignment::{Horizontal, Vertical},
    widget::{
        button, container, rule, scrollable, text, toggler, Column, Container, Row, Space, Text,
    },
    Border, Color, Element, Length, Theme,
};

/// Contextual data needed to render the diagnostics screen.
pub struct ViewContext<'a> {
    pub i18n: &'a I18n,
    /// Current collection status.
    pub status: CollectionStatus,
    /// Number of events in the buffer.
    pub event_count: usize,
    /// Duration since collection started.
    pub collection_duration: Duration,
    /// Whether the dark theme is active (for icon selection).
    pub is_dark_theme: bool,
}

/// Messages emitted by the diagnostics screen.
#[derive(Debug, Clone)]
pub enum Message {
    BackToViewer,
    /// Refresh status on subscription tick.
    RefreshStatus,
    /// Toggle resource collection on/off.
    ToggleResourceCollection(bool),
    /// Export diagnostic report to file.
    ExportToFile,
    /// Copy diagnostic report to clipboard.
    ExportToClipboard,
}

/// Events propagated to the parent application.
#[derive(Debug, Clone)]
pub enum Event {
    None,
    BackToViewer,
    /// Request to toggle resource collection.
    ToggleResourceCollection(bool),
    /// Request to export diagnostic report to file.
    ExportToFile,
    /// Request to copy diagnostic report to clipboard.
    ExportToClipboard,
}

/// Process a diagnostics screen message and return the corresponding event.
#[must_use]
pub fn update(message: &Message) -> Event {
    match message {
        Message::BackToViewer => Event::BackToViewer,
        Message::RefreshStatus => Event::None,
        Message::ToggleResourceCollection(enabled) => Event::ToggleResourceCollection(*enabled),
        Message::ExportToFile => Event::ExportToFile,
        Message::ExportToClipboard => Event::ExportToClipboard,
    }
}

/// Formats a duration for display.
///
/// - Under 1 hour: "Xm Ys"
/// - Over 1 hour: "Xh Ym Zs"
#[must_use]
pub fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    if hours > 0 {
        format!("{hours}h {minutes}m {seconds}s")
    } else {
        format!("{minutes}m {seconds}s")
    }
}

/// Gets the color for the status indicator based on collection status.
fn status_color(status: &CollectionStatus) -> Color {
    match status {
        CollectionStatus::Enabled { .. } => palette::SUCCESS_500,
        CollectionStatus::Disabled => palette::GRAY_400,
        CollectionStatus::Error { .. } => palette::ERROR_500,
    }
}

/// Builds the status indicator dot.
fn build_status_indicator(color: Color) -> Element<'static, Message> {
    let dot_size = 12.0;
    container(text(""))
        .width(dot_size)
        .height(dot_size)
        .style(move |_theme| container::Style {
            background: Some(color.into()),
            border: Border::default().rounded(radius::FULL),
            ..Default::default()
        })
        .into()
}

/// Builds the info section with description, data collected list, privacy notice, and docs link.
fn build_info_section<'a>(ctx: &ViewContext<'a>) -> Element<'a, Message> {
    // Description (AC 1)
    let description = Text::new(ctx.i18n.tr("diagnostics-info-description")).size(typography::BODY);

    // Data collected header (AC 2)
    let data_header =
        Text::new(ctx.i18n.tr("diagnostics-data-collected-title")).size(typography::BODY_LG);

    // Data collected list (AC 2)
    let data_list = Column::new()
        .spacing(spacing::XS)
        .push(data_header)
        .push(build_data_item(
            &ctx.i18n.tr("diagnostics-data-item-resources"),
        ))
        .push(build_data_item(
            &ctx.i18n.tr("diagnostics-data-item-actions"),
        ))
        .push(build_data_item(
            &ctx.i18n.tr("diagnostics-data-item-states"),
        ))
        .push(build_data_item(
            &ctx.i18n.tr("diagnostics-data-item-errors"),
        ));

    // Privacy notice (AC 3) - subtle gray styling
    let privacy = Text::new(ctx.i18n.tr("diagnostics-privacy-notice"))
        .size(typography::BODY)
        .color(palette::GRAY_400);

    // Documentation link (AC 4)
    let docs_link = build_link_item(&ctx.i18n.tr("diagnostics-docs-link"), DOCS_URL);

    let content = Column::new()
        .spacing(spacing::SM)
        .push(description)
        .push(data_list)
        .push(privacy)
        .push(docs_link);

    build_section(
        icons::info(),
        ctx.i18n.tr("diagnostics-info-title"),
        content.into(),
    )
}

/// Build a bullet point item for the data collected list.
fn build_data_item<'a>(description: &str) -> Element<'a, Message> {
    Text::new(format!("• {description}"))
        .size(typography::BODY)
        .into()
}

/// Build a link item with label and URL.
fn build_link_item<'a>(label: &str, url: &'a str) -> Element<'a, Message> {
    Row::new()
        .spacing(spacing::SM)
        .push(Text::new(format!("{label}:")).size(typography::BODY))
        .push(Text::new(url).size(typography::BODY))
        .into()
}

/// Build a section with icon, title, and content (same pattern as about.rs).
fn build_section(
    icon: iced::widget::Image<iced::widget::image::Handle>,
    title: String,
    content: Element<'_, Message>,
) -> Element<'_, Message> {
    let icon_sized = icons::sized(icon, sizing::ICON_MD);

    let header = Row::new()
        .spacing(spacing::SM)
        .align_y(Vertical::Center)
        .push(icon_sized)
        .push(Text::new(title).size(typography::TITLE_SM));

    let inner = Column::new()
        .spacing(spacing::SM)
        .push(header)
        .push(rule::horizontal(1))
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

/// Builds the status section showing collection state, durations, and buffer count.
fn build_status_section<'a>(ctx: &ViewContext<'a>) -> Element<'a, Message> {
    let color = status_color(&ctx.status);

    // Status text
    let status_text = match &ctx.status {
        CollectionStatus::Enabled { .. } => ctx.i18n.tr("diagnostics-status-enabled"),
        CollectionStatus::Disabled => ctx.i18n.tr("diagnostics-status-disabled"),
        CollectionStatus::Error { message } => {
            format!("{}: {}", ctx.i18n.tr("diagnostics-status-error"), message)
        }
    };

    // Status row with indicator dot and text
    let status_row = Row::new()
        .spacing(spacing::XS)
        .align_y(iced::Alignment::Center)
        .push(build_status_indicator(color))
        .push(Text::new(status_text).size(typography::BODY));

    // Event collection duration (always running since app start)
    let event_duration_str = format_duration(ctx.collection_duration);
    let event_duration_text = ctx.i18n.tr_with_args(
        "diagnostics-events-running-for",
        &[("duration", &event_duration_str)],
    );

    // Buffer count text
    let count_str = ctx.event_count.to_string();
    let buffer_text = ctx
        .i18n
        .tr_with_args("diagnostics-buffer-count", &[("count", &count_str)]);

    let mut column = Column::new()
        .spacing(spacing::SM)
        .push(status_row)
        .push(Text::new(event_duration_text).size(typography::BODY));

    // Resource collection duration (only when enabled)
    if let CollectionStatus::Enabled { started_at } = &ctx.status {
        let resource_duration_str = format_duration(started_at.elapsed());
        let resource_duration_text = ctx.i18n.tr_with_args(
            "diagnostics-resources-running-for",
            &[("duration", &resource_duration_str)],
        );
        column = column.push(Text::new(resource_duration_text).size(typography::BODY));
    }

    column
        .push(Text::new(buffer_text).size(typography::BODY))
        .into()
}

/// Builds the toggle section for enabling/disabling resource collection.
fn build_toggle_section<'a>(ctx: &ViewContext<'a>) -> Element<'a, Message> {
    let is_enabled = matches!(ctx.status, CollectionStatus::Enabled { .. });

    let label = Text::new(ctx.i18n.tr("diagnostics-toggle-label")).size(typography::BODY);

    let toggle = toggler(is_enabled)
        .on_toggle(Message::ToggleResourceCollection)
        .size(20.0); // Match existing IcedLens style

    Row::new()
        .spacing(spacing::SM)
        .align_y(iced::Alignment::Center)
        .push(label)
        .push(Space::new().width(Length::Fill))
        .push(toggle)
        .into()
}

/// Builds the export buttons section.
fn build_export_section<'a>(ctx: &ViewContext<'a>) -> Element<'a, Message> {
    let can_export = ctx.event_count > 0;

    // Export to File button (icon + text)
    let file_icon =
        action_icons::diagnostics::export_file(ctx.is_dark_theme).width(sizing::ICON_SM);
    let file_content = Row::new()
        .spacing(spacing::XS)
        .align_y(Vertical::Center)
        .push(file_icon)
        .push(text(ctx.i18n.tr("diagnostics-export-file")).size(typography::BODY));

    let file_button = button(file_content).padding([spacing::XS, spacing::SM]);
    let file_button = if can_export {
        file_button.on_press(Message::ExportToFile)
    } else {
        file_button // Disabled - no on_press
    };

    // Copy to Clipboard button (icon + text)
    let clipboard_icon =
        action_icons::diagnostics::export_clipboard(ctx.is_dark_theme).width(sizing::ICON_SM);
    let clipboard_content = Row::new()
        .spacing(spacing::XS)
        .align_y(Vertical::Center)
        .push(clipboard_icon)
        .push(text(ctx.i18n.tr("diagnostics-export-clipboard")).size(typography::BODY));

    let clipboard_button = button(clipboard_content).padding([spacing::XS, spacing::SM]);
    let clipboard_button = if can_export {
        clipboard_button.on_press(Message::ExportToClipboard)
    } else {
        clipboard_button // Disabled - no on_press
    };

    Row::new()
        .spacing(spacing::SM)
        .push(file_button)
        .push(clipboard_button)
        .into()
}

/// Render the diagnostics screen.
#[must_use]
#[allow(clippy::needless_pass_by_value)] // ViewContext is small and consumed
pub fn view(ctx: ViewContext<'_>) -> Element<'_, Message> {
    let back_button = button(
        text(format!("← {}", ctx.i18n.tr("diagnostics-back-button"))).size(typography::BODY),
    )
    .on_press(Message::BackToViewer);

    let title = Text::new(ctx.i18n.tr("diagnostics-title")).size(typography::TITLE_LG);

    let info_section = build_info_section(&ctx);
    let toggle_section = build_toggle_section(&ctx);
    let status_section = build_status_section(&ctx);
    let export_section = build_export_section(&ctx);

    let content = Column::new()
        .width(Length::Fill)
        .spacing(spacing::LG)
        .align_x(Horizontal::Left)
        .padding(spacing::MD)
        .push(back_button)
        .push(title)
        .push(info_section)
        .push(toggle_section)
        .push(status_section)
        .push(export_section);

    scrollable(content).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::fluent::I18n;
    use std::time::Instant;

    #[test]
    fn diagnostics_view_renders() {
        let i18n = I18n::default();
        let ctx = ViewContext {
            i18n: &i18n,
            status: CollectionStatus::Disabled,
            event_count: 0,
            collection_duration: Duration::from_secs(0),
            is_dark_theme: false,
        };
        let _element = view(ctx);
    }

    #[test]
    fn back_to_viewer_emits_event() {
        let event = update(&Message::BackToViewer);
        assert!(matches!(event, Event::BackToViewer));
    }

    #[test]
    fn refresh_status_emits_none() {
        let event = update(&Message::RefreshStatus);
        assert!(matches!(event, Event::None));
    }

    #[test]
    fn toggle_resource_collection_emits_event() {
        let event = update(&Message::ToggleResourceCollection(true));
        assert!(matches!(event, Event::ToggleResourceCollection(true)));

        let event = update(&Message::ToggleResourceCollection(false));
        assert!(matches!(event, Event::ToggleResourceCollection(false)));
    }

    #[test]
    fn export_to_file_emits_event() {
        let event = update(&Message::ExportToFile);
        assert!(matches!(event, Event::ExportToFile));
    }

    #[test]
    fn export_to_clipboard_emits_event() {
        let event = update(&Message::ExportToClipboard);
        assert!(matches!(event, Event::ExportToClipboard));
    }

    #[test]
    fn format_duration_under_one_hour() {
        let duration = Duration::from_secs(5 * 60 + 32); // 5m 32s
        assert_eq!(format_duration(duration), "5m 32s");
    }

    #[test]
    fn format_duration_over_one_hour() {
        let duration = Duration::from_secs(2 * 3600 + 15 * 60 + 45); // 2h 15m 45s
        assert_eq!(format_duration(duration), "2h 15m 45s");
    }

    #[test]
    fn format_duration_zero() {
        let duration = Duration::from_secs(0);
        assert_eq!(format_duration(duration), "0m 0s");
    }

    #[test]
    fn status_color_for_enabled() {
        let status = CollectionStatus::Enabled {
            started_at: Instant::now(),
        };
        assert_eq!(status_color(&status), palette::SUCCESS_500);
    }

    #[test]
    fn status_color_for_disabled() {
        let status = CollectionStatus::Disabled;
        assert_eq!(status_color(&status), palette::GRAY_400);
    }

    #[test]
    fn status_color_for_error() {
        let status = CollectionStatus::Error {
            message: "test".to_string(),
        };
        assert_eq!(status_color(&status), palette::ERROR_500);
    }
}
