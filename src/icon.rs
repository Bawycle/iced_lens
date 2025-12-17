// SPDX-License-Identifier: MPL-2.0
//! Window/application icon loading.
//! Uses the project SVG and rasterizes it at runtime to produce a RGBA icon
//! for the window title bar. Falls back to `None` if rendering fails.

use iced::window::{icon, Icon};
use resvg::usvg;

/// Rasterize the embedded SVG icon to a 128x128 RGBA buffer.
/// Returns `None` if parsing or rendering fails.
pub fn load_window_icon() -> Option<Icon> {
    // Embed the SVG so packaging does not need to locate assets on disk.
    const SVG_SOURCE: &str = include_str!("../assets/branding/iced_lens.svg");

    // Parse SVG using usvg (via resvg)
    let tree = match usvg::Tree::from_data(SVG_SOURCE.as_bytes(), &usvg::Options::default()) {
        Ok(t) => t,
        Err(_) => return None,
    };

    // Target size
    let target = 128u32;
    let orig_size = tree.size();
    let scale_x = target as f32 / orig_size.width();
    let scale_y = target as f32 / orig_size.height();
    let transform = tiny_skia::Transform::from_scale(scale_x, scale_y);

    let mut pixmap = tiny_skia::Pixmap::new(target, target)?;

    resvg::render(&tree, transform, &mut pixmap.as_mut());

    let data = pixmap.data();
    icon::from_rgba(data.to_vec(), target, target).ok()
}
