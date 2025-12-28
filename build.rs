// SPDX-License-Identifier: MPL-2.0
//! Build script for platform-specific resources and icon generation.
//!
//! This script:
//! - On Windows, embeds the application icon into the executable
//! - On all platforms, generates PNG icons from SVG sources

use std::fs;
use std::path::Path;

/// Icon size in pixels (width and height).
const ICON_SIZE: u32 = 32;

fn main() {
    // Windows-specific: embed application icon
    #[cfg(target_os = "windows")]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/branding/iced_lens.ico");
        res.compile().expect("Failed to compile Windows resources");
    }

    // Generate PNG icons from SVG sources
    generate_icons();
}

/// Generates PNG icons from SVG sources at compile time.
///
/// Icons are rendered to the `OUT_DIR` for inclusion via `include_bytes!`.
/// Dark icons (black on transparent) are created by inverting the white SVGs.
/// Light icons (white on transparent) are direct renders of the SVGs.
fn generate_icons() {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    let icons_dir = Path::new(&out_dir).join("icons");
    let dark_dir = icons_dir.join("dark");
    let light_dir = icons_dir.join("light");

    // Create output directories
    fs::create_dir_all(&dark_dir).expect("Failed to create dark icons directory");
    fs::create_dir_all(&light_dir).expect("Failed to create light icons directory");

    // Track source changes for incremental builds
    println!("cargo::rerun-if-changed=assets/icons/source/");

    let source_dir = Path::new("assets/icons/source");
    assert!(
        source_dir.exists(),
        "Icon source directory not found: {}",
        source_dir.display()
    );

    // Process each SVG file
    for entry in fs::read_dir(source_dir).expect("Failed to read icon source directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if path.extension().is_some_and(|e| e == "svg") {
            let stem = path
                .file_stem()
                .expect("File has no stem")
                .to_str()
                .expect("Invalid UTF-8 in filename");

            // Render dark icon (inverted colors - black)
            let dark_output = dark_dir.join(format!("{stem}.png"));
            render_svg_to_png(&path, &dark_output, true);

            // Render light icon (original colors - white) for specific icons
            if needs_light_variant(stem) {
                let light_output = light_dir.join(format!("{stem}.png"));
                render_svg_to_png(&path, &light_output, false);
            }
        }
    }
}

/// Renders an SVG file to a PNG file.
///
/// # Arguments
/// * `svg_path` - Path to the source SVG file
/// * `output_path` - Path to write the output PNG
/// * `invert` - If true, invert RGB channels (white → black)
fn render_svg_to_png(svg_path: &Path, output_path: &Path, invert: bool) {
    // Read and parse SVG
    let svg_data = fs::read_to_string(svg_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", svg_path.display(), e));

    // Normalize colors: replace currentColor with white
    // This ensures consistent rendering since resvg interprets currentColor as black
    let svg_data = svg_data.replace("currentColor", "white");

    let options = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_str(&svg_data, &options)
        .unwrap_or_else(|e| panic!("Failed to parse {}: {}", svg_path.display(), e));

    // Create pixmap for rendering
    let mut pixmap = tiny_skia::Pixmap::new(ICON_SIZE, ICON_SIZE).expect("Failed to create pixmap");

    // Calculate transform to fit SVG into icon size
    // Note: ICON_SIZE is small (32), so f32 precision loss is negligible
    #[allow(clippy::cast_precision_loss)]
    let icon_size_f32 = ICON_SIZE as f32;
    let svg_size = tree.size();
    let scale_x = icon_size_f32 / svg_size.width();
    let scale_y = icon_size_f32 / svg_size.height();
    let scale = scale_x.min(scale_y);

    // Center the icon
    let offset_x = (icon_size_f32 - svg_size.width() * scale) / 2.0;
    let offset_y = (icon_size_f32 - svg_size.height() * scale) / 2.0;

    let transform =
        tiny_skia::Transform::from_scale(scale, scale).post_translate(offset_x, offset_y);

    // Render SVG to pixmap
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // Invert colors if needed (for dark icons)
    if invert {
        invert_colors(&mut pixmap);
    }

    // Save as PNG
    save_png(&pixmap, output_path);
}

/// Inverts RGB channels while preserving alpha.
///
/// Transforms white (255, 255, 255) → black (0, 0, 0) while keeping
/// transparent pixels transparent.
///
/// tiny-skia uses premultiplied alpha, where `stored_rgb` = rgb * alpha / 255.
/// For premultiplied pixels: `inverted_rgb` = alpha - `stored_rgb`
fn invert_colors(pixmap: &mut tiny_skia::Pixmap) {
    let data = pixmap.data_mut();
    // Pixels are stored as RGBA (premultiplied), 4 bytes each
    for chunk in data.chunks_exact_mut(4) {
        let alpha = chunk[3];
        // For premultiplied alpha: new_rgb = alpha - old_rgb
        // This correctly inverts the color while preserving transparency
        chunk[0] = alpha.saturating_sub(chunk[0]); // R
        chunk[1] = alpha.saturating_sub(chunk[1]); // G
        chunk[2] = alpha.saturating_sub(chunk[2]); // B
    }
}

/// Saves a pixmap as a PNG file using the image crate.
fn save_png(pixmap: &tiny_skia::Pixmap, path: &Path) {
    let img =
        image_rs::RgbaImage::from_raw(pixmap.width(), pixmap.height(), pixmap.data().to_vec())
            .expect("Failed to create image buffer");

    img.save(path)
        .unwrap_or_else(|e| panic!("Failed to save {}: {}", path.display(), e));
}

/// Determines if an icon needs a light (white) variant.
///
/// Light variants are used for:
/// - Toolbar buttons (white icons on dark button backgrounds)
/// - Dark theme UI elements
/// - Overlays on dark backgrounds (video HUD, notifications)
fn needs_light_variant(name: &str) -> bool {
    matches!(
        name,
        // Navigation (dark theme sidebar)
        "chevron_double_left"
            | "chevron_double_right"
            | "chevron_down"
            | "chevron_left"
            | "chevron_right"
            // Editor (dark theme)
            | "pencil"
            | "triangle_minus"
            | "triangle_plus"
            | "rotate_left"
            | "rotate_right"
            | "flip_horizontal"
            | "flip_vertical"
            // Video toolbar
            | "play"
            | "pause"
            | "loop"
            | "volume"
            | "volume_mute"
            | "triangle_bar_left"
            | "triangle_bar_right"
            | "camera"
            | "ellipsis_horizontal"
            // Navbar
            | "hamburger"
            // Viewer toolbar
            | "zoom_in"
            | "zoom_out"
            | "refresh"
            | "compress"
            | "expand"
            | "fullscreen"
            | "trash"
            | "funnel"
            // HUD indicators
            | "crosshair"
            | "magnifier"
            | "video_camera"
            | "video_camera_audio"
            // Notifications
            | "warning"
            | "checkmark"
    )
}
