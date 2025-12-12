// SPDX-License-Identifier: MPL-2.0
//! Shared styles and constants for viewer components (image and video).
//!
//! This module centralizes visual design tokens (colors, spacing, sizes) to ensure
//! consistency between image viewer and video player controls, following the DRY principle.

use crate::ui::design_tokens::{opacity, palette, sizing, spacing};
use iced::Color;

/// Control bar spacing between elements (8px)
pub const CONTROL_SPACING: f32 = spacing::XS;

/// Control bar padding (16px)
pub const CONTROL_PADDING: f32 = spacing::MD;

/// Standard icon button size (32x32px)
pub const ICON_SIZE: f32 = sizing::ICON_LG;

/// Navigation arrow size (48x48px for better touch targets)
pub const NAV_ARROW_SIZE: f32 = sizing::ICON_XL;

/// Semi-transparent overlay background for fullscreen controls.
pub const OVERLAY_BACKGROUND: Color = Color {
    a: opacity::OVERLAY_STRONG,
    ..palette::BLACK
};

/// Transparent background for HUD
pub const HUD_BACKGROUND: Color = Color {
    a: opacity::OVERLAY_MEDIUM,
    ..palette::BLACK
};

/// HUD text color (white).
pub const HUD_TEXT_COLOR: Color = palette::WHITE;

/// Control bar background (semi-transparent dark)
pub const CONTROL_BAR_BACKGROUND: Color = Color {
    a: opacity::OVERLAY_PRESSED,
    ..palette::GRAY_900
};

/// Control bar text/icon color (white).
pub const CONTROL_BAR_ICON_COLOR: Color = palette::WHITE;

/// Control bar border color (subtle gray)
pub const CONTROL_BAR_BORDER: Color = palette::GRAY_700;

/// Timeline scrubber thumb size (12px diameter)
pub const SCRUBBER_THUMB_SIZE: f32 = sizing::SCRUBBER_THUMB;

/// Timeline track height (4px)
pub const TIMELINE_TRACK_HEIGHT: f32 = sizing::TIMELINE_TRACK;

/// Timeline progress color (white).
pub const TIMELINE_PROGRESS_COLOR: Color = palette::WHITE;

/// Timeline background color (gray)
pub const TIMELINE_BACKGROUND_COLOR: Color = palette::GRAY_400;

// Compile-time assertions to validate constants
const _: () = {
    assert!(CONTROL_SPACING > 0.0);
    assert!(CONTROL_PADDING > 0.0);
    assert!(ICON_SIZE > 0.0);
    assert!(NAV_ARROW_SIZE > 0.0);
    assert!(SCRUBBER_THUMB_SIZE > 0.0);
    assert!(TIMELINE_TRACK_HEIGHT > 0.0);
    assert!(NAV_ARROW_SIZE > ICON_SIZE);
    assert!(OVERLAY_BACKGROUND.a < 1.0);
    assert!(OVERLAY_BACKGROUND.a > 0.0);
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::design_tokens::{sizing, spacing};

    #[test]
    fn constants_use_design_tokens() {
        assert_eq!(CONTROL_SPACING, spacing::XS);
        assert_eq!(ICON_SIZE, sizing::ICON_LG);
    }
}
