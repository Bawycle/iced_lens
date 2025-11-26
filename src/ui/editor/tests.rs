// SPDX-License-Identifier: MPL-2.0

use super::*;
use iced::widget::image;
use image_rs::{Rgba, RgbaImage};
use tempfile::tempdir;

fn create_test_image(width: u32, height: u32) -> (tempfile::TempDir, PathBuf, ImageData) {
    let temp_dir = tempdir().expect("temp dir");
    let path = temp_dir.path().join("test.png");
    let img = RgbaImage::from_pixel(width, height, Rgba([0, 0, 0, 255]));
    img.save(&path).expect("write png");
    let pixels = vec![0; (width * height * 4) as usize];
    let image = ImageData {
        handle: image::Handle::from_rgba(width, height, pixels),
        width,
        height,
    };
    (temp_dir, path, image)
}

#[test]
fn new_editor_state_has_no_changes() {
    let (_dir, path, img) = create_test_image(4, 3);
    let state = State::new(path, img).expect("editor state");

    assert!(!state.has_unsaved_changes());
    assert!(!state.can_undo());
    assert!(!state.can_redo());
    assert_eq!(state.active_tool(), None);
}

#[test]
fn new_editor_state_initializes_resize_state() {
    let (_dir, path, img) = create_test_image(4, 3);
    let state = State::new(path, img).expect("editor state");

    assert_eq!(state.resize_state.width, 4);
    assert_eq!(state.resize_state.height, 3);
    assert_eq!(state.resize_state.scale_percent, 100.0);
    assert!(state.resize_state.lock_aspect);
    assert_eq!(state.resize_state.original_aspect, 4.0 / 3.0);
}

#[test]
fn sidebar_starts_expanded() {
    let (_dir, path, img) = create_test_image(4, 3);
    let state = State::new(path, img).expect("editor state");

    assert!(state.is_sidebar_expanded());
}

#[test]
fn crop_ratio_variants_are_distinct() {
    assert_ne!(CropRatio::Free, CropRatio::Square);
    assert_ne!(CropRatio::Landscape, CropRatio::Portrait);
    assert_ne!(CropRatio::Photo, CropRatio::PhotoPortrait);
}

#[test]
fn apply_resize_updates_image_dimensions() {
    let (_dir, path, img) = create_test_image(8, 6);
    let mut state = State::new(path, img).expect("editor state");

    state.update(Message::Sidebar(SidebarMessage::SelectTool(
        EditorTool::Resize,
    )));
    state.resize_state.width = 4;
    state.resize_state.height = 3;
    state.resize_state.width_input = "4".into();
    state.resize_state.height_input = "3".into();
    state.update(Message::Sidebar(SidebarMessage::ApplyResize));

    assert_eq!(state.current_image.width, 4);
    assert_eq!(state.current_image.height, 3);
}

#[test]
fn cancel_hides_resize_overlay() {
    let (_dir, path, img) = create_test_image(5, 4);
    let mut state = State::new(path, img).expect("editor state");

    state.update(Message::Sidebar(SidebarMessage::SelectTool(
        EditorTool::Resize,
    )));
    assert!(state.resize_state.overlay.visible);

    state.discard_changes();

    assert!(!state.resize_state.overlay.visible);
}
