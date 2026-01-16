// SPDX-License-Identifier: MPL-2.0
//! Media holder sub-component for managing current media state.

use crate::media::MediaData;
use std::path::PathBuf;

/// Media holder sub-component state.
#[derive(Debug, Clone, Default)]
pub struct State {
    /// The currently loaded media (image or video).
    media: Option<MediaData>,
    /// Path to the current media file.
    current_path: Option<PathBuf>,
}

/// Messages for the media holder sub-component.
#[derive(Debug, Clone)]
pub enum Message {
    /// Set new media data after successful load.
    SetMedia { data: MediaData, path: PathBuf },
    /// Clear all media state (no media loaded).
    Clear,
}

/// Effects produced by media holder changes.
#[derive(Debug, Clone)]
pub enum Effect {
    /// No effect.
    None,
    /// Media changed - view needs update.
    MediaChanged,
    /// Media cleared - view should show empty state.
    MediaCleared,
}

impl State {
    /// Handle a media holder message.
    pub fn handle(&mut self, msg: Message) -> Effect {
        match msg {
            Message::SetMedia { data, path } => {
                self.media = Some(data);
                self.current_path = Some(path);
                Effect::MediaChanged
            }
            Message::Clear => {
                self.media = None;
                self.current_path = None;
                Effect::MediaCleared
            }
        }
    }

    /// Get the current media data.
    #[must_use]
    pub fn media(&self) -> Option<&MediaData> {
        self.media.as_ref()
    }

    /// Get the current media data mutably.
    pub fn media_mut(&mut self) -> Option<&mut MediaData> {
        self.media.as_mut()
    }

    /// Take ownership of the current media data.
    #[must_use]
    pub fn take_media(&mut self) -> Option<MediaData> {
        self.media.take()
    }

    /// Get the current media path.
    #[must_use]
    pub fn current_path(&self) -> Option<&PathBuf> {
        self.current_path.as_ref()
    }

    /// Check if any media is loaded.
    #[must_use]
    pub fn has_media(&self) -> bool {
        self.media.is_some()
    }

    /// Check if the loaded media is an image.
    #[must_use]
    pub fn is_image(&self) -> bool {
        matches!(self.media, Some(MediaData::Image(_)))
    }

    /// Check if the loaded media is a video.
    #[must_use]
    pub fn is_video(&self) -> bool {
        matches!(self.media, Some(MediaData::Video(_)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::ImageData;

    fn sample_image_data() -> MediaData {
        let pixels = vec![0_u8; 100 * 100 * 4];
        MediaData::Image(ImageData::from_rgba(100, 100, pixels))
    }

    #[test]
    fn default_state_has_no_media() {
        let state = State::default();
        assert!(!state.has_media());
        assert!(state.media().is_none());
        assert!(state.current_path().is_none());
    }

    #[test]
    fn set_media_stores_data_and_path() {
        let mut state = State::default();
        let path = PathBuf::from("/test/image.jpg");

        let effect = state.handle(Message::SetMedia {
            data: sample_image_data(),
            path: path.clone(),
        });

        assert!(matches!(effect, Effect::MediaChanged));
        assert!(state.has_media());
        assert!(state.is_image());
        assert_eq!(state.current_path(), Some(&path));
    }

    #[test]
    fn clear_removes_media() {
        let mut state = State::default();
        let path = PathBuf::from("/test/image.jpg");

        state.handle(Message::SetMedia {
            data: sample_image_data(),
            path,
        });
        assert!(state.has_media());

        let effect = state.handle(Message::Clear);

        assert!(matches!(effect, Effect::MediaCleared));
        assert!(!state.has_media());
        assert!(state.current_path().is_none());
    }

    #[test]
    fn take_media_removes_and_returns() {
        let mut state = State::default();
        let path = PathBuf::from("/test/image.jpg");

        state.handle(Message::SetMedia {
            data: sample_image_data(),
            path,
        });

        let taken = state.take_media();
        assert!(taken.is_some());
        assert!(!state.has_media());
        // Path remains even after taking media
        assert!(state.current_path().is_some());
    }

    #[test]
    fn is_image_returns_correct_value() {
        let mut state = State::default();
        assert!(!state.is_image());

        state.handle(Message::SetMedia {
            data: sample_image_data(),
            path: PathBuf::from("/test/image.jpg"),
        });
        assert!(state.is_image());
        assert!(!state.is_video());
    }
}
