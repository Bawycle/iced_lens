// SPDX-License-Identifier: MPL-2.0
//! Transformation history bookkeeping (undo/redo).

use crate::image_handler::transform;
use crate::ui::editor::{State, Transformation};

impl State {
    /// Returns true when the user has applied at least one transformation since load/save.
    pub fn has_unsaved_changes(&self) -> bool {
        !self.transformation_history.is_empty()
    }

    /// Whether an undo operation is currently possible.
    pub fn can_undo(&self) -> bool {
        self.history_index > 0
    }

    /// Whether a redo operation is currently possible.
    pub fn can_redo(&self) -> bool {
        self.history_index < self.transformation_history.len()
    }

    pub(crate) fn sidebar_undo(&mut self) {
        if self.can_undo() {
            self.history_index -= 1;
            self.replay_transformations_up_to_index();
        }
    }

    pub(crate) fn sidebar_redo(&mut self) {
        if self.can_redo() {
            self.history_index += 1;
            self.replay_transformations_up_to_index();
        }
    }

    pub(crate) fn record_transformation(&mut self, transformation: Transformation) {
        if self.history_index < self.transformation_history.len() {
            self.transformation_history.truncate(self.history_index);
        }
        self.transformation_history.push(transformation);
        self.history_index = self.transformation_history.len();
    }

    pub(crate) fn replay_transformations_up_to_index(&mut self) {
        // Reload the original image from disk
        let Ok(mut working_image) = image_rs::open(&self.image_path) else {
            eprintln!("Failed to reload original image for replay");
            return;
        };

        // Apply transformations up to history_index
        for i in 0..self.history_index {
            if i >= self.transformation_history.len() {
                break;
            }

            working_image = match &self.transformation_history[i] {
                Transformation::RotateLeft => transform::rotate_left(&working_image),
                Transformation::RotateRight => transform::rotate_right(&working_image),
                Transformation::FlipHorizontal => transform::flip_horizontal(&working_image),
                Transformation::FlipVertical => transform::flip_vertical(&working_image),
                Transformation::Crop { rect } => {
                    let x = rect.x.max(0.0) as u32;
                    let y = rect.y.max(0.0) as u32;
                    let width = rect.width.max(1.0) as u32;
                    let height = rect.height.max(1.0) as u32;
                    match transform::crop(&working_image, x, y, width, height) {
                        Some(cropped) => cropped,
                        None => {
                            eprintln!("Failed to apply crop during replay: invalid crop area");
                            working_image
                        }
                    }
                }
                Transformation::Resize { width, height } => {
                    transform::resize(&working_image, *width, *height)
                }
            };
        }

        // Update current state with replayed image
        self.working_image = working_image;
        match transform::dynamic_to_image_data(&self.working_image) {
            Ok(image_data) => {
                self.current_image = image_data;
                self.sync_resize_state_dimensions();
                self.preview_image = None;
            }
            Err(err) => {
                eprintln!("Failed to convert replayed image: {err:?}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image_handler::ImageData;
    use iced::widget::image;
    use image_rs::{Rgba, RgbaImage};
    use tempfile::TempDir;

    fn create_test_image(width: u32, height: u32) -> (TempDir, std::path::PathBuf, ImageData) {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let path = temp_dir.path().join("history.png");
        let rgba = RgbaImage::from_pixel(width, height, Rgba([0, 0, 0, 255]));
        rgba.save(&path).expect("write png");
        let pixels = vec![0; (width * height * 4) as usize];
        let image = ImageData {
            handle: image::Handle::from_rgba(width, height, pixels),
            width,
            height,
        };
        (temp_dir, path, image)
    }

    fn editor_state(width: u32, height: u32) -> (TempDir, State) {
        let (dir, path, image) = create_test_image(width, height);
        let state = State::new(path, image).expect("editor state");
        (dir, state)
    }

    #[test]
    fn recording_after_undo_discards_redo_stack() {
        let (_dir, mut state) = editor_state(8, 6);

        state.record_transformation(Transformation::RotateLeft);
        state.record_transformation(Transformation::RotateRight);
        assert_eq!(state.transformation_history.len(), 2);

        state.sidebar_undo();
        assert_eq!(state.history_index, 1);

        state.record_transformation(Transformation::Resize {
            width: 4,
            height: 3,
        });

        assert_eq!(state.transformation_history.len(), 2);
        assert!(matches!(
            state.transformation_history[1],
            Transformation::Resize {
                width: 4,
                height: 3
            }
        ));
        assert_eq!(state.history_index, 2);
    }

    #[test]
    fn undo_redo_respect_history_bounds() {
        let (_dir, mut state) = editor_state(6, 4);

        state.record_transformation(Transformation::RotateLeft);
        assert!(state.can_undo());
        assert!(!state.can_redo());

        state.sidebar_undo();
        assert_eq!(state.history_index, 0);
        assert!(!state.can_undo());

        state.sidebar_undo();
        assert_eq!(state.history_index, 0);

        state.sidebar_redo();
        assert_eq!(state.history_index, 1);
        assert!(!state.can_redo());

        state.sidebar_redo();
        assert_eq!(state.history_index, 1);
    }

    #[test]
    fn replay_reapplies_transformations() {
        let (_dir, mut state) = editor_state(5, 3);

        state.record_transformation(Transformation::RotateLeft);
        state.replay_transformations_up_to_index();

        assert_eq!(state.current_image.width, 3);
        assert_eq!(state.current_image.height, 5);

        state.sidebar_undo();
        assert_eq!(state.current_image.width, 5);
        assert_eq!(state.current_image.height, 3);

        state.sidebar_redo();
        assert_eq!(state.current_image.width, 3);
        assert_eq!(state.current_image.height, 5);
    }
}
