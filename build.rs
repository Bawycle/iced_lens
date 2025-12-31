// SPDX-License-Identifier: MPL-2.0
//! Build script for icon generation and platform-specific resources.
//!
//! This script generates all application icons from SVG sources at compile time,
//! ensuring a single source of truth for visual assets.
//!
//! ## Generated Assets
//!
//! ### Branding Icons (from `assets/branding/iced_lens.svg`)
//! - PNG files at multiple sizes (16, 32, 48, 64, 128, 256, 512 px)
//! - ICO file for Windows (multi-resolution)
//! - Output: `target/branding/`
//!
//! ### UI Icons (from `assets/icons/source/*.svg`)
//! - Dark icons (black on transparent) for light backgrounds
//! - Light icons (white on transparent) for dark backgrounds
//! - Output: `$OUT_DIR/icons/`
//!
//! ## Windows
//! The ICO file is embedded into the executable via winresource.

use std::fs;
use std::io::BufWriter;
use std::path::Path;

/// Branding icon sizes to generate (in pixels).
const BRANDING_SIZES: &[u32] = &[16, 32, 48, 64, 128, 256, 512];

/// Icon sizes to include in the Windows ICO file.
/// Windows recommends 16, 32, 48, and 256 for best display at all DPI settings.
const ICO_SIZES: &[u32] = &[16, 32, 48, 256];

/// Icon sizes to include in the macOS ICNS file.
/// macOS expects specific sizes for Retina and standard displays.
const ICNS_SIZES: &[u32] = &[16, 32, 64, 128, 256, 512];

/// UI icon size in pixels (width and height).
const UI_ICON_SIZE: u32 = 32;

fn main() {
    // Generate branding icons from master SVG
    generate_branding_icons();

    // Windows-specific: embed application icon
    #[cfg(target_os = "windows")]
    {
        let ico_path = get_branding_output_dir().join("iced_lens.ico");
        let mut res = winresource::WindowsResource::new();
        res.set_icon(ico_path.to_str().expect("ICO path is not valid UTF-8"));
        res.compile().expect("Failed to compile Windows resources");
    }

    // Generate UI icons from SVG sources
    generate_ui_icons();
}

/// Returns the output directory for branding assets.
///
/// Uses `target/branding/` for easy access by build scripts and installers.
fn get_branding_output_dir() -> std::path::PathBuf {
    // Find the target directory by going up from OUT_DIR
    // OUT_DIR is typically: target/<profile>/build/<pkg>/out
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    let out_path = Path::new(&out_dir);

    // Navigate up to target directory
    let target_dir = out_path
        .ancestors()
        .find(|p| p.file_name().is_some_and(|n| n == "target"))
        .expect("Could not find target directory");

    target_dir.join("branding")
}

/// Generates branding icons from the master SVG.
///
/// Creates PNG files at multiple sizes and an ICO file for Windows.
fn generate_branding_icons() {
    let source_svg = Path::new("assets/branding/iced_lens.svg");
    let output_dir = get_branding_output_dir();

    // Track source changes for incremental builds
    println!("cargo::rerun-if-changed=assets/branding/iced_lens.svg");

    // Create output directory
    fs::create_dir_all(&output_dir).expect("Failed to create branding output directory");

    // Read and parse SVG
    let svg_data = fs::read_to_string(source_svg)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", source_svg.display(), e));

    let options = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_str(&svg_data, &options)
        .unwrap_or_else(|e| panic!("Failed to parse {}: {}", source_svg.display(), e));

    // Generate PNG files at each size
    let mut png_images: Vec<(u32, Vec<u8>)> = Vec::new();

    for &size in BRANDING_SIZES {
        let png_data = render_svg_to_png_data(&tree, size);
        let output_path = output_dir.join(format!("iced_lens-{size}.png"));

        // Save PNG file
        fs::write(&output_path, &png_data)
            .unwrap_or_else(|e| panic!("Failed to write {}: {}", output_path.display(), e));

        // Keep PNG data for ICO generation
        if ICO_SIZES.contains(&size) {
            png_images.push((size, png_data));
        }
    }

    // Generate ICO file for Windows
    generate_ico_file(&output_dir.join("iced_lens.ico"), &png_images);

    // Generate ICNS file for macOS
    // Collect sizes needed for ICNS (some may already be in png_images, others need generation)
    let icns_images: Vec<(u32, Vec<u8>)> = ICNS_SIZES
        .iter()
        .map(|&size| {
            if let Some((_, data)) = png_images.iter().find(|(s, _)| *s == size) {
                (size, data.clone())
            } else {
                // Generate missing sizes for ICNS
                (size, render_svg_to_png_data(&tree, size))
            }
        })
        .collect();
    generate_icns_file(&output_dir.join("iced_lens.icns"), &icns_images);

    eprintln!(
        "cargo:warning=Generated branding icons in {}",
        output_dir.display()
    );
}

/// Renders an SVG tree to PNG data at the specified size.
fn render_svg_to_png_data(tree: &resvg::usvg::Tree, size: u32) -> Vec<u8> {
    // Create pixmap for rendering
    let mut pixmap =
        tiny_skia::Pixmap::new(size, size).expect("Failed to create pixmap for branding icon");

    // Calculate transform to fit SVG into target size
    #[allow(clippy::cast_precision_loss)]
    let size_f32 = size as f32;
    let svg_size = tree.size();
    let scale_x = size_f32 / svg_size.width();
    let scale_y = size_f32 / svg_size.height();
    let scale = scale_x.min(scale_y);

    // Center the icon
    let offset_x = (size_f32 - svg_size.width() * scale) / 2.0;
    let offset_y = (size_f32 - svg_size.height() * scale) / 2.0;

    let transform =
        tiny_skia::Transform::from_scale(scale, scale).post_translate(offset_x, offset_y);

    // Render SVG to pixmap
    resvg::render(tree, transform, &mut pixmap.as_mut());

    // Encode as PNG
    pixmap.encode_png().expect("Failed to encode PNG")
}

/// Generates an ICO file containing multiple icon sizes.
fn generate_ico_file(output_path: &Path, png_images: &[(u32, Vec<u8>)]) {
    let file = fs::File::create(output_path)
        .unwrap_or_else(|e| panic!("Failed to create {}: {}", output_path.display(), e));
    let mut writer = BufWriter::new(file);

    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);

    for (size, png_data) in png_images {
        // Decode PNG to get raw RGBA data
        let img = image_rs::load_from_memory_with_format(png_data, image_rs::ImageFormat::Png)
            .expect("Failed to decode PNG for ICO")
            .into_rgba8();

        // Create ICO image entry
        // For sizes <= 256, we embed as PNG (better compression)
        let ico_image = if *size <= 256 {
            ico::IconImage::from_rgba_data(*size, *size, img.into_raw())
        } else {
            // For larger sizes, also use PNG encoding
            ico::IconImage::from_rgba_data(*size, *size, img.into_raw())
        };

        icon_dir.add_entry(ico::IconDirEntry::encode(&ico_image).expect("Failed to encode ICO entry"));
    }

    icon_dir
        .write(&mut writer)
        .unwrap_or_else(|e| panic!("Failed to write ICO file: {e}"));
}

/// Generates an ICNS file for macOS containing multiple icon sizes.
fn generate_icns_file(output_path: &Path, png_images: &[(u32, Vec<u8>)]) {
    let mut icon_family = icns::IconFamily::new();

    for (size, png_data) in png_images {
        // Decode PNG to get raw RGBA data
        let img = image_rs::load_from_memory_with_format(png_data, image_rs::ImageFormat::Png)
            .expect("Failed to decode PNG for ICNS")
            .into_rgba8();

        // Map size to ICNS icon type
        // ICNS uses specific type codes for each size
        let icon_type = match *size {
            16 => icns::IconType::RGBA32_16x16,
            32 => icns::IconType::RGBA32_32x32,
            64 => icns::IconType::RGBA32_64x64,
            128 => icns::IconType::RGBA32_128x128,
            256 => icns::IconType::RGBA32_256x256,
            512 => icns::IconType::RGBA32_512x512,
            _ => continue, // Skip unsupported sizes
        };

        // Create ICNS image from RGBA data (row-major, RGBA order)
        let icns_image =
            icns::Image::from_data(icns::PixelFormat::RGBA, *size, *size, img.into_raw())
                .unwrap_or_else(|e| panic!("Failed to create ICNS image for size {size}: {e}"));

        icon_family
            .add_icon_with_type(&icns_image, icon_type)
            .unwrap_or_else(|e| panic!("Failed to add icon to ICNS: {e}"));
    }

    let file = fs::File::create(output_path)
        .unwrap_or_else(|e| panic!("Failed to create {}: {e}", output_path.display()));

    icon_family
        .write(file)
        .unwrap_or_else(|e| panic!("Failed to write ICNS file: {e}"));
}

// ============================================================================
// UI Icons Generation (existing functionality)
// ============================================================================

/// Generates UI icons from SVG sources at compile time.
///
/// Icons are rendered to the `OUT_DIR` for inclusion via `include_bytes!`.
/// Dark icons (black on transparent) are created by inverting the white SVGs.
/// Light icons (white on transparent) are direct renders of the SVGs.
fn generate_ui_icons() {
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
            render_ui_icon(&path, &dark_output, true);

            // Render light icon (original colors - white) for specific icons
            if needs_light_variant(stem) {
                let light_output = light_dir.join(format!("{stem}.png"));
                render_ui_icon(&path, &light_output, false);
            }
        }
    }
}

/// Renders an SVG file to a PNG file for UI icons.
fn render_ui_icon(svg_path: &Path, output_path: &Path, invert: bool) {
    // Read and parse SVG
    let svg_data = fs::read_to_string(svg_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", svg_path.display(), e));

    // Normalize colors: replace currentColor with white
    let svg_data = svg_data.replace("currentColor", "white");

    let options = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_str(&svg_data, &options)
        .unwrap_or_else(|e| panic!("Failed to parse {}: {}", svg_path.display(), e));

    // Create pixmap for rendering
    let mut pixmap =
        tiny_skia::Pixmap::new(UI_ICON_SIZE, UI_ICON_SIZE).expect("Failed to create pixmap");

    // Calculate transform to fit SVG into icon size
    #[allow(clippy::cast_precision_loss)]
    let icon_size_f32 = UI_ICON_SIZE as f32;
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
fn invert_colors(pixmap: &mut tiny_skia::Pixmap) {
    let data = pixmap.data_mut();
    for chunk in data.chunks_exact_mut(4) {
        let alpha = chunk[3];
        chunk[0] = alpha.saturating_sub(chunk[0]); // R
        chunk[1] = alpha.saturating_sub(chunk[1]); // G
        chunk[2] = alpha.saturating_sub(chunk[2]); // B
    }
}

/// Saves a pixmap as a PNG file.
fn save_png(pixmap: &tiny_skia::Pixmap, path: &Path) {
    let img =
        image_rs::RgbaImage::from_raw(pixmap.width(), pixmap.height(), pixmap.data().to_vec())
            .expect("Failed to create image buffer");

    img.save(path)
        .unwrap_or_else(|e| panic!("Failed to save {}: {}", path.display(), e));
}

/// Determines if an icon needs a light (white) variant.
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
