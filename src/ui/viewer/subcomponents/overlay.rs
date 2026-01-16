// SPDX-License-Identifier: MPL-2.0
//! Overlay visibility sub-component for fullscreen controls.

use iced::Point;
use std::time::{Duration, Instant};

/// Timeout before hiding overlay controls.
const OVERLAY_TIMEOUT: Duration = Duration::from_secs(3);

/// Minimum mouse movement to be considered significant.
const MOUSE_MOVEMENT_THRESHOLD: f32 = 10.0;

/// Delay after entering fullscreen before responding to mouse movements.
const FULLSCREEN_ENTRY_IGNORE_DELAY: Duration = Duration::from_millis(500);

/// Overlay visibility state for fullscreen controls.
#[derive(Debug, Clone, Default)]
pub struct State {
    /// Whether navigation arrows are visible.
    pub arrows_visible: bool,
    /// Last significant mouse movement timestamp.
    last_mouse_move: Option<Instant>,
    /// Last user interaction with overlay controls.
    last_overlay_interaction: Option<Instant>,
    /// Last mouse position (to filter micro-movements).
    last_mouse_position: Option<Point>,
    /// When fullscreen was entered (to ignore initial movements).
    fullscreen_entered_at: Option<Instant>,
}

/// Messages for the overlay sub-component.
#[derive(Debug, Clone)]
pub enum Message {
    /// Mouse moved to a new position.
    MouseMoved(Point),
    /// User interacted with overlay controls.
    OverlayInteraction,
    /// Entered fullscreen mode.
    EnteredFullscreen,
    /// Exited fullscreen mode.
    ExitedFullscreen,
    /// Check if overlay should be hidden due to timeout.
    CheckTimeout,
}

/// Effects produced by overlay visibility changes.
#[derive(Debug, Clone)]
pub enum Effect {
    /// No effect.
    None,
    /// Visibility changed.
    VisibilityChanged(bool),
}

impl State {
    /// Handle an overlay message.
    ///
    /// Note: Takes `Message` by value following Iced's `update(message: Message)` pattern.
    #[allow(clippy::needless_pass_by_value)]
    pub fn handle(&mut self, msg: Message) -> Effect {
        match msg {
            Message::MouseMoved(pos) => {
                // Ignore movements right after entering fullscreen
                if let Some(entered) = self.fullscreen_entered_at {
                    if entered.elapsed() < FULLSCREEN_ENTRY_IGNORE_DELAY {
                        return Effect::None;
                    }
                }

                // Filter micro-movements (sensor noise)
                let is_significant = self.last_mouse_position.is_none_or(|last| {
                    let dx = pos.x - last.x;
                    let dy = pos.y - last.y;
                    (dx * dx + dy * dy).sqrt() > MOUSE_MOVEMENT_THRESHOLD
                });

                self.last_mouse_position = Some(pos);

                if is_significant {
                    self.last_mouse_move = Some(Instant::now());
                    if !self.arrows_visible {
                        self.arrows_visible = true;
                        return Effect::VisibilityChanged(true);
                    }
                }
                Effect::None
            }
            Message::OverlayInteraction => {
                self.last_overlay_interaction = Some(Instant::now());
                Effect::None
            }
            Message::EnteredFullscreen => {
                self.fullscreen_entered_at = Some(Instant::now());
                self.arrows_visible = true;
                self.last_mouse_move = Some(Instant::now());
                Effect::VisibilityChanged(true)
            }
            Message::ExitedFullscreen => {
                self.fullscreen_entered_at = None;
                self.arrows_visible = true; // Always visible when not fullscreen
                Effect::VisibilityChanged(true)
            }
            Message::CheckTimeout => {
                // Only hide in fullscreen mode
                if self.fullscreen_entered_at.is_none() {
                    return Effect::None;
                }

                let last_activity = [self.last_mouse_move, self.last_overlay_interaction]
                    .into_iter()
                    .flatten()
                    .max();

                if let Some(t) = last_activity {
                    if t.elapsed() > OVERLAY_TIMEOUT && self.arrows_visible {
                        self.arrows_visible = false;
                        return Effect::VisibilityChanged(false);
                    }
                }
                Effect::None
            }
        }
    }

    /// Check if in fullscreen mode.
    #[must_use]
    pub fn is_fullscreen(&self) -> bool {
        self.fullscreen_entered_at.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entering_fullscreen_shows_overlay() {
        let mut state = State::default();
        let effect = state.handle(Message::EnteredFullscreen);

        assert!(state.arrows_visible);
        assert!(state.is_fullscreen());
        assert!(matches!(effect, Effect::VisibilityChanged(true)));
    }

    #[test]
    fn exiting_fullscreen_keeps_overlay_visible() {
        let mut state = State::default();
        state.handle(Message::EnteredFullscreen);
        state.arrows_visible = false;

        let effect = state.handle(Message::ExitedFullscreen);
        assert!(state.arrows_visible);
        assert!(!state.is_fullscreen());
        assert!(matches!(effect, Effect::VisibilityChanged(true)));
    }

    #[test]
    fn overlay_hides_after_timeout_in_fullscreen() {
        let mut state = State::default();
        state.handle(Message::EnteredFullscreen);
        assert!(state.arrows_visible);

        // Simulate time passing
        state.last_mouse_move = Instant::now().checked_sub(Duration::from_secs(5));
        state.last_overlay_interaction = None;

        let effect = state.handle(Message::CheckTimeout);
        assert!(!state.arrows_visible);
        assert!(matches!(effect, Effect::VisibilityChanged(false)));
    }

    #[test]
    fn timeout_does_nothing_outside_fullscreen() {
        let mut state = State {
            arrows_visible: true,
            last_mouse_move: Instant::now().checked_sub(Duration::from_secs(10)),
            ..Default::default()
        };

        let effect = state.handle(Message::CheckTimeout);
        assert!(state.arrows_visible);
        assert!(matches!(effect, Effect::None));
    }
}
