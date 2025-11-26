// SPDX-License-Identifier: MPL-2.0
//! Generic editor tool helpers (e.g., rotation).

use crate::image_handler::transform;
use crate::ui::editor::{State, Transformation};

impl State {
    pub(crate) fn sidebar_rotate_left(&mut self) {
        self.apply_dynamic_transformation(Transformation::RotateLeft, transform::rotate_left);
    }

    pub(crate) fn sidebar_rotate_right(&mut self) {
        self.apply_dynamic_transformation(Transformation::RotateRight, transform::rotate_right);
    }
}
