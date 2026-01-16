// SPDX-License-Identifier: MPL-2.0
//! Navigation sub-component for gallery navigation with auto-skip support.

use crate::media::MaxSkipAttempts;
use std::path::PathBuf;

/// Direction of navigation for auto-skip retry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Navigate to next media.
    Next,
    /// Navigate to previous media.
    Previous,
}

/// Origin of a media load request for determining auto-skip behavior.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum LoadOrigin {
    /// Media was loaded via navigation (arrows, keyboard).
    /// On failure, auto-skip to next/previous.
    Navigation {
        /// Direction of the navigation.
        direction: Direction,
        /// Number of consecutive skip attempts.
        skip_attempts: u32,
        /// Filenames that have been skipped (for grouped notification).
        skipped_files: Vec<String>,
    },
    /// Media was loaded directly (drag-drop, file dialog, CLI, initial load).
    /// On failure, show error notification and stay on current media.
    #[default]
    DirectOpen,
}

impl LoadOrigin {
    /// Check if this load origin allows navigation (arrows, keyboard nav).
    #[must_use]
    pub fn can_navigate(&self) -> bool {
        matches!(self, Self::Navigation { .. })
    }

    /// Create a navigation origin for the next direction.
    #[must_use]
    pub fn next() -> Self {
        Self::Navigation {
            direction: Direction::Next,
            skip_attempts: 0,
            skipped_files: Vec::new(),
        }
    }

    /// Create a navigation origin for the previous direction.
    #[must_use]
    pub fn previous() -> Self {
        Self::Navigation {
            direction: Direction::Previous,
            skip_attempts: 0,
            skipped_files: Vec::new(),
        }
    }
}

/// Navigation sub-component state.
#[derive(Debug, Clone, Default)]
pub struct State {
    /// Origin of the current/pending load.
    pub load_origin: LoadOrigin,
    /// Path being loaded (pending confirmation).
    pub pending_path: Option<PathBuf>,
}

/// Messages for the navigation sub-component.
#[derive(Debug, Clone)]
pub enum Message {
    /// Start navigation to next media.
    NavigateNext,
    /// Start navigation to previous media.
    NavigatePrevious,
    /// Load started with a specific origin and target path.
    LoadStarted { origin: LoadOrigin, path: PathBuf },
    /// Load succeeded - confirm navigation.
    LoadSucceeded,
    /// Load failed - determine if auto-skip should occur.
    LoadFailed {
        filename: String,
        max_attempts: MaxSkipAttempts,
    },
    /// Reset navigation state (after giving up or on direct open).
    Reset,
}

/// Effects produced by navigation changes.
#[derive(Debug, Clone)]
pub enum Effect {
    /// No effect.
    None,
    /// Request navigation to next media.
    NavigateNext,
    /// Request navigation to previous media.
    NavigatePrevious,
    /// Confirm navigation to the loaded path.
    ConfirmNavigation {
        path: PathBuf,
        skipped_files: Vec<String>,
    },
    /// Retry navigation after a failed load (auto-skip).
    RetryNavigation {
        direction: Direction,
        skip_attempts: u32,
        skipped_files: Vec<String>,
    },
    /// Show notification for skipped files (max attempts reached).
    ShowSkippedNotification { skipped_files: Vec<String> },
    /// Show error notification (direct open failed or max attempts).
    ShowError {
        key: &'static str,
        args: Vec<(&'static str, String)>,
    },
}

impl State {
    /// Handle a navigation message.
    pub fn handle(&mut self, msg: Message) -> Effect {
        match msg {
            Message::NavigateNext => Effect::NavigateNext,
            Message::NavigatePrevious => Effect::NavigatePrevious,
            Message::LoadStarted { origin, path } => {
                self.load_origin = origin;
                self.pending_path = Some(path);
                Effect::None
            }
            Message::LoadSucceeded => {
                let path = self.pending_path.take();
                let skipped_files = match &self.load_origin {
                    LoadOrigin::Navigation { skipped_files, .. } => skipped_files.clone(),
                    LoadOrigin::DirectOpen => Vec::new(),
                };
                self.load_origin = LoadOrigin::DirectOpen;

                if let Some(path) = path {
                    Effect::ConfirmNavigation {
                        path,
                        skipped_files,
                    }
                } else {
                    Effect::None
                }
            }
            Message::LoadFailed {
                filename,
                max_attempts,
            } => {
                self.pending_path = None;

                match &mut self.load_origin {
                    LoadOrigin::Navigation {
                        direction,
                        skip_attempts,
                        skipped_files,
                    } => {
                        skipped_files.push(filename);
                        *skip_attempts += 1;

                        if *skip_attempts >= max_attempts.value() {
                            // Max attempts reached - show notification and give up
                            let files = std::mem::take(skipped_files);
                            self.load_origin = LoadOrigin::DirectOpen;
                            Effect::ShowSkippedNotification {
                                skipped_files: files,
                            }
                        } else {
                            // Retry navigation
                            Effect::RetryNavigation {
                                direction: *direction,
                                skip_attempts: *skip_attempts,
                                skipped_files: skipped_files.clone(),
                            }
                        }
                    }
                    LoadOrigin::DirectOpen => {
                        // Direct open failed - show error
                        Effect::ShowError {
                            key: "error-media-load-failed",
                            args: vec![("filename", filename)],
                        }
                    }
                }
            }
            Message::Reset => {
                self.load_origin = LoadOrigin::DirectOpen;
                self.pending_path = None;
                Effect::None
            }
        }
    }

    /// Check if navigation should auto-skip on failure.
    #[must_use]
    pub fn should_auto_skip(&self) -> bool {
        matches!(self.load_origin, LoadOrigin::Navigation { .. })
    }

    /// Get the current navigation direction, if any.
    #[must_use]
    pub fn direction(&self) -> Option<Direction> {
        match &self.load_origin {
            LoadOrigin::Navigation { direction, .. } => Some(*direction),
            LoadOrigin::DirectOpen => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigate_next_returns_effect() {
        let mut state = State::default();
        let effect = state.handle(Message::NavigateNext);
        assert!(matches!(effect, Effect::NavigateNext));
    }

    #[test]
    fn navigate_previous_returns_effect() {
        let mut state = State::default();
        let effect = state.handle(Message::NavigatePrevious);
        assert!(matches!(effect, Effect::NavigatePrevious));
    }

    #[test]
    fn load_started_stores_origin_and_path() {
        let mut state = State::default();
        let path = PathBuf::from("/test/image.jpg");

        state.handle(Message::LoadStarted {
            origin: LoadOrigin::next(),
            path: path.clone(),
        });

        assert!(state.should_auto_skip());
        assert_eq!(state.pending_path, Some(path));
    }

    #[test]
    fn load_succeeded_confirms_navigation() {
        let mut state = State::default();
        let path = PathBuf::from("/test/image.jpg");

        state.handle(Message::LoadStarted {
            origin: LoadOrigin::next(),
            path: path.clone(),
        });

        let effect = state.handle(Message::LoadSucceeded);
        assert!(matches!(
            effect,
            Effect::ConfirmNavigation { path: p, .. } if p == path
        ));
        assert!(!state.should_auto_skip());
    }

    #[test]
    fn load_failed_retries_navigation() {
        let mut state = State::default();
        let path = PathBuf::from("/test/image.jpg");

        state.handle(Message::LoadStarted {
            origin: LoadOrigin::next(),
            path,
        });

        let effect = state.handle(Message::LoadFailed {
            filename: "image.jpg".to_string(),
            max_attempts: MaxSkipAttempts::default(),
        });

        assert!(matches!(effect, Effect::RetryNavigation { skip_attempts: 1, .. }));
    }

    #[test]
    fn load_failed_shows_error_for_direct_open() {
        let mut state = State::default();

        let effect = state.handle(Message::LoadFailed {
            filename: "image.jpg".to_string(),
            max_attempts: MaxSkipAttempts::default(),
        });

        assert!(matches!(effect, Effect::ShowError { .. }));
    }

    #[test]
    fn direction_returns_current_direction() {
        let mut state = State::default();
        assert!(state.direction().is_none());

        state.load_origin = LoadOrigin::next();
        assert_eq!(state.direction(), Some(Direction::Next));

        state.load_origin = LoadOrigin::previous();
        assert_eq!(state.direction(), Some(Direction::Previous));
    }
}
