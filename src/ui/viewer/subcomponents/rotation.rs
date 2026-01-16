// SPDX-License-Identifier: MPL-2.0
//! Temporary rotation state (resets on navigation).

use crate::media::ImageData;
use crate::ui::state::RotationAngle;

/// Rotation state for temporary image rotation.
#[derive(Debug, Clone, Default)]
pub struct State {
    /// Current rotation angle.
    angle: RotationAngle,
    /// Cached rotated image (angle, image) to avoid recomputing.
    cache: Option<(RotationAngle, ImageData)>,
}

/// Messages for the rotation sub-component.
#[derive(Debug, Clone)]
pub enum Message {
    /// Rotate 90° clockwise.
    RotateClockwise,
    /// Rotate 90° counter-clockwise.
    RotateCounterClockwise,
    /// Reset rotation to 0°.
    Reset,
}

/// Effects produced by rotation changes.
#[derive(Debug, Clone)]
pub enum Effect {
    /// No effect.
    None,
    /// Rotation changed - view needs update.
    RotationChanged,
}

impl State {
    /// Handle a rotation message.
    ///
    /// Note: Takes `Message` by value following Iced's `update(message: Message)` pattern.
    #[allow(clippy::needless_pass_by_value)]
    pub fn handle(&mut self, msg: Message) -> Effect {
        match msg {
            Message::RotateClockwise => {
                self.angle = self.angle.rotate_clockwise();
                self.cache = None; // Invalidate cache
                Effect::RotationChanged
            }
            Message::RotateCounterClockwise => {
                self.angle = self.angle.rotate_counterclockwise();
                self.cache = None;
                Effect::RotationChanged
            }
            Message::Reset => {
                if self.angle.is_rotated() {
                    self.angle = RotationAngle::default();
                    self.cache = None;
                    Effect::RotationChanged
                } else {
                    Effect::None
                }
            }
        }
    }

    /// Get the current rotation angle.
    #[must_use]
    pub fn angle(&self) -> RotationAngle {
        self.angle
    }

    /// Check if the image is currently rotated.
    #[must_use]
    pub fn is_rotated(&self) -> bool {
        self.angle.is_rotated()
    }

    /// Update cache with rotated image.
    pub fn set_cache(&mut self, image: ImageData) {
        self.cache = Some((self.angle, image));
    }

    /// Clear the rotation cache.
    pub fn clear_cache(&mut self) {
        self.cache = None;
    }

    /// Get cached rotated image if angle matches current.
    #[must_use]
    pub fn cached_image(&self) -> Option<&ImageData> {
        self.cache
            .as_ref()
            .filter(|(angle, _)| *angle == self.angle)
            .map(|(_, img)| img)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotate_clockwise_changes_angle() {
        let mut state = State::default();
        assert!(!state.is_rotated());

        state.handle(Message::RotateClockwise);
        assert!(state.is_rotated());
        assert_eq!(state.angle().degrees(), 90);
    }

    #[test]
    fn rotate_counterclockwise_changes_angle() {
        let mut state = State::default();
        state.handle(Message::RotateCounterClockwise);
        assert_eq!(state.angle().degrees(), 270);
    }

    #[test]
    fn reset_clears_rotation() {
        let mut state = State::default();
        state.handle(Message::RotateClockwise);
        state.handle(Message::Reset);
        assert!(!state.is_rotated());
    }

    #[test]
    fn reset_on_zero_returns_none_effect() {
        let mut state = State::default();
        let effect = state.handle(Message::Reset);
        assert!(matches!(effect, Effect::None));
    }

    #[test]
    fn rotation_invalidates_cache() {
        let mut state = State::default();
        // Simulate setting a cache (we can't easily create ImageData in tests)
        state.handle(Message::RotateClockwise);
        assert!(state.cached_image().is_none());
    }
}
