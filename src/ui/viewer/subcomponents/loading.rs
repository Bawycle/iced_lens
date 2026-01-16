// SPDX-License-Identifier: MPL-2.0
//! Loading state sub-component with animated spinner.

use std::time::{Duration, Instant};

/// Timeout before considering a load operation as potentially stuck.
const LOADING_TIMEOUT: Duration = Duration::from_secs(10);

/// Spinner rotation speed in radians per tick.
const SPINNER_SPEED: f32 = 0.1;

/// Loading state for the media viewer.
#[derive(Debug, Clone, Default)]
pub struct State {
    /// Whether media is currently being loaded.
    pub is_loading: bool,
    /// When loading started (for timeout detection).
    started_at: Option<Instant>,
    /// Current spinner rotation angle in radians.
    spinner_rotation: f32,
    /// Media type being loaded (for diagnostics).
    media_type: Option<crate::diagnostics::MediaType>,
    /// File size in bytes (for diagnostics).
    file_size: Option<u64>,
}

/// Messages for the loading state sub-component.
#[derive(Debug, Clone)]
pub enum Message {
    /// Start loading with optional metadata for diagnostics.
    StartLoading {
        media_type: Option<crate::diagnostics::MediaType>,
        file_size: Option<u64>,
    },
    /// Stop loading (success or failure).
    StopLoading,
    /// Animate the spinner.
    SpinnerTick,
}

/// Effects produced by the loading state.
#[derive(Debug, Clone)]
pub enum Effect {
    /// No effect.
    None,
    /// Loading has timed out.
    LoadingTimedOut,
}

impl State {
    /// Handle a loading state message.
    pub fn handle(&mut self, msg: Message) -> Effect {
        match msg {
            Message::StartLoading {
                media_type,
                file_size,
            } => {
                self.is_loading = true;
                self.started_at = Some(Instant::now());
                self.media_type = media_type;
                self.file_size = file_size;
                Effect::None
            }
            Message::StopLoading => {
                self.is_loading = false;
                self.started_at = None;
                self.spinner_rotation = 0.0;
                self.media_type = None;
                self.file_size = None;
                Effect::None
            }
            Message::SpinnerTick => {
                if self.is_loading {
                    self.spinner_rotation += SPINNER_SPEED;
                    if self.spinner_rotation > std::f32::consts::TAU {
                        self.spinner_rotation -= std::f32::consts::TAU;
                    }
                    // Check timeout
                    if let Some(started) = self.started_at {
                        if started.elapsed() > LOADING_TIMEOUT {
                            return Effect::LoadingTimedOut;
                        }
                    }
                }
                Effect::None
            }
        }
    }

    /// Get the current spinner rotation angle in radians.
    #[must_use]
    pub fn spinner_rotation(&self) -> f32 {
        self.spinner_rotation
    }

    /// Check if currently loading.
    #[must_use]
    pub fn is_loading(&self) -> bool {
        self.is_loading
    }

    /// Get the media type being loaded (if any).
    #[must_use]
    pub fn media_type(&self) -> Option<crate::diagnostics::MediaType> {
        self.media_type
    }

    /// Get the file size being loaded (if any).
    #[must_use]
    pub fn file_size(&self) -> Option<u64> {
        self.file_size
    }

    /// Get when loading started (if currently loading).
    #[must_use]
    pub fn started_at(&self) -> Option<Instant> {
        self.started_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_loading_sets_state() {
        let mut state = State::default();
        assert!(!state.is_loading());

        state.handle(Message::StartLoading {
            media_type: None,
            file_size: Some(1024),
        });

        assert!(state.is_loading());
        assert!(state.started_at().is_some());
        assert_eq!(state.file_size(), Some(1024));
    }

    #[test]
    fn stop_loading_clears_state() {
        let mut state = State::default();
        state.handle(Message::StartLoading {
            media_type: None,
            file_size: None,
        });
        state.handle(Message::StopLoading);

        assert!(!state.is_loading());
        assert!(state.started_at().is_none());
        assert_eq!(state.spinner_rotation(), 0.0);
    }

    #[test]
    fn spinner_tick_advances_rotation() {
        let mut state = State::default();
        state.handle(Message::StartLoading {
            media_type: None,
            file_size: None,
        });

        let initial = state.spinner_rotation();
        state.handle(Message::SpinnerTick);
        assert!(state.spinner_rotation() > initial);
    }
}
