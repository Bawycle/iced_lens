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
fn cancel_clears_resize_preview() {
    let (_dir, path, img) = create_test_image(5, 4);
    let mut state = State::new(path, img).expect("editor state");

    state.update(Message::Sidebar(SidebarMessage::SelectTool(
        EditorTool::Resize,
    )));
    // In Option A2, no overlay is used
    assert!(!state.resize_state.overlay.visible);

    // Make a change to create a preview
    state.update(Message::Sidebar(SidebarMessage::ScaleChanged(50.0)));
    assert!(
        state.preview_image.is_some(),
        "Preview should exist after making changes"
    );

    state.discard_changes();

    // After canceling, preview should be cleared
    assert!(
        state.preview_image.is_none(),
        "Preview should be cleared after cancel"
    );
}

#[test]
fn resize_preview_updates_when_width_changes() {
    let (_dir, path, img) = create_test_image(100, 100);
    let mut state = State::new(path, img).expect("editor state");

    state.update(Message::Sidebar(SidebarMessage::SelectTool(
        EditorTool::Resize,
    )));
    state.update(Message::Sidebar(SidebarMessage::WidthInputChanged(
        "50".to_string(),
    )));

    // Preview should exist with new dimensions
    assert!(
        state.preview_image.is_some(),
        "Preview image should be generated when width changes"
    );
    let preview = state.preview_image.as_ref().unwrap();
    assert_eq!(preview.width, 50, "Preview width should match input");
    assert_eq!(
        preview.height, 50,
        "Preview height should maintain aspect ratio"
    );
}

#[test]
fn resize_preview_updates_when_scale_changes() {
    let (_dir, path, img) = create_test_image(100, 80);
    let mut state = State::new(path, img).expect("editor state");

    state.update(Message::Sidebar(SidebarMessage::SelectTool(
        EditorTool::Resize,
    )));
    state.update(Message::Sidebar(SidebarMessage::ScaleChanged(50.0)));

    // Preview should exist and be at 50% size
    assert!(
        state.preview_image.is_some(),
        "Preview should be generated when scale changes"
    );
    let preview = state.preview_image.as_ref().unwrap();
    assert_eq!(preview.width, 50, "Preview should be 50% of original width");
    assert_eq!(
        preview.height, 40,
        "Preview should be 50% of original height"
    );
}

#[test]
fn resize_preview_clears_when_dimensions_match_original() {
    let (_dir, path, img) = create_test_image(100, 100);
    let mut state = State::new(path, img).expect("editor state");

    state.update(Message::Sidebar(SidebarMessage::SelectTool(
        EditorTool::Resize,
    )));

    // First change dimensions
    state.update(Message::Sidebar(SidebarMessage::WidthInputChanged(
        "50".to_string(),
    )));
    assert!(
        state.preview_image.is_some(),
        "Preview should exist after resize"
    );

    // Then reset to original dimensions
    state.update(Message::Sidebar(SidebarMessage::WidthInputChanged(
        "100".to_string(),
    )));

    // Preview should be cleared when dimensions match original
    assert!(
        state.preview_image.is_none(),
        "Preview should be None when dimensions match original"
    );
}

#[test]
fn resize_preview_updates_when_height_changes() {
    let (_dir, path, img) = create_test_image(100, 100);
    let mut state = State::new(path, img).expect("editor state");

    state.update(Message::Sidebar(SidebarMessage::SelectTool(
        EditorTool::Resize,
    )));

    // Unlock aspect ratio to test independent height change
    state.update(Message::Sidebar(SidebarMessage::ToggleLockAspect));

    state.update(Message::Sidebar(SidebarMessage::HeightInputChanged(
        "75".to_string(),
    )));

    assert!(
        state.preview_image.is_some(),
        "Preview should exist after height change"
    );
    let preview = state.preview_image.as_ref().unwrap();
    assert_eq!(preview.height, 75, "Preview height should match input");
}

#[test]
fn resize_preview_works_with_tool_selected() {
    let (_dir, path, img) = create_test_image(100, 100);
    let mut state = State::new(path, img).expect("editor state");

    state.update(Message::Sidebar(SidebarMessage::SelectTool(
        EditorTool::Resize,
    )));

    // In Option A2, no overlay is shown - preview is direct
    assert!(
        !state.resize_state.overlay.visible,
        "Overlay should not be visible in Option A2"
    );

    // Change dimensions - preview should be generated directly on canvas
    state.update(Message::Sidebar(SidebarMessage::ScaleChanged(75.0)));

    assert!(
        state.preview_image.is_some(),
        "Preview should be generated when resize tool is active (Option A2)"
    );
    let preview = state.preview_image.as_ref().unwrap();
    assert_eq!(preview.width, 75, "Preview should reflect the new scale");
}
