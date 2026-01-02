// SPDX-License-Identifier: MPL-2.0
//! About screen module displaying application information and licenses.
//!
//! This module shows application details, copyright information, license
//! notices (MPL-2.0 for the code, custom license for the icon), credits
//! for dependencies, and links to the project repository.

use crate::i18n::fluent::I18n;
use crate::ui::design_tokens::{radius, sizing, spacing, typography};
use crate::ui::icons;
use iced::{
    alignment::{Horizontal, Vertical},
    widget::{button, container, rule, scrollable, text, Column, Container, Row, Text},
    Border, Element, Length, Theme,
};

/// Application version from Cargo.toml.
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Repository URL.
const REPOSITORY_URL: &str = "https://codeberg.org/Bawycle/iced_lens";

/// Issues URL.
const ISSUES_URL: &str = "https://codeberg.org/Bawycle/iced_lens/issues";

/// Dependencies list URL (Cargo.toml).
const DEPENDENCIES_URL: &str =
    "https://codeberg.org/Bawycle/iced_lens/src/branch/master/Cargo.toml";

/// Third-party licenses file URL.
const THIRD_PARTY_LICENSES_URL: &str =
    "https://codeberg.org/Bawycle/iced_lens/src/branch/master/THIRD_PARTY_LICENSES.md";

/// Contextual data needed to render the about screen.
pub struct ViewContext<'a> {
    pub i18n: &'a I18n,
}

/// Messages emitted by the about screen.
#[derive(Debug, Clone)]
pub enum Message {
    BackToViewer,
}

/// Events propagated to the parent application.
#[derive(Debug, Clone)]
pub enum Event {
    None,
    BackToViewer,
}

/// Process an about screen message and return the corresponding event.
#[must_use]
pub fn update(message: &Message) -> Event {
    match message {
        Message::BackToViewer => Event::BackToViewer,
    }
}

/// Render the about screen.
#[must_use]
#[allow(clippy::needless_pass_by_value)] // ViewContext is small and consumed
pub fn view(ctx: ViewContext<'_>) -> Element<'_, Message> {
    let back_button = button(
        text(format!("← {}", ctx.i18n.tr("about-back-to-viewer-button"))).size(typography::BODY),
    )
    .on_press(Message::BackToViewer);

    let title = Text::new(ctx.i18n.tr("about-title")).size(typography::TITLE_LG);

    // Build sections
    let app_section = build_app_section(&ctx);
    let license_section = build_license_section(&ctx);
    let icon_license_section = build_icon_license_section(&ctx);
    let credits_section = build_credits_section(&ctx);
    let third_party_section = build_third_party_section(&ctx);
    let links_section = build_links_section(&ctx);

    let content = Column::new()
        .width(Length::Fill)
        .spacing(spacing::LG)
        .align_x(Horizontal::Left)
        .padding(spacing::MD)
        .push(back_button)
        .push(title)
        .push(app_section)
        .push(license_section)
        .push(icon_license_section)
        .push(credits_section)
        .push(third_party_section)
        .push(links_section);

    scrollable(content).into()
}

/// Build the application info section.
fn build_app_section<'a>(ctx: &ViewContext<'a>) -> Element<'a, Message> {
    let app_name = Text::new(ctx.i18n.tr("about-app-name")).size(typography::TITLE_MD);
    let version = Text::new(format!("v{APP_VERSION}")).size(typography::BODY);
    let description = Text::new(ctx.i18n.tr("about-app-description")).size(typography::BODY);

    let content = Column::new()
        .spacing(spacing::XS)
        .push(
            Row::new()
                .spacing(spacing::SM)
                .align_y(Vertical::Center)
                .push(app_name)
                .push(version),
        )
        .push(description);

    build_section(
        icons::info(),
        ctx.i18n.tr("about-section-app"),
        content.into(),
    )
}

/// Build the license section (MPL-2.0).
fn build_license_section<'a>(ctx: &ViewContext<'a>) -> Element<'a, Message> {
    let license_name = Text::new(ctx.i18n.tr("about-license-name")).size(typography::BODY_LG);
    let license_summary = Text::new(ctx.i18n.tr("about-license-summary")).size(typography::BODY);

    let content = Column::new()
        .spacing(spacing::SM)
        .push(license_name)
        .push(license_summary);

    build_section(
        icons::globe(),
        ctx.i18n.tr("about-section-license"),
        content.into(),
    )
}

/// Build the icon license section.
fn build_icon_license_section<'a>(ctx: &ViewContext<'a>) -> Element<'a, Message> {
    let license_name = Text::new(ctx.i18n.tr("about-icon-license-name")).size(typography::BODY_LG);
    let license_summary =
        Text::new(ctx.i18n.tr("about-icon-license-summary")).size(typography::BODY);

    let content = Column::new()
        .spacing(spacing::SM)
        .push(license_name)
        .push(license_summary);

    build_section(
        icons::image(),
        ctx.i18n.tr("about-section-icon-license"),
        content.into(),
    )
}

/// Build the credits section.
fn build_credits_section<'a>(ctx: &ViewContext<'a>) -> Element<'a, Message> {
    let content = Column::new()
        .spacing(spacing::XS)
        .push(build_credit_item(&ctx.i18n.tr("about-credits-iced")))
        .push(build_credit_item(&ctx.i18n.tr("about-credits-ffmpeg")))
        .push(build_credit_item(&ctx.i18n.tr("about-credits-onnx")))
        .push(build_credit_item(&ctx.i18n.tr("about-credits-fluent")))
        .push(build_link_item(
            &ctx.i18n.tr("about-credits-full-list"),
            DEPENDENCIES_URL,
        ));

    build_section(
        icons::cog(),
        ctx.i18n.tr("about-section-credits"),
        content.into(),
    )
}

/// Build the third-party licenses section.
fn build_third_party_section<'a>(ctx: &ViewContext<'a>) -> Element<'a, Message> {
    let content = Column::new()
        .spacing(spacing::XS)
        .push(build_credit_item(&ctx.i18n.tr("about-third-party-ffmpeg")))
        .push(build_credit_item(&ctx.i18n.tr("about-third-party-onnx")))
        .push(build_link_item(
            &ctx.i18n.tr("about-third-party-details"),
            THIRD_PARTY_LICENSES_URL,
        ));

    build_section(
        icons::globe(),
        ctx.i18n.tr("about-section-third-party"),
        content.into(),
    )
}

/// Build a single credit item.
fn build_credit_item<'a>(description: &str) -> Element<'a, Message> {
    Text::new(format!("• {description}"))
        .size(typography::BODY)
        .into()
}

/// Build the links section.
fn build_links_section<'a>(ctx: &ViewContext<'a>) -> Element<'a, Message> {
    let repo_label = ctx.i18n.tr("about-link-repository");
    let issues_label = ctx.i18n.tr("about-link-issues");

    let content = Column::new()
        .spacing(spacing::SM)
        .push(build_link_item(&repo_label, REPOSITORY_URL))
        .push(build_link_item(&issues_label, ISSUES_URL));

    build_section(
        icons::globe(),
        ctx.i18n.tr("about-section-links"),
        content.into(),
    )
}

/// Build a link item with label and URL.
fn build_link_item<'a>(label: &str, url: &'a str) -> Element<'a, Message> {
    Row::new()
        .spacing(spacing::SM)
        .push(Text::new(format!("{label}:")).size(typography::BODY))
        .push(Text::new(url).size(typography::BODY))
        .into()
}

/// Build a section with icon, title, and content (same pattern as settings/help).
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::fluent::I18n;

    #[test]
    fn about_view_renders() {
        let i18n = I18n::default();
        let ctx = ViewContext { i18n: &i18n };
        let _element = view(ctx);
    }

    #[test]
    fn back_to_viewer_emits_event() {
        let event = update(&Message::BackToViewer);
        assert!(matches!(event, Event::BackToViewer));
    }

    #[test]
    fn app_version_is_valid() {
        assert!(!APP_VERSION.is_empty());
    }
}
