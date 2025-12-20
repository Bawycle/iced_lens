// SPDX-License-Identifier: MPL-2.0
#![doc = r#"
# Design Tokens

This module defines all of the application's design tokens, following the W3C Design Tokens standard.

## Organization

- **Palette**: Base colors
- **Opacity**: Standardized opacity levels
- **Spacing**: Spacing scale (8px grid)
- **Sizing**: Component sizes
- **Typography**: Font size scale
- **Border**: Border width scale
- **Radius**: Border radii
- **Shadow**: Shadow definitions

## Examples

```
use iced_lens::ui::design_tokens::{palette, spacing, opacity};
use iced::Color;

// Create an overlay color
let overlay_bg = Color {
    a: opacity::OVERLAY_STRONG,
    ..palette::BLACK
};

// Use the spacing scale
let padding = spacing::MD; // 16px
```

## Modification

⚠️ Tokens are designed to be consistent. Before modifying:
1. Check the impact on all components
2. Maintain ratios (e.g., MD = XS * 2)
3. Run validation tests
"#]

//! Design tokens centralisés suivant le Design Tokens W3C standard.

use iced::Color;

// ============================================================================
// Color Palette
// ============================================================================

pub mod palette {
    use super::Color;

    // Grayscale
    pub const BLACK: Color = Color::BLACK;
    pub const WHITE: Color = Color::WHITE;
    pub const GRAY_900: Color = Color::from_rgb(0.1, 0.1, 0.1);
    pub const GRAY_700: Color = Color::from_rgb(0.3, 0.3, 0.3);
    pub const GRAY_400: Color = Color::from_rgb(0.4, 0.4, 0.4);
    pub const GRAY_200: Color = Color::from_rgb(0.75, 0.75, 0.75);
    pub const GRAY_100: Color = Color::from_rgb(0.85, 0.85, 0.85);

    // Brand colors (blue scale)
    pub const PRIMARY_100: Color = Color::from_rgb(0.85, 0.92, 1.0); // Very light blue
    pub const PRIMARY_200: Color = Color::from_rgb(0.7, 0.84, 0.98); // Light blue
    pub const PRIMARY_400: Color = Color::from_rgb(0.4, 0.7, 1.0); // Medium light blue
    pub const PRIMARY_500: Color = Color::from_rgb(0.3, 0.6, 0.9); // Primary blue
    pub const PRIMARY_600: Color = Color::from_rgb(0.2, 0.5, 0.8); // Medium dark blue
    pub const PRIMARY_700: Color = Color::from_rgb(0.15, 0.4, 0.7); // Dark blue
    pub const PRIMARY_800: Color = Color::from_rgb(0.1, 0.3, 0.6); // Very dark blue

    // Semantic colors
    pub const ERROR_500: Color = Color::from_rgb(0.898, 0.224, 0.208);
    pub const WARNING_500: Color = Color::from_rgb(0.945, 0.651, 0.125);
    pub const SUCCESS_500: Color = Color::from_rgb(0.263, 0.702, 0.404);
    pub const INFO_500: Color = Color::from_rgb(0.392, 0.588, 1.0);
}

// ============================================================================
// Opacity Scale
// ============================================================================

pub mod opacity {
    pub const TRANSPARENT: f32 = 0.0;
    pub const OVERLAY_SUBTLE: f32 = 0.2;
    pub const OVERLAY_MEDIUM: f32 = 0.5;
    pub const OVERLAY_STRONG: f32 = 0.7;
    pub const OVERLAY_HOVER: f32 = 0.8;
    pub const OVERLAY_PRESSED: f32 = 0.9;
    pub const OPAQUE: f32 = 1.0;

    /// Surface background - Semi-transparent panels and containers
    pub const SURFACE: f32 = 0.95;
}

// ============================================================================
// Spacing Scale (8px baseline grid)
// ============================================================================

pub mod spacing {
    pub const XXS: f32 = 4.0; // 0.5 unit
    pub const XS: f32 = 8.0; // 1 unit
    pub const SM: f32 = 12.0; // 1.5 units
    pub const MD: f32 = 16.0; // 2 units
    pub const LG: f32 = 24.0; // 3 units
    pub const XL: f32 = 32.0; // 4 units
    pub const XXL: f32 = 48.0; // 6 units
}

// ============================================================================
// Sizing Scale
// ============================================================================

pub mod sizing {
    // Icon sizes
    pub const ICON_SM: f32 = 16.0;
    pub const ICON_MD: f32 = 24.0;
    pub const ICON_LG: f32 = 32.0;
    pub const ICON_XL: f32 = 48.0;
    pub const ICON_XXL: f32 = 64.0;

    // Interactive element heights
    pub const BUTTON_HEIGHT: f32 = 36.0;
    pub const INPUT_HEIGHT: f32 = 40.0;

    // Video controls
    pub const SCRUBBER_THUMB: f32 = 12.0;
    pub const TIMELINE_TRACK: f32 = 4.0;

    // Component widths
    pub const SIDEBAR_WIDTH: f32 = 290.0;
    pub const TOAST_WIDTH: f32 = 320.0;

    // Crop overlay handles
    /// Visual size of crop handles (rendered size)
    pub const CROP_HANDLE_SIZE: f32 = 16.0;
    /// Hit area for crop handles - WCAG 2.5.5 compliant (44x44 minimum)
    pub const CROP_HANDLE_HIT_SIZE: f32 = 44.0;
}

// ============================================================================
// Typography Scale
// ============================================================================

pub mod typography {
    //! Font size scale following Material Design 3 type scale principles.
    //!
    //! The scale provides semantic sizes for consistent text hierarchy:
    //! - Titles: Large headings (pages, dialogs)
    //! - Body: Primary content text
    //! - Caption: Secondary, supporting text

    /// Large title - Main page headings (Settings, Help, About)
    pub const TITLE_LG: f32 = 30.0;

    /// Medium title - App name, prominent labels
    pub const TITLE_MD: f32 = 20.0;

    /// Small title - Section headers
    pub const TITLE_SM: f32 = 18.0;

    /// Large body - Form inputs, emphasis text
    pub const BODY_LG: f32 = 16.0;

    /// Standard body - Most UI text, labels, descriptions
    pub const BODY: f32 = 14.0;

    /// Small body - Hints, secondary labels
    pub const BODY_SM: f32 = 13.0;

    /// Caption - Badges, timestamps, small info
    pub const CAPTION: f32 = 12.0;
}

// ============================================================================
// Border Scale
// ============================================================================

pub mod border {
    /// Thin border - Subtle separators, input fields
    pub const WIDTH_SM: f32 = 1.0;

    /// Medium border - Emphasis borders, toast accents
    pub const WIDTH_MD: f32 = 2.0;
}

// ============================================================================
// Border Radius Scale
// ============================================================================

pub mod radius {
    pub const NONE: f32 = 0.0;
    pub const SM: f32 = 4.0;
    pub const MD: f32 = 8.0;
    pub const LG: f32 = 12.0;
    pub const FULL: f32 = 9999.0; // Pill shape
}

// ============================================================================
// Shadow Definitions
// ============================================================================

pub mod shadow {
    use super::palette;
    use iced::{Shadow, Vector};

    pub const NONE: Shadow = Shadow {
        color: palette::BLACK,
        offset: Vector::ZERO,
        blur_radius: 0.0,
    };

    pub const SM: Shadow = Shadow {
        color: palette::BLACK,
        offset: Vector { x: 0.0, y: 2.0 },
        blur_radius: 4.0,
    };

    pub const MD: Shadow = Shadow {
        color: palette::BLACK,
        offset: Vector { x: 0.0, y: 4.0 },
        blur_radius: 8.0,
    };

    pub const LG: Shadow = Shadow {
        color: palette::BLACK,
        offset: Vector { x: 0.0, y: 8.0 },
        blur_radius: 16.0,
    };
}

// ============================================================================
// Compile-time Validation
// ============================================================================

const _: () = {
    // Spacing validation
    assert!(spacing::XS > 0.0);
    assert!(spacing::SM > spacing::XS);
    assert!(spacing::MD > spacing::SM);
    assert!(spacing::LG > spacing::MD);

    // Opacity validation
    assert!(opacity::TRANSPARENT == 0.0);
    assert!(opacity::OPAQUE == 1.0);
    assert!(opacity::OVERLAY_MEDIUM > 0.0 && opacity::OVERLAY_MEDIUM < 1.0);
    assert!(opacity::SURFACE > 0.0 && opacity::SURFACE < 1.0);

    // Sizing validation
    assert!(sizing::ICON_XL > sizing::ICON_LG);
    assert!(sizing::ICON_LG > sizing::ICON_MD);

    // Typography validation
    assert!(typography::TITLE_LG > typography::TITLE_MD);
    assert!(typography::TITLE_MD > typography::TITLE_SM);
    assert!(typography::TITLE_SM > typography::BODY_LG);
    assert!(typography::BODY > typography::BODY_SM);
    assert!(typography::BODY_SM > typography::CAPTION);

    // Border validation
    assert!(border::WIDTH_MD > border::WIDTH_SM);

    // Color validation
    assert!(palette::PRIMARY_500.r >= 0.0 && palette::PRIMARY_500.r <= 1.0);
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spacing_scale_is_consistent() {
        assert_eq!(spacing::MD, spacing::XS * 2.0);
        assert_eq!(spacing::LG, spacing::MD * 1.5);
    }
}
