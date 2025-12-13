// SPDX-License-Identifier: MPL-2.0
//! Integration tests to validate style and design token coherence.

#[cfg(test)]
mod tests {
    use iced::Theme;
    use iced_lens::ui::design_tokens::{opacity, palette, sizing, spacing};
    use iced_lens::ui::styles::button;
    use iced_lens::ui::theming::{AppTheme, ThemeMode};

    #[test]
    fn all_button_styles_compile() {
        let theme = Theme::Dark;

        // Smoke-test all button styles compile and are callable
        let _ = button::primary(&theme, iced::widget::button::Status::Active);
        let _ = button::overlay(palette::WHITE, 0.5, 0.8);
        let _ = button::video_play_overlay();
    }

    #[test]
    fn design_tokens_are_accessible() {
        // Palette
        let _ = palette::PRIMARY_500;
        let _ = palette::WHITE;

        // Spacing
        let _ = spacing::MD;

        // Opacity
        let _ = opacity::OVERLAY_STRONG;

        // Sizing
        let _ = sizing::ICON_LG;
    }

    #[test]
    fn theming_switches_correctly() {
        let light = AppTheme::new(ThemeMode::Light);
        let dark = AppTheme::new(ThemeMode::Dark);

        // Surface colors should be visually opposite between light and dark
        assert!(light.colors.surface_primary.r > dark.colors.surface_primary.r);

        // Text colors should also be opposite between light and dark
        assert!(light.colors.text_primary.r < dark.colors.text_primary.r);
    }
}
