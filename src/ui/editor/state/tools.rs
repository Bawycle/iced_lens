// SPDX-License-Identifier: MPL-2.0
//! Generic editor tool helpers (e.g., rotation).

use crate::media::image_transform;
use crate::ui::editor::{State, Transformation};

impl State {
    pub(crate) fn sidebar_rotate_left(&mut self) {
        self.apply_dynamic_transformation(Transformation::RotateLeft, image_transform::rotate_left);
    }

    pub(crate) fn sidebar_rotate_right(&mut self) {
        self.apply_dynamic_transformation(
            Transformation::RotateRight,
            image_transform::rotate_right,
        );
    }

    pub(crate) fn sidebar_flip_horizontal(&mut self) {
        self.apply_dynamic_transformation(
            Transformation::FlipHorizontal,
            image_transform::flip_horizontal,
        );
    }

    pub(crate) fn sidebar_flip_vertical(&mut self) {
        self.apply_dynamic_transformation(
            Transformation::FlipVertical,
            image_transform::flip_vertical,
        );
    }
}
