// SPDX-License-Identifier: MPL-2.0
//! Tooltip styles with improved visibility.
//!
//! Provides styled tooltips with proper contrast, shadows, and rounded corners
//! following UX best practices for accessibility.

use crate::ui::design_tokens::{radius, spacing, typography};
use iced::widget::{container, tooltip, Container, Text};
use iced::{Background, Border, Color, Element, Shadow, Theme, Vector};

/// Style for tooltip container with good contrast and shadow.
///
/// Automatically adapts to light/dark theme for optimal visibility.
pub fn tooltip_container(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    // Determine if we're in dark mode by checking the background luminance
    let bg = palette.background.base.color;
    let is_dark = (bg.r + bg.g + bg.b) / 3.0 < 0.5;

    let (bg_color, text_color, border_color) = if is_dark {
        // Dark theme: light tooltip for contrast
        (
            Color::from_rgba(0.95, 0.95, 0.95, 0.98),
            Color::from_rgb(0.1, 0.1, 0.1),
            Color::from_rgba(0.7, 0.7, 0.7, 0.3),
        )
    } else {
        // Light theme: dark tooltip for contrast
        (
            Color::from_rgba(0.15, 0.15, 0.15, 0.98),
            Color::from_rgb(0.95, 0.95, 0.95),
            Color::from_rgba(0.3, 0.3, 0.3, 0.3),
        )
    };

    container::Style {
        background: Some(Background::Color(bg_color)),
        border: Border {
            radius: radius::SM.into(),
            width: 1.0,
            color: border_color,
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        },
        text_color: Some(text_color),
        ..Default::default()
    }
}

/// Creates a styled tooltip with improved visibility.
///
/// The tooltip has:
/// - Opaque background with good contrast (adapts to theme)
/// - Subtle shadow for separation from content
/// - Rounded corners matching the design system
/// - Consistent padding and gap
///
/// # Example
///
/// ```ignore
/// use crate::ui::styles::tooltip;
///
/// tooltip::styled(
///     my_button,
///     "Click to save",
///     tooltip::Position::Bottom,
/// )
/// ```
pub fn styled<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    tip: impl Into<String>,
    position: tooltip::Position,
) -> tooltip::Tooltip<'a, Message, Theme, iced::Renderer> {
    let tip_container = Container::new(Text::new(tip.into()).size(typography::BODY_SM))
        .padding(spacing::XS)
        .style(tooltip_container);

    tooltip(content, tip_container, position).gap(spacing::XS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tooltip_container_has_background() {
        let light = tooltip_container(&Theme::Light);
        let dark = tooltip_container(&Theme::Dark);

        assert!(light.background.is_some());
        assert!(dark.background.is_some());
    }

    #[test]
    fn tooltip_container_has_text_color() {
        let light = tooltip_container(&Theme::Light);
        let dark = tooltip_container(&Theme::Dark);

        assert!(light.text_color.is_some());
        assert!(dark.text_color.is_some());
    }

    #[test]
    fn tooltip_container_has_shadow() {
        let style = tooltip_container(&Theme::Light);
        // Shadow blur radius should be positive
        assert!(style.shadow.blur_radius > 0.0);
    }

    #[test]
    fn light_theme_uses_dark_tooltip() {
        let style = tooltip_container(&Theme::Light);
        let Some(Background::Color(bg)) = style.background else {
            panic!("Expected color background")
        };
        // Dark tooltip on light theme: low RGB values
        assert!(bg.r < 0.5);
    }

    #[test]
    fn dark_theme_uses_light_tooltip() {
        let style = tooltip_container(&Theme::Dark);
        let Some(Background::Color(bg)) = style.background else {
            panic!("Expected color background")
        };
        // Light tooltip on dark theme: high RGB values
        assert!(bg.r > 0.5);
    }
}
