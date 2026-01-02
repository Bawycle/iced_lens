// SPDX-License-Identifier: MPL-2.0
//! Window/application icon loading.
//! Uses the project SVG and rasterizes it at runtime to produce a RGBA icon
//! for the window title bar. Falls back to `None` if rendering fails.

use iced::window::{icon, Icon};
use resvg::usvg;

/// Rasterize the embedded SVG icon to a 128x128 RGBA buffer.
/// Returns `None` if parsing or rendering fails.
#[must_use]
pub fn load_window_icon() -> Option<Icon> {
    // Embed the SVG so packaging does not need to locate assets on disk.
    const SVG_SOURCE: &str = include_str!("../assets/branding/iced_lens.svg");
    // Target size (128 chosen to fit in u8 for lossless f32 conversion)
    const TARGET_SIZE_U8: u8 = 128;
    const TARGET_SIZE: u32 = TARGET_SIZE_U8 as u32;

    // Parse SVG using usvg (via resvg)
    let Ok(tree) = usvg::Tree::from_data(SVG_SOURCE.as_bytes(), &usvg::Options::default()) else {
        return None;
    };

    let orig_size = tree.size();
    let scale_x = f32::from(TARGET_SIZE_U8) / orig_size.width();
    let scale_y = f32::from(TARGET_SIZE_U8) / orig_size.height();
    let transform = tiny_skia::Transform::from_scale(scale_x, scale_y);

    let mut pixmap = tiny_skia::Pixmap::new(TARGET_SIZE, TARGET_SIZE)?;

    resvg::render(&tree, transform, &mut pixmap.as_mut());

    let data = pixmap.data();
    icon::from_rgba(data.to_vec(), TARGET_SIZE, TARGET_SIZE).ok()
}
