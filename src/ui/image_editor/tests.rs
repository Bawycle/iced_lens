// SPDX-License-Identifier: MPL-2.0

use super::*;
use image_rs::{Rgba, RgbaImage};
use tempfile::tempdir;

fn create_test_image(width: u32, height: u32) -> (tempfile::TempDir, PathBuf, ImageData) {
    let temp_dir = tempdir().expect("temp dir");
    let path = temp_dir.path().join("test.png");
    let img = RgbaImage::from_pixel(width, height, Rgba([0, 0, 0, 255]));
    img.save(&path).expect("write png");
    let pixels = vec![0; (width * height * 4) as usize];
    let image = ImageData::from_rgba(width, height, pixels);
    (temp_dir, path, image)
}

#[test]
fn new_editor_state_has_no_changes() {
    let (_dir, path, img) = create_test_image(4, 3);
    let state = State::new(path, &img).expect("editor state");

    assert!(!state.has_unsaved_changes());
    assert!(!state.can_undo());
    assert!(!state.can_redo());
    assert_eq!(state.active_tool(), None);
}

#[test]
fn new_editor_state_initializes_resize_state() {
    let (_dir, path, img) = create_test_image(4, 3);
    let state = State::new(path, &img).expect("editor state");

    assert_eq!(state.resize_state.width, 4);
    assert_eq!(state.resize_state.height, 3);
    assert_eq!(state.resize_state.scale.value(), 100.0);
    assert!(state.resize_state.lock_aspect);
    assert_eq!(state.resize_state.original_aspect, 4.0 / 3.0);
}

#[test]
fn sidebar_starts_expanded() {
    let (_dir, path, img) = create_test_image(4, 3);
    let state = State::new(path, &img).expect("editor state");

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
    let mut state = State::new(path, &img).expect("editor state");

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
    let mut state = State::new(path, &img).expect("editor state");

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
    let mut state = State::new(path, &img).expect("editor state");

    state.update(Message::Sidebar(SidebarMessage::SelectTool(
        EditorTool::Resize,
    )));
    // Type the new value
    state.update(Message::Sidebar(SidebarMessage::WidthInputChanged(
        "50".to_string(),
    )));
    // Submit to trigger calculation and preview update
    state.update(Message::Sidebar(SidebarMessage::WidthInputSubmitted));

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
    let mut state = State::new(path, &img).expect("editor state");

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
    let mut state = State::new(path, &img).expect("editor state");

    state.update(Message::Sidebar(SidebarMessage::SelectTool(
        EditorTool::Resize,
    )));

    // First change dimensions
    state.update(Message::Sidebar(SidebarMessage::WidthInputChanged(
        "50".to_string(),
    )));
    state.update(Message::Sidebar(SidebarMessage::WidthInputSubmitted));
    assert!(
        state.preview_image.is_some(),
        "Preview should exist after resize"
    );

    // Then reset to original dimensions
    state.update(Message::Sidebar(SidebarMessage::WidthInputChanged(
        "100".to_string(),
    )));
    state.update(Message::Sidebar(SidebarMessage::WidthInputSubmitted));

    // Preview should be cleared when dimensions match original
    assert!(
        state.preview_image.is_none(),
        "Preview should be None when dimensions match original"
    );
}

#[test]
fn resize_preview_updates_when_height_changes() {
    let (_dir, path, img) = create_test_image(100, 100);
    let mut state = State::new(path, &img).expect("editor state");

    state.update(Message::Sidebar(SidebarMessage::SelectTool(
        EditorTool::Resize,
    )));

    // Unlock aspect ratio to test independent height change
    state.update(Message::Sidebar(SidebarMessage::ToggleLockAspect));

    state.update(Message::Sidebar(SidebarMessage::HeightInputChanged(
        "75".to_string(),
    )));
    state.update(Message::Sidebar(SidebarMessage::HeightInputSubmitted));

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
    let mut state = State::new(path, &img).expect("editor state");

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

#[test]
fn crop_handle_detection_with_extended_hit_area() {
    let (_dir, path, img) = create_test_image(200, 200);
    let mut state = State::new(path, &img).expect("editor state");

    // Setup crop state with known dimensions
    // Crop at (50, 50) with size 100x100
    state.crop_state.x = 50;
    state.crop_state.y = 50;
    state.crop_state.width = 100;
    state.crop_state.height = 100;
    state.crop_base_width = 200;
    state.crop_base_height = 200;

    // Test TopLeft handle at (50, 50)
    // Exact position should be detected
    state.handle_crop_overlay_mouse_down(50.0, 50.0);
    assert!(
        matches!(
            state.crop_state.overlay.drag_state,
            CropDragState::DraggingHandle {
                handle: HandlePosition::TopLeft,
                ..
            }
        ),
        "Should detect TopLeft handle at exact position"
    );

    // Reset drag state
    state.crop_state.overlay.drag_state = CropDragState::None;

    // Click 15 pixels away from TopLeft handle (within extended hit area)
    // With CROP_HANDLE_HIT_SIZE = 44 (radius 22), this should still be detected
    state.handle_crop_overlay_mouse_down(65.0, 65.0);
    assert!(
        matches!(
            state.crop_state.overlay.drag_state,
            CropDragState::DraggingHandle {
                handle: HandlePosition::TopLeft,
                ..
            }
        ),
        "Should detect TopLeft handle within 15px distance (extended hit area)"
    );

    // Reset drag state
    state.crop_state.overlay.drag_state = CropDragState::None;

    // Test BottomRight handle at (150, 150)
    state.handle_crop_overlay_mouse_down(150.0, 150.0);
    assert!(
        matches!(
            state.crop_state.overlay.drag_state,
            CropDragState::DraggingHandle {
                handle: HandlePosition::BottomRight,
                ..
            }
        ),
        "Should detect BottomRight handle at exact position"
    );

    // Reset drag state
    state.crop_state.overlay.drag_state = CropDragState::None;

    // Click 15 pixels away from BottomRight handle
    state.handle_crop_overlay_mouse_down(135.0, 135.0);
    assert!(
        matches!(
            state.crop_state.overlay.drag_state,
            CropDragState::DraggingHandle {
                handle: HandlePosition::BottomRight,
                ..
            }
        ),
        "Should detect BottomRight handle within 15px distance"
    );

    // Reset drag state
    state.crop_state.overlay.drag_state = CropDragState::None;

    // Test Top center handle at (100, 50)
    state.handle_crop_overlay_mouse_down(100.0, 50.0);
    assert!(
        matches!(
            state.crop_state.overlay.drag_state,
            CropDragState::DraggingHandle {
                handle: HandlePosition::Top,
                ..
            }
        ),
        "Should detect Top handle at exact position"
    );

    // Reset drag state
    state.crop_state.overlay.drag_state = CropDragState::None;

    // Click far from any handle (should trigger rectangle drag, not handle)
    state.handle_crop_overlay_mouse_down(100.0, 100.0);
    assert!(
        matches!(
            state.crop_state.overlay.drag_state,
            CropDragState::DraggingRectangle { .. }
        ),
        "Clicking in center of crop rect should start rectangle drag, not handle drag"
    );

    // Reset drag state
    state.crop_state.overlay.drag_state = CropDragState::None;

    // Click outside crop rectangle entirely
    state.handle_crop_overlay_mouse_down(10.0, 10.0);
    assert!(
        matches!(state.crop_state.overlay.drag_state, CropDragState::None),
        "Clicking outside crop rect should not trigger any drag"
    );
}

#[test]
fn crop_handle_detection_at_image_edges() {
    let (_dir, path, img) = create_test_image(100, 100);
    let mut state = State::new(path, &img).expect("editor state");

    // Crop that extends to image edges
    state.crop_state.x = 0;
    state.crop_state.y = 0;
    state.crop_state.width = 100;
    state.crop_state.height = 100;
    state.crop_base_width = 100;
    state.crop_base_height = 100;

    // Test TopLeft corner at (0, 0) - at image edge
    // Even at the extremity, extended hit area should make it clickable
    state.handle_crop_overlay_mouse_down(0.0, 0.0);
    assert!(
        matches!(
            state.crop_state.overlay.drag_state,
            CropDragState::DraggingHandle {
                handle: HandlePosition::TopLeft,
                ..
            }
        ),
        "Should detect TopLeft handle even at image edge (0, 0)"
    );

    // Reset drag state
    state.crop_state.overlay.drag_state = CropDragState::None;

    // Click slightly offset from edge (within extended hit area)
    state.handle_crop_overlay_mouse_down(15.0, 15.0);
    assert!(
        matches!(
            state.crop_state.overlay.drag_state,
            CropDragState::DraggingHandle {
                handle: HandlePosition::TopLeft,
                ..
            }
        ),
        "Should detect TopLeft handle near edge with extended hit area"
    );

    // Reset drag state
    state.crop_state.overlay.drag_state = CropDragState::None;

    // Test BottomRight corner at (100, 100) - at opposite image edge
    state.handle_crop_overlay_mouse_down(100.0, 100.0);
    assert!(
        matches!(
            state.crop_state.overlay.drag_state,
            CropDragState::DraggingHandle {
                handle: HandlePosition::BottomRight,
                ..
            }
        ),
        "Should detect BottomRight handle at image edge (100, 100)"
    );
}

#[test]
fn flip_horizontal_preserves_dimensions() {
    let (_dir, path, img) = create_test_image(8, 6);
    let mut state = State::new(path, &img).expect("editor state");

    let original_width = state.current_image.width;
    let original_height = state.current_image.height;

    state.update(Message::Sidebar(SidebarMessage::FlipHorizontal));

    assert_eq!(
        state.current_image.width, original_width,
        "Width should be preserved after horizontal flip"
    );
    assert_eq!(
        state.current_image.height, original_height,
        "Height should be preserved after horizontal flip"
    );
}

#[test]
fn flip_vertical_preserves_dimensions() {
    let (_dir, path, img) = create_test_image(8, 6);
    let mut state = State::new(path, &img).expect("editor state");

    let original_width = state.current_image.width;
    let original_height = state.current_image.height;

    state.update(Message::Sidebar(SidebarMessage::FlipVertical));

    assert_eq!(
        state.current_image.width, original_width,
        "Width should be preserved after vertical flip"
    );
    assert_eq!(
        state.current_image.height, original_height,
        "Height should be preserved after vertical flip"
    );
}

#[test]
fn flip_horizontal_records_transformation() {
    let (_dir, path, img) = create_test_image(4, 4);
    let mut state = State::new(path, &img).expect("editor state");

    assert!(!state.has_unsaved_changes(), "Should start with no changes");

    state.update(Message::Sidebar(SidebarMessage::FlipHorizontal));

    assert!(
        state.has_unsaved_changes(),
        "Should have unsaved changes after flip"
    );
    assert!(state.can_undo(), "Should be able to undo flip");
    assert!(!state.can_redo(), "Should not be able to redo before undo");
}

#[test]
fn flip_vertical_records_transformation() {
    let (_dir, path, img) = create_test_image(4, 4);
    let mut state = State::new(path, &img).expect("editor state");

    assert!(!state.has_unsaved_changes(), "Should start with no changes");

    state.update(Message::Sidebar(SidebarMessage::FlipVertical));

    assert!(
        state.has_unsaved_changes(),
        "Should have unsaved changes after flip"
    );
    assert!(state.can_undo(), "Should be able to undo flip");
    assert!(!state.can_redo(), "Should not be able to redo before undo");
}

#[test]
fn flip_horizontal_can_be_undone() {
    let (_dir, path, img) = create_test_image(8, 6);
    let mut state = State::new(path, &img).expect("editor state");

    let original_width = state.current_image.width;
    let original_height = state.current_image.height;

    state.update(Message::Sidebar(SidebarMessage::FlipHorizontal));
    assert!(state.has_unsaved_changes());

    state.update(Message::Sidebar(SidebarMessage::Undo));
    assert!(!state.can_undo(), "Should not be able to undo further");
    assert!(state.can_redo(), "Should be able to redo after undo");
    assert_eq!(
        state.current_image.width, original_width,
        "Dimensions should be restored"
    );
    assert_eq!(
        state.current_image.height, original_height,
        "Dimensions should be restored"
    );
}

#[test]
fn flip_vertical_can_be_undone() {
    let (_dir, path, img) = create_test_image(8, 6);
    let mut state = State::new(path, &img).expect("editor state");

    let original_width = state.current_image.width;
    let original_height = state.current_image.height;

    state.update(Message::Sidebar(SidebarMessage::FlipVertical));
    assert!(state.has_unsaved_changes());

    state.update(Message::Sidebar(SidebarMessage::Undo));
    assert!(!state.can_undo(), "Should not be able to undo further");
    assert!(state.can_redo(), "Should be able to redo after undo");
    assert_eq!(
        state.current_image.width, original_width,
        "Dimensions should be restored"
    );
    assert_eq!(
        state.current_image.height, original_height,
        "Dimensions should be restored"
    );
}

#[test]
fn flip_operations_can_be_combined() {
    let (_dir, path, img) = create_test_image(8, 6);
    let mut state = State::new(path, &img).expect("editor state");

    // Apply both flips
    state.update(Message::Sidebar(SidebarMessage::FlipHorizontal));
    state.update(Message::Sidebar(SidebarMessage::FlipVertical));

    assert_eq!(state.current_image.width, 8);
    assert_eq!(state.current_image.height, 6);
    assert!(state.has_unsaved_changes());

    // Undo both
    state.update(Message::Sidebar(SidebarMessage::Undo));
    assert!(state.can_undo(), "Should be able to undo once more");
    state.update(Message::Sidebar(SidebarMessage::Undo));

    assert!(!state.can_undo(), "Should not be able to undo further");
    assert!(
        state.has_unsaved_changes(),
        "History still contains transformations"
    );
}

#[test]
fn flip_combined_with_rotate() {
    let (_dir, path, img) = create_test_image(8, 6);
    let mut state = State::new(path, &img).expect("editor state");

    // Rotate then flip
    state.update(Message::Sidebar(SidebarMessage::RotateLeft));
    assert_eq!(
        state.current_image.width, 6,
        "Rotate should swap dimensions"
    );
    assert_eq!(state.current_image.height, 8);

    state.update(Message::Sidebar(SidebarMessage::FlipHorizontal));
    assert_eq!(
        state.current_image.width, 6,
        "Flip should preserve dimensions"
    );
    assert_eq!(state.current_image.height, 8);

    // Undo flip, dimensions should stay rotated
    state.update(Message::Sidebar(SidebarMessage::Undo));
    assert_eq!(state.current_image.width, 6);
    assert_eq!(state.current_image.height, 8);

    // Undo rotate, dimensions should be back to original
    state.update(Message::Sidebar(SidebarMessage::Undo));
    assert_eq!(state.current_image.width, 8);
    assert_eq!(state.current_image.height, 6);
}
