// SPDX-License-Identifier: MPL-2.0
//! Zoom sub-component encapsulating ZoomState and its handlers.

use crate::ui::state::ZoomState;

/// Zoom sub-component state.
/// Encapsulates the existing ZoomState and adds handler logic.
#[derive(Debug, Clone)]
pub struct State {
    /// The underlying zoom state (existing type).
    pub inner: ZoomState,
}

/// Messages for the zoom sub-component.
#[derive(Debug, Clone)]
pub enum Message {
    /// Zoom in by one step.
    ZoomIn,
    /// Zoom out by one step.
    ZoomOut,
    /// Reset zoom to default.
    Reset,
    /// Set zoom to a specific percentage.
    SetPercent(f32),
    /// Enable or disable fit-to-window mode.
    SetFitToWindow(bool),
    /// Zoom input text changed.
    InputChanged(String),
    /// Zoom input submitted (Enter pressed).
    InputSubmitted,
}

/// Effects produced by zoom changes.
#[derive(Debug, Clone)]
pub enum Effect {
    /// No effect.
    None,
    /// Zoom level changed.
    ZoomChanged,
    /// Fit-to-window mode changed.
    FitToWindowChanged(bool),
    /// Preferences should be persisted.
    PersistPreferences,
}

impl Default for State {
    fn default() -> Self {
        Self {
            inner: ZoomState::default(),
        }
    }
}

impl State {
    /// Handle a zoom message.
    pub fn handle(&mut self, msg: Message) -> Effect {
        match msg {
            Message::ZoomIn => {
                self.inner.zoom_in();
                Effect::ZoomChanged
            }
            Message::ZoomOut => {
                self.inner.zoom_out();
                Effect::ZoomChanged
            }
            Message::Reset => {
                self.inner.reset_zoom();
                Effect::ZoomChanged
            }
            Message::SetPercent(percent) => {
                self.inner.apply_manual_zoom(percent);
                Effect::ZoomChanged
            }
            Message::SetFitToWindow(enabled) => {
                if enabled {
                    self.inner.enable_fit_to_window();
                } else {
                    self.inner.disable_fit_to_window();
                }
                Effect::FitToWindowChanged(enabled)
            }
            Message::InputChanged(input) => {
                self.inner.on_zoom_input_changed(input);
                Effect::None
            }
            Message::InputSubmitted => {
                if self.inner.on_zoom_input_submitted() {
                    Effect::ZoomChanged
                } else {
                    Effect::None // Invalid input, error shown via zoom_input_error_key
                }
            }
        }
    }

    /// Get the current zoom percentage.
    #[must_use]
    pub fn zoom_percent(&self) -> f32 {
        self.inner.zoom_percent
    }

    /// Check if fit-to-window mode is enabled.
    #[must_use]
    pub fn fit_to_window(&self) -> bool {
        self.inner.fit_to_window
    }

    /// Get the zoom input value.
    #[must_use]
    pub fn zoom_input(&self) -> &str {
        self.inner.zoom_input_value()
    }

    /// Check if zoom input has an error.
    #[must_use]
    pub fn has_input_error(&self) -> bool {
        self.inner.zoom_input_error_key.is_some()
    }

    /// Get the zoom input error key (if any).
    #[must_use]
    pub fn input_error_key(&self) -> Option<&'static str> {
        self.inner.zoom_input_error_key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoom_in_increases_percent() {
        let mut state = State::default();
        let initial = state.inner.zoom_percent;
        state.handle(Message::ZoomIn);
        assert!(state.inner.zoom_percent > initial);
    }

    #[test]
    fn zoom_out_decreases_percent() {
        let mut state = State::default();
        // First zoom in to have room to zoom out
        state.handle(Message::ZoomIn);
        state.handle(Message::ZoomIn);
        let after_zoom_in = state.inner.zoom_percent;

        state.handle(Message::ZoomOut);
        assert!(state.inner.zoom_percent < after_zoom_in);
    }

    #[test]
    fn reset_restores_default_and_disables_fit() {
        let mut state = State::default();
        state.handle(Message::ZoomIn);
        state.handle(Message::Reset);

        // reset_zoom() disables fit_to_window
        assert!(!state.inner.fit_to_window);
    }

    #[test]
    fn set_fit_to_window_returns_correct_effect() {
        let mut state = State::default();

        let effect = state.handle(Message::SetFitToWindow(true));
        assert!(matches!(effect, Effect::FitToWindowChanged(true)));
        assert!(state.fit_to_window());

        let effect = state.handle(Message::SetFitToWindow(false));
        assert!(matches!(effect, Effect::FitToWindowChanged(false)));
        assert!(!state.fit_to_window());
    }

    #[test]
    fn input_changed_updates_state() {
        let mut state = State::default();
        state.handle(Message::InputChanged("150".to_string()));
        assert_eq!(state.zoom_input(), "150");
    }

    #[test]
    fn valid_input_submitted_returns_zoom_changed() {
        let mut state = State::default();
        state.handle(Message::InputChanged("150".to_string()));
        let effect = state.handle(Message::InputSubmitted);
        assert!(matches!(effect, Effect::ZoomChanged));
    }

    #[test]
    fn invalid_input_submitted_returns_none() {
        let mut state = State::default();
        state.handle(Message::InputChanged("invalid".to_string()));
        let effect = state.handle(Message::InputSubmitted);
        assert!(matches!(effect, Effect::None));
        assert!(state.has_input_error());
    }
}
