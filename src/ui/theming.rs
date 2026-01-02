// SPDX-License-Identifier: MPL-2.0
//! Extensible theming system.

use crate::ui::design_tokens::{opacity, palette};
use dark_light;
use iced::Color;
use serde::{Deserialize, Serialize};

/// Color palette for a theme.
#[derive(Debug, Clone)]
pub struct ColorScheme {
    // Surface colors
    pub surface_primary: Color,
    pub surface_secondary: Color,
    pub surface_tertiary: Color,

    // Text colors
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_tertiary: Color,

    // Brand colors
    pub brand_primary: Color,
    pub brand_secondary: Color,

    // Semantic colors
    pub error: Color,
    pub warning: Color,
    pub success: Color,
    pub info: Color,

    // Overlay colors
    pub overlay_background: Color,
    pub overlay_text: Color,
}

impl ColorScheme {
    /// Light theme (Light mode).
    #[must_use]
    pub fn light() -> Self {
        Self {
            surface_primary: palette::WHITE,
            surface_secondary: palette::GRAY_100,
            surface_tertiary: palette::GRAY_200,

            text_primary: palette::GRAY_900,
            text_secondary: palette::GRAY_700,
            text_tertiary: palette::GRAY_400,

            brand_primary: palette::PRIMARY_500,
            brand_secondary: palette::PRIMARY_600,

            error: palette::ERROR_500,
            warning: palette::WARNING_500,
            success: palette::SUCCESS_500,
            info: palette::INFO_500,

            overlay_background: Color {
                a: opacity::OVERLAY_STRONG,
                ..palette::BLACK
            },
            overlay_text: palette::WHITE,
        }
    }

    /// Dark theme (Dark mode).
    #[must_use]
    pub fn dark() -> Self {
        Self {
            surface_primary: palette::GRAY_900,
            surface_secondary: Color::from_rgb(0.15, 0.15, 0.15),
            surface_tertiary: Color::from_rgb(0.2, 0.2, 0.2),

            text_primary: palette::WHITE,
            text_secondary: palette::GRAY_200,
            text_tertiary: palette::GRAY_400,

            brand_primary: palette::PRIMARY_400,
            brand_secondary: palette::PRIMARY_500,

            error: palette::ERROR_500,
            warning: palette::WARNING_500,
            success: palette::SUCCESS_500,
            info: palette::INFO_500,

            overlay_background: Color {
                a: opacity::OVERLAY_HOVER,
                ..palette::BLACK
            },
            overlay_text: palette::WHITE,
        }
    }

    /// Detects the system theme and returns the appropriate `ColorScheme`.
    #[must_use]
    pub fn from_system() -> Self {
        if let Ok(dark_light::Mode::Light) = dark_light::detect() {
            Self::light()
        } else {
            Self::dark() // Default to dark for Dark mode or on error
        }
    }
}

/// Configuration de thème globale.
#[derive(Debug, Clone)]
pub struct AppTheme {
    pub colors: ColorScheme,
    pub mode: ThemeMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    Light,
    Dark,
    #[default]
    System,
}

impl ThemeMode {
    /// Returns true if the effective theme is dark.
    /// For System mode, detects the actual system theme.
    #[must_use]
    pub fn is_dark(self) -> bool {
        match self {
            ThemeMode::Light => false,
            ThemeMode::Dark => true,
            ThemeMode::System => {
                // Detect system theme; default to dark on detection error
                !matches!(dark_light::detect(), Ok(dark_light::Mode::Light))
            }
        }
    }
}

impl AppTheme {
    #[must_use]
    pub fn new(mode: ThemeMode) -> Self {
        let colors = match mode {
            ThemeMode::Light => ColorScheme::light(),
            ThemeMode::Dark => ColorScheme::dark(),
            ThemeMode::System => ColorScheme::from_system(),
        };

        Self { colors, mode }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_theme_has_light_surface() {
        let scheme = ColorScheme::light();
        assert!(scheme.surface_primary.r > 0.9); // Close to white
    }

    #[test]
    fn dark_theme_has_dark_surface() {
        let scheme = ColorScheme::dark();
        assert!(scheme.surface_primary.r < 0.2); // Close to black
    }

    #[test]
    fn both_themes_have_same_brand_hue() {
        let light = ColorScheme::light();
        let dark = ColorScheme::dark();

        // Brand colors should be similar (same hue)
        // Simplified test: ensure they are not grayscale
        assert!(light.brand_primary.b > light.brand_primary.r);
        assert!(dark.brand_primary.b > dark.brand_primary.r);
    }

    #[test]
    fn theme_mode_is_dark_returns_correct_values() {
        assert!(!ThemeMode::Light.is_dark());
        assert!(ThemeMode::Dark.is_dark());
        // System mode depends on actual system theme, so we just verify it doesn't panic
        let _ = ThemeMode::System.is_dark();
    }
}
