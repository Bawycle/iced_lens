// SPDX-License-Identifier: MPL-2.0
//! Diagnostics screen module for viewing and exporting diagnostic data.
//!
//! This module provides a UI for accessing diagnostic controls, viewing
//! collection status, and exporting diagnostic reports.

use std::time::Duration;

use crate::diagnostics::CollectionStatus;
use crate::i18n::fluent::I18n;
use crate::ui::design_tokens::{palette, radius, spacing, typography};
use iced::{
    alignment::Horizontal,
    widget::{button, container, scrollable, text, Column, Row, Text},
    Border, Color, Element, Length,
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
}

/// Messages emitted by the diagnostics screen.
#[derive(Debug, Clone)]
pub enum Message {
    BackToViewer,
    /// Refresh status on subscription tick.
    RefreshStatus,
}

/// Events propagated to the parent application.
#[derive(Debug, Clone)]
pub enum Event {
    None,
    BackToViewer,
}

/// Process a diagnostics screen message and return the corresponding event.
#[must_use]
pub fn update(message: &Message) -> Event {
    match message {
        Message::BackToViewer => Event::BackToViewer,
        Message::RefreshStatus => Event::None,
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

/// Builds the status section showing collection state, duration, and buffer count.
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

    // Duration text
    let duration_str = format_duration(ctx.collection_duration);
    let duration_text = ctx
        .i18n
        .tr_with_args("diagnostics-running-for", &[("duration", &duration_str)]);

    // Buffer count text
    let count_str = ctx.event_count.to_string();
    let buffer_text = ctx
        .i18n
        .tr_with_args("diagnostics-buffer-count", &[("count", &count_str)]);

    Column::new()
        .spacing(spacing::SM)
        .push(status_row)
        .push(Text::new(duration_text).size(typography::BODY))
        .push(Text::new(buffer_text).size(typography::BODY))
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

    let status_section = build_status_section(&ctx);

    let content = Column::new()
        .width(Length::Fill)
        .spacing(spacing::LG)
        .align_x(Horizontal::Left)
        .padding(spacing::MD)
        .push(back_button)
        .push(title)
        .push(status_section);

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
