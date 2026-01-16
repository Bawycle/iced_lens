// SPDX-License-Identifier: MPL-2.0
//! Media lifecycle cluster - loading, media holder, and errors managed together.
//!
//! This cluster groups the core lifecycle of media handling:
//! - Loading state (spinner, timeout)
//! - Media data holder (current media and path)
//! - Error state (user-friendly errors with details)
//!
//! Note: Navigation logic (auto-skip, direction) remains in component.rs
//! because it's tightly coupled with the external API and MediaNavigator.
//!
//! ## Composition
//!
//! - `loading::State`: Reused from subcomponents (spinner animation, timeout)
//! - `media_holder::State`: Reused from subcomponents (media data container)
//! - `error_state::State`: Reused from subcomponents (error display)

use crate::i18n::fluent::I18n;
use crate::media::MediaData;
use crate::ui::viewer::subcomponents::{error_state, loading, media_holder};
use std::path::PathBuf;

/// Media lifecycle cluster state.
///
/// Combines loading, media holder, and error state into a cohesive unit.
/// Cross-cutting interactions (e.g., load success clearing error and setting media)
/// are handled within the cluster, not by the orchestrator.
#[derive(Debug, Clone, Default)]
pub struct State {
    /// Loading state (spinner, timeout detection).
    pub loading: loading::State,

    /// Media data holder.
    pub media: media_holder::State,

    /// Error state (optional - only present when there's an error).
    pub error: Option<error_state::State>,
}

/// Messages for the media lifecycle cluster.
#[derive(Debug, Clone)]
pub enum Message {
    // ═══════════════════════════════════════════════════════════════════════
    // LOADING MESSAGES
    // ═══════════════════════════════════════════════════════════════════════
    /// Start loading media with metadata.
    StartLoading {
        /// Media type for diagnostics.
        media_type: Option<crate::diagnostics::MediaType>,
        /// File size for diagnostics.
        file_size: Option<u64>,
    },
    /// Stop loading (called externally when load is handled elsewhere).
    StopLoading,
    /// Animate the loading spinner.
    SpinnerTick,

    // ═══════════════════════════════════════════════════════════════════════
    // LOAD RESULT MESSAGES
    // ═══════════════════════════════════════════════════════════════════════
    /// Media loaded successfully.
    MediaLoaded {
        /// The loaded media data.
        data: MediaData,
        /// Path to the loaded media.
        path: PathBuf,
    },
    /// Media load failed - show blocking error.
    MediaLoadFailed {
        /// i18n key for the error message.
        error_key: &'static str,
        /// Technical error details.
        error_details: String,
    },

    // ═══════════════════════════════════════════════════════════════════════
    // ERROR MESSAGES
    // ═══════════════════════════════════════════════════════════════════════
    /// Toggle error details visibility.
    ToggleErrorDetails,
    /// Clear the current error.
    ClearError,

    // ═══════════════════════════════════════════════════════════════════════
    // CROSS-CUTTING MESSAGES
    // ═══════════════════════════════════════════════════════════════════════
    /// Clear all media state (no media loaded).
    ClearMedia,
    /// Refresh translations when locale changes.
    RefreshTranslations,
}

/// Effects produced by media lifecycle operations.
#[derive(Debug, Clone)]
pub enum Effect {
    /// No effect.
    None,
    /// Media loaded successfully - orchestrator should handle follow-up actions.
    MediaLoaded {
        /// Path to the loaded media.
        path: PathBuf,
    },
    /// Media cleared - view should show empty state.
    MediaCleared,
    /// Error occurred - blocking error is displayed.
    ShowError,
    /// Loading has timed out.
    LoadingTimedOut,
}

impl State {
    /// Handle a cluster message.
    ///
    /// This method handles all loading and error messages, including
    /// cross-cutting interactions like load success/failure affecting multiple states.
    #[allow(clippy::needless_pass_by_value)]
    pub fn handle(&mut self, msg: Message, i18n: &I18n) -> Effect {
        match msg {
            // ═══════════════════════════════════════════════════════════════
            // LOADING HANDLERS
            // ═══════════════════════════════════════════════════════════════
            Message::StartLoading {
                media_type,
                file_size,
            } => {
                // Clear any previous error
                self.error = None;

                // Start loading
                self.loading.handle(loading::Message::StartLoading {
                    media_type,
                    file_size,
                });

                Effect::None
            }
            Message::StopLoading => {
                self.loading.handle(loading::Message::StopLoading);
                Effect::None
            }
            Message::SpinnerTick => {
                let effect = self.loading.handle(loading::Message::SpinnerTick);
                match effect {
                    loading::Effect::LoadingTimedOut => Effect::LoadingTimedOut,
                    loading::Effect::None => Effect::None,
                }
            }

            // ═══════════════════════════════════════════════════════════════
            // LOAD RESULT HANDLERS
            // ═══════════════════════════════════════════════════════════════
            Message::MediaLoaded { data, path } => {
                // Stop loading
                self.loading.handle(loading::Message::StopLoading);

                // Clear any error
                self.error = None;

                // Set media data
                self.media.handle(media_holder::Message::SetMedia {
                    data,
                    path: path.clone(),
                });

                Effect::MediaLoaded { path }
            }
            Message::MediaLoadFailed {
                error_key,
                error_details,
            } => {
                // Stop loading
                self.loading.handle(loading::Message::StopLoading);

                // Set error state
                self.error = Some(error_state::State::new(error_key, error_details, i18n));

                Effect::ShowError
            }

            // ═══════════════════════════════════════════════════════════════
            // ERROR HANDLERS
            // ═══════════════════════════════════════════════════════════════
            Message::ToggleErrorDetails => {
                if let Some(error) = &mut self.error {
                    error.handle(error_state::Message::ToggleDetails);
                }
                Effect::None
            }
            Message::ClearError => {
                self.error = None;
                Effect::None
            }

            // ═══════════════════════════════════════════════════════════════
            // CROSS-CUTTING HANDLERS
            // ═══════════════════════════════════════════════════════════════
            Message::ClearMedia => {
                // Stop any loading
                self.loading.handle(loading::Message::StopLoading);

                // Clear media
                self.media.handle(media_holder::Message::Clear);

                // Clear error
                self.error = None;

                Effect::MediaCleared
            }
            Message::RefreshTranslations => {
                if let Some(error) = &mut self.error {
                    error.refresh_translation(i18n);
                }
                Effect::None
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ACCESSORS
    // ═══════════════════════════════════════════════════════════════════════

    /// Check if currently loading.
    #[must_use]
    pub fn is_loading(&self) -> bool {
        self.loading.is_loading()
    }

    /// Get the spinner rotation angle.
    #[must_use]
    pub fn spinner_rotation(&self) -> f32 {
        self.loading.spinner_rotation()
    }

    /// Check if any media is loaded.
    #[must_use]
    pub fn has_media(&self) -> bool {
        self.media.has_media()
    }

    /// Get the current media data.
    #[must_use]
    pub fn media(&self) -> Option<&MediaData> {
        self.media.media()
    }

    /// Get the current media data mutably.
    pub fn media_mut(&mut self) -> Option<&mut MediaData> {
        self.media.media_mut()
    }

    /// Take ownership of the current media data.
    pub fn take_media(&mut self) -> Option<MediaData> {
        self.media.take_media()
    }

    /// Get the current media path.
    #[must_use]
    pub fn current_path(&self) -> Option<&PathBuf> {
        self.media.current_path()
    }

    /// Check if the loaded media is an image.
    #[must_use]
    pub fn is_image(&self) -> bool {
        self.media.is_image()
    }

    /// Check if the loaded media is a video.
    #[must_use]
    pub fn is_video(&self) -> bool {
        self.media.is_video()
    }

    /// Get the current error state.
    #[must_use]
    pub fn error(&self) -> Option<&error_state::State> {
        self.error.as_ref()
    }

    /// Check if there's an error to display.
    #[must_use]
    pub fn has_error(&self) -> bool {
        self.error.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::ImageData;

    fn sample_image() -> MediaData {
        let pixels = vec![0_u8; 100 * 100 * 4];
        MediaData::Image(ImageData::from_rgba(100, 100, pixels))
    }

    fn i18n() -> I18n {
        I18n::default()
    }

    #[test]
    fn start_loading_clears_error_and_sets_state() {
        let mut state = State::default();
        state.error = Some(error_state::State::new("test", "details".into(), &i18n()));

        state.handle(
            Message::StartLoading {
                media_type: None,
                file_size: Some(1024),
            },
            &i18n(),
        );

        assert!(state.is_loading());
        assert!(state.error.is_none());
    }

    #[test]
    fn media_loaded_stops_loading_and_sets_media() {
        let mut state = State::default();
        let path = PathBuf::from("/test/image.jpg");

        state.handle(
            Message::StartLoading {
                media_type: None,
                file_size: None,
            },
            &i18n(),
        );

        let effect = state.handle(
            Message::MediaLoaded {
                data: sample_image(),
                path: path.clone(),
            },
            &i18n(),
        );

        assert!(!state.is_loading());
        assert!(state.has_media());
        assert!(state.is_image());
        assert!(matches!(effect, Effect::MediaLoaded { .. }));
    }

    #[test]
    fn media_load_failed_shows_error() {
        let mut state = State::default();

        state.handle(
            Message::StartLoading {
                media_type: None,
                file_size: None,
            },
            &i18n(),
        );

        let effect = state.handle(
            Message::MediaLoadFailed {
                error_key: "error-load-image",
                error_details: "corrupt file".into(),
            },
            &i18n(),
        );

        assert!(!state.is_loading());
        assert!(state.has_error());
        assert!(matches!(effect, Effect::ShowError));
    }

    #[test]
    fn clear_media_resets_all_state() {
        let mut state = State::default();
        let path = PathBuf::from("/test/image.jpg");

        // Load some media
        state.handle(
            Message::StartLoading {
                media_type: None,
                file_size: None,
            },
            &i18n(),
        );
        state.handle(
            Message::MediaLoaded {
                data: sample_image(),
                path,
            },
            &i18n(),
        );
        assert!(state.has_media());

        // Clear
        let effect = state.handle(Message::ClearMedia, &i18n());

        assert!(!state.has_media());
        assert!(!state.is_loading());
        assert!(!state.has_error());
        assert!(matches!(effect, Effect::MediaCleared));
    }

    #[test]
    fn toggle_error_details_works() {
        let mut state = State::default();
        state.error = Some(error_state::State::new("test", "details".into(), &i18n()));

        assert!(!state.error.as_ref().unwrap().show_details());

        state.handle(Message::ToggleErrorDetails, &i18n());
        assert!(state.error.as_ref().unwrap().show_details());

        state.handle(Message::ToggleErrorDetails, &i18n());
        assert!(!state.error.as_ref().unwrap().show_details());
    }

    #[test]
    fn spinner_tick_advances_rotation() {
        let mut state = State::default();
        state.handle(
            Message::StartLoading {
                media_type: None,
                file_size: None,
            },
            &i18n(),
        );

        let initial = state.spinner_rotation();
        state.handle(Message::SpinnerTick, &i18n());
        assert!(state.spinner_rotation() > initial);
    }

    #[test]
    fn stop_loading_clears_loading_state() {
        let mut state = State::default();
        state.handle(
            Message::StartLoading {
                media_type: None,
                file_size: None,
            },
            &i18n(),
        );
        assert!(state.is_loading());

        state.handle(Message::StopLoading, &i18n());
        assert!(!state.is_loading());
    }

    #[test]
    fn clear_error_removes_error_state() {
        let mut state = State::default();
        state.error = Some(error_state::State::new("test", "details".into(), &i18n()));
        assert!(state.has_error());

        state.handle(Message::ClearError, &i18n());
        assert!(!state.has_error());
    }
}
