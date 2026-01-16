// SPDX-License-Identifier: MPL-2.0
//! Viewer component encapsulating state and update logic.

use crate::error::{Error, VideoError};
use crate::i18n::fluent::I18n;
use crate::media::navigator::NavigationInfo;
use crate::media::{MaxSkipAttempts, MediaData};
use crate::ui::state::{DragState, RotationAngle, ViewportState, ZoomState, ZoomStep};
use crate::ui::viewer::{
    self, clusters, controls, filter_dropdown, pane, state as geometry, subcomponents,
    video_controls, HudIconKind, HudLine,
};
use crate::ui::widgets::VideoShader;
use crate::video_player::{
    subscription::PlaybackMessage, KeyboardSeekStep, SharedLufsCache, VideoPlayer, Volume,
};
use iced::widget::scrollable::{AbsoluteOffset, RelativeOffset};
use iced::widget::{operation, Id};
use iced::{event, keyboard, mouse, window, Element, Point, Rectangle, Task};
use std::path::PathBuf;

/// Identifier used for the viewer scrollable widget.
pub const SCROLLABLE_ID: &str = "viewer-image-scrollable";

/// Messages emitted by viewer-related widgets.
#[derive(Debug, Clone)]
pub enum Message {
    StartLoadingMedia,
    MediaLoaded(Result<MediaData, Error>),
    /// Clear all media state (used when no media is available, e.g., after deleting last media).
    ClearMedia,
    ToggleErrorDetails,
    Controls(controls::Message),
    VideoControls(video_controls::Message),
    ViewportChanged {
        bounds: Rectangle,
        offset: AbsoluteOffset,
    },
    RawEvent {
        window: window::Id,
        event: event::Event,
    },
    NavigateNext,
    NavigatePrevious,
    DeleteCurrentImage,
    OpenSettings,
    EnterEditor,
    InitiatePlayback,
    PlaybackEvent(PlaybackMessage),
    SpinnerTick,
    /// Request to open file dialog from empty state.
    OpenFileRequested,
    /// Rotate current media 90° clockwise (temporary, session-only).
    RotateClockwise,
    /// Rotate current media 90° counter-clockwise (temporary, session-only).
    RotateCounterClockwise,
    /// Filter dropdown messages (routed from navbar).
    FilterDropdown(filter_dropdown::Message),
}

/// Direction of navigation for auto-skip retry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationDirection {
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
        direction: NavigationDirection,
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

/// Side effects the application should perform after handling a viewer message.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    None,
    PersistPreferences,
    ToggleFullscreen,
    ExitFullscreen,
    OpenSettings,
    EnterEditor,
    NavigateNext,
    NavigatePrevious,
    /// Capture current frame and open editor.
    /// Contains the captured frame data and metadata for filename generation.
    CaptureFrame {
        frame: crate::media::frame_export::ExportableFrame,
        video_path: PathBuf,
        position_secs: f64,
    },
    /// Request to delete the current media file.
    /// App will handle the actual deletion using `media_navigator`.
    RequestDelete,
    /// Toggle the info/metadata panel.
    ToggleInfoPanel,
    /// Request to open file dialog (from empty state).
    OpenFileDialog,
    /// Show error notification (used when load fails with no media loaded).
    ShowErrorNotification {
        /// The i18n key for the notification message.
        key: &'static str,
        /// Optional arguments for the i18n message.
        args: Vec<(&'static str, String)>,
    },
    /// Retry navigation after a failed load (auto-skip).
    /// App will navigate in the given direction and try to load the next media.
    RetryNavigation {
        /// Direction to retry navigation.
        direction: NavigationDirection,
        /// Number of consecutive skip attempts so far.
        skip_attempts: u32,
        /// Filenames that have been skipped.
        skipped_files: Vec<String>,
    },
    /// Show grouped notification for skipped files after max attempts reached.
    ShowSkippedFilesNotification {
        /// Filenames that were skipped.
        skipped_files: Vec<String>,
    },
    /// Confirm navigation after successful media load.
    /// App will update `MediaNavigator`'s position to the loaded path.
    ConfirmNavigation {
        /// Path to confirm as the current position.
        path: PathBuf,
        /// Filenames that were skipped during navigation (if any).
        skipped_files: Vec<String>,
    },
    /// Filter changed via dropdown. App should update navigator's filter.
    FilterChanged(filter_dropdown::Message),
    /// Loading timed out. App should show timeout notification.
    LoadingTimedOut,
}

/// Error state for displaying user-friendly errors with optional details.
/// Currently not used (errors handled via notifications), but kept for potential future use.
/// Uses the sub-component implementation for consistency.
pub type ErrorState = subcomponents::error_state::State;

/// Environment information required to render the viewer.
#[allow(clippy::struct_field_names)] // Fields describe their content, not the struct
pub struct ViewEnv<'a> {
    pub i18n: &'a I18n,
    pub background_theme: crate::config::BackgroundTheme,
    pub is_fullscreen: bool,
    pub overlay_hide_delay: std::time::Duration,
    /// Navigation state from the central `MediaNavigator`.
    /// This is the single source of truth for navigation info.
    pub navigation: NavigationInfo,
    /// Whether metadata editor has unsaved changes (disables navigation).
    pub metadata_editor_has_changes: bool,
    /// Current media filter (reference to navigator's filter).
    pub filter: &'a crate::media::filter::MediaFilter,
}

/// Complete viewer component state.
#[allow(clippy::struct_excessive_bools)] // Complex UI state requires multiple boolean flags
pub struct State {
    pub viewport: ViewportState,

    /// Image transformation cluster (zoom, drag/pan, rotation).
    image_transform: clusters::image_transform::State,

    /// Media lifecycle cluster (loading, media holder, errors).
    media_lifecycle: clusters::media_lifecycle::State,

    pub current_media_path: Option<PathBuf>,

    /// Overlay visibility sub-component (fullscreen controls auto-hide).
    overlay: subcomponents::overlay::State,

    /// Origin of the current media load request (for auto-skip behavior).
    pub load_origin: LoadOrigin,
    /// Maximum number of consecutive corrupted files to skip during navigation.
    pub max_skip_attempts: MaxSkipAttempts,

    /// Video playback cluster (player state, settings, seek management).
    video_playback: clusters::video_playback::State,

    /// Video shader widget (rendering, stays in component.rs as it's UI-specific).
    video_shader: VideoShader<Message>,

    /// Filter dropdown UI state.
    filter_dropdown: filter_dropdown::FilterDropdownState,

    /// Diagnostics handle for logging events (set from external context).
    /// Used for internal message dispatches (e.g., keyboard shortcuts).
    diagnostics: Option<crate::diagnostics::DiagnosticsHandle>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            viewport: ViewportState::default(),
            image_transform: clusters::image_transform::State::default(),
            media_lifecycle: clusters::media_lifecycle::State::default(),
            current_media_path: None,
            overlay: subcomponents::overlay::State::default(),
            load_origin: LoadOrigin::DirectOpen,
            max_skip_attempts: MaxSkipAttempts::default(),
            video_playback: clusters::video_playback::State::default(),
            video_shader: VideoShader::new(),
            filter_dropdown: filter_dropdown::FilterDropdownState::default(),
            diagnostics: None,
        }
    }
}

impl State {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has_media(&self) -> bool {
        self.media_lifecycle.has_media()
    }

    pub fn media(&self) -> Option<&MediaData> {
        self.media_lifecycle.media()
    }

    pub fn error(&self) -> Option<&ErrorState> {
        self.media_lifecycle.error()
    }

    /// Check if currently loading media.
    pub fn is_loading(&self) -> bool {
        self.media_lifecycle.is_loading()
    }

    /// Get the spinner rotation angle.
    pub fn spinner_rotation(&self) -> f32 {
        self.media_lifecycle.spinner_rotation()
    }

    pub fn zoom_state(&self) -> &ZoomState {
        &self.image_transform.zoom
    }

    pub fn zoom_state_mut(&mut self) -> &mut ZoomState {
        &mut self.image_transform.zoom
    }

    pub fn viewport_state(&self) -> &ViewportState {
        &self.viewport
    }

    pub fn viewport_state_mut(&mut self) -> &mut ViewportState {
        &mut self.viewport
    }

    pub fn drag_state(&self) -> &DragState {
        &self.image_transform.drag.inner
    }

    pub fn drag_state_mut(&mut self) -> &mut DragState {
        &mut self.image_transform.drag.inner
    }

    /// Get the cursor position within the viewer.
    pub fn cursor_position(&self) -> Option<Point> {
        self.image_transform.cursor_position()
    }

    /// Update the cursor position.
    pub fn set_cursor_position(&mut self, position: Option<Point>) {
        self.image_transform.drag.cursor_position = position;
    }

    /// Closes the filter dropdown panel.
    pub fn close_filter_dropdown(&mut self) {
        self.filter_dropdown.close();
    }

    /// Returns a reference to the filter dropdown state.
    pub fn filter_dropdown_state(&self) -> &filter_dropdown::FilterDropdownState {
        &self.filter_dropdown
    }

    /// Returns the current temporary rotation angle.
    pub fn current_rotation(&self) -> RotationAngle {
        self.image_transform.rotation_angle()
    }

    /// Returns true if the current media is an image (not a video).
    fn is_current_media_image(&self) -> bool {
        self.media_lifecycle.is_image()
    }

    /// Returns the cached rotated image if available.
    pub fn rotated_image_cache(&self) -> Option<&crate::media::ImageData> {
        self.image_transform.cached_rotated_image()
    }

    /// Handle rotation effect from `image_transform` cluster and rebuild cache if needed.
    fn handle_rotation_changed(&mut self) {
        // Rebuild cache - cluster doesn't have access to media
        if let Some(MediaData::Image(ref image_data)) = self.media_lifecycle.media() {
            if self.image_transform.is_rotated() {
                let rotated = image_data.rotated(self.image_transform.rotation_angle().degrees());
                self.image_transform.set_rotation_cache(rotated);
            } else {
                self.image_transform.clear_rotation_cache();
            }
        } else {
            self.image_transform.clear_rotation_cache();
        }
        // Refresh fit zoom for rotated dimensions
        self.refresh_fit_zoom();
    }

    /// Internal message dispatch using stored diagnostics handle.
    ///
    /// Used by keyboard handlers and other internal code paths that need to
    /// dispatch messages without an external diagnostics reference.
    /// Returns `(Effect::None, Task::none())` if no diagnostics handle is available.
    fn dispatch_message(&mut self, message: Message) -> (Effect, Task<Message>) {
        if let Some(ref diagnostics) = self.diagnostics.clone() {
            self.handle_message(message, &I18n::default(), diagnostics)
        } else {
            // No diagnostics handle available - this shouldn't happen in normal operation
            // but we handle it gracefully
            (Effect::None, Task::none())
        }
    }

    /// Returns true if the video overflow menu (advanced controls) is open.
    pub fn is_overflow_menu_open(&self) -> bool {
        self.video_playback.is_overflow_menu_open()
    }

    /// Resets the viewport offset to zero, causing the media to recenter.
    /// Call this when the available viewport area changes (e.g., sidebar toggle).
    pub fn reset_viewport_offset(&mut self) {
        self.viewport.reset_offset();
    }

    pub fn zoom_step_percent(&self) -> f32 {
        self.image_transform.zoom.zoom_step.value()
    }

    pub fn set_zoom_step_percent(&mut self, value: f32) {
        self.image_transform.zoom.zoom_step = ZoomStep::new(value);
    }

    /// Returns the effective fit-to-window setting.
    /// For videos, uses the separate `video_playback.fit_to_window` (not persisted).
    /// For images, uses `zoom.fit_to_window` (persisted).
    pub fn fit_to_window(&self) -> bool {
        if self.is_video() {
            self.video_playback.fit_to_window()
        } else {
            self.image_transform.zoom.fit_to_window
        }
    }

    /// Returns the image fit-to-window setting (persisted).
    /// Use this when saving preferences - only saves image setting.
    pub fn image_fit_to_window(&self) -> bool {
        self.image_transform.zoom.fit_to_window
    }

    /// Returns true if the current media is a video.
    pub fn is_video(&self) -> bool {
        self.media_lifecycle.is_video()
    }

    /// Returns the current seek preview position if one is set.
    ///
    /// This is used by the App layer to log `SeekVideo` actions at handler level.
    pub fn seek_preview_position(&self) -> Option<f64> {
        self.video_playback.seek_preview_position()
    }

    /// Returns true if a video is playing or will resume playing after seek/buffer.
    ///
    /// This determines if arrow keys should seek (true) vs navigate (false).
    /// Uses the state machine's `is_playing_or_will_resume()` to correctly handle
    /// the Seeking state during rapid key repeats.
    fn is_video_playing_or_will_resume(&self) -> bool {
        self.video_playback.is_playing_or_will_resume()
    }

    /// Returns true if a video player exists and has an active session.
    ///
    /// An active session means the player is not stopped or in error state.
    /// This is used to determine if Space should toggle playback vs initiate.
    fn has_active_video_session(&self) -> bool {
        self.video_playback.has_active_session()
    }

    pub fn enable_fit_to_window(&mut self) {
        if self.is_video() {
            self.video_playback
                .handle(clusters::video_playback::Message::SetFitToWindow(true));
        } else {
            self.image_transform.zoom.enable_fit_to_window();
        }
    }

    pub fn disable_fit_to_window(&mut self) {
        if self.is_video() {
            self.video_playback
                .handle(clusters::video_playback::Message::SetFitToWindow(false));
        } else {
            self.image_transform.zoom.disable_fit_to_window();
        }
    }

    pub fn refresh_error_translation(&mut self, i18n: &I18n) {
        self.media_lifecycle.handle(
            clusters::media_lifecycle::Message::RefreshTranslations,
            i18n,
        );
    }

    /// Sets whether videos should auto-play when loaded.
    pub fn set_video_autoplay(&mut self, enabled: bool) {
        self.video_playback
            .handle(clusters::video_playback::Message::SetAutoplay(enabled));
    }

    /// Sets the video volume level (0.0 to 1.0).
    pub fn set_video_volume(&mut self, volume: f32) {
        self.video_playback.set_volume_raw(volume);
    }

    /// Returns the current video volume level.
    pub fn video_volume(&self) -> f32 {
        self.video_playback.volume()
    }

    /// Sets whether video audio is muted.
    pub fn set_video_muted(&mut self, muted: bool) {
        self.video_playback.set_muted_raw(muted);
    }

    /// Returns whether video audio is muted.
    pub fn video_muted(&self) -> bool {
        self.video_playback.is_muted()
    }

    /// Sets whether video playback should loop.
    pub fn set_video_loop(&mut self, enabled: bool) {
        self.video_playback.set_loop_raw(enabled);
    }

    /// Returns whether video playback loops.
    pub fn video_loop(&self) -> bool {
        self.video_playback.is_loop_enabled()
    }

    /// Returns the current video playback position in seconds (for diagnostics logging).
    ///
    /// Returns `None` if no video is loaded.
    pub fn video_position(&self) -> Option<f64> {
        self.video_playback.position()
    }

    /// Returns the current video playback speed (for diagnostics logging).
    ///
    /// Returns `None` if no video is loaded.
    pub fn video_playback_speed(&self) -> Option<f64> {
        self.video_playback.playback_speed()
    }

    /// Sets the keyboard seek step.
    pub fn set_keyboard_seek_step(&mut self, step: KeyboardSeekStep) {
        self.video_playback
            .handle(clusters::video_playback::Message::SetKeyboardSeekStep(step));
    }

    /// Sets the maximum number of skip attempts for auto-skip.
    pub fn set_max_skip_attempts(&mut self, max_attempts: MaxSkipAttempts) {
        self.max_skip_attempts = max_attempts;
    }

    /// Sets the origin of the current media load request.
    ///
    /// This determines auto-skip behavior when loading fails:
    /// - `LoadOrigin::Navigation`: Auto-skip to next/previous on failure
    /// - `LoadOrigin::DirectOpen`: Show error notification, stay on current media
    pub fn set_load_origin(&mut self, origin: LoadOrigin) {
        self.load_origin = origin;
    }

    /// Sets the load origin for navigation with initial state.
    ///
    /// Use this when starting a new navigation sequence.
    pub fn set_navigation_origin(&mut self, direction: NavigationDirection) {
        self.load_origin = LoadOrigin::Navigation {
            direction,
            skip_attempts: 0,
            skipped_files: Vec::new(),
        };
    }

    /// Sets the load origin for direct open (drag-drop, file dialog, CLI).
    pub fn set_direct_open_origin(&mut self) {
        self.load_origin = LoadOrigin::DirectOpen;
    }

    /// Starts loading a new media file.
    ///
    /// Sets loading indicators that will be cleared by the `MediaLoaded` message handler.
    /// This encapsulates the loading state management that was previously scattered
    /// across multiple app handlers.
    pub fn start_loading(&mut self) {
        // Clear video shader immediately to prevent stale frame from being rendered
        // with wrong dimensions when navigating to a different media
        self.video_shader.clear();

        // Determine media type and size for diagnostics
        let media_type = self
            .current_media_path
            .as_ref()
            .and_then(|p| p.extension())
            .map(|ext| {
                let ext = ext.to_string_lossy().to_lowercase();
                if ["mp4", "webm", "avi", "mkv", "mov", "m4v", "wmv", "flv"].contains(&ext.as_str())
                {
                    crate::diagnostics::MediaType::Video
                } else {
                    crate::diagnostics::MediaType::Image
                }
            });

        let file_size_bytes = self
            .current_media_path
            .as_ref()
            .and_then(|p| std::fs::metadata(p).ok())
            .map(|m| m.len());

        // Delegate to media_lifecycle cluster (clears error + starts loading)
        self.media_lifecycle.handle(
            clusters::media_lifecycle::Message::StartLoading {
                media_type,
                file_size: file_size_bytes,
            },
            &I18n::default(),
        );
    }

    /// Returns an exportable frame from the video canvas, if available.
    pub fn exportable_frame(&self) -> Option<crate::media::frame_export::ExportableFrame> {
        self.video_shader.exportable_frame()
    }

    /// Returns true if media is currently being loaded.
    pub fn is_loading_media(&self) -> bool {
        self.media_lifecycle.is_loading()
    }

    /// Handle loading timeout effect from `media_lifecycle` cluster.
    fn handle_loading_timeout(&mut self) {
        self.media_lifecycle.handle(
            clusters::media_lifecycle::Message::StopLoading,
            &I18n::default(),
        );
        self.current_media_path = None;
    }

    /// Returns the subscriptions for video playback and spinner animation.
    ///
    /// # Arguments
    /// * `lufs_cache` - Optional shared cache for LUFS measurements (audio normalization)
    /// * `normalization_enabled` - Whether to apply audio normalization
    /// * `frame_cache_mb` - Maximum memory for frame cache (seek optimization), in MB
    /// * `history_mb` - Maximum memory for frame history (backward stepping), in MB
    pub fn subscription(
        &self,
        lufs_cache: Option<SharedLufsCache>,
        normalization_enabled: bool,
        frame_cache_mb: u32,
        history_mb: u32,
    ) -> iced::Subscription<Message> {
        // Keep subscription active for ALL playback states including Stopped
        // This ensures the decoder stays alive and can receive pause/resume commands
        // The subscription only gets recreated when playback_session_id changes
        // (which happens when navigating to a different video or starting fresh)
        let video_subscription =
            if let (true, Some(path)) = (self.video_playback.has_player(), self.video_playback.current_path()) {
                // Create cache config from MB setting
                let cache_config = crate::video_player::CacheConfig::new(
                    (frame_cache_mb as usize) * 1024 * 1024,
                    crate::video_player::frame_cache::DEFAULT_MAX_FRAMES,
                );

                // Always create subscription when we have a video player and path
                // The decoder will handle pause/resume via commands
                crate::video_player::subscription::video_playback(
                    path.clone(),
                    self.video_playback.session_id(),
                    lufs_cache,
                    normalization_enabled,
                    cache_config,
                    history_mb,
                )
                .map(Message::PlaybackEvent)
            } else {
                iced::Subscription::none()
            };

        let spinner_subscription = if self.media_lifecycle.is_loading() {
            // Animate spinner at 60 FPS while loading
            iced::time::every(std::time::Duration::from_millis(16)).map(|_| Message::SpinnerTick)
        } else {
            iced::Subscription::none()
        };

        iced::Subscription::batch([video_subscription, spinner_subscription])
    }

    #[allow(clippy::too_many_lines)] // Message handler with many variants, inherent complexity
    pub fn handle_message(
        &mut self,
        message: Message,
        _i18n: &I18n,
        diagnostics: &crate::diagnostics::DiagnosticsHandle,
    ) -> (Effect, Task<Message>) {
        // Store diagnostics handle for internal use (keyboard shortcuts, etc.)
        self.diagnostics = Some(diagnostics.clone());
        match message {
            Message::StartLoadingMedia => {
                // Set loading state via encapsulated method
                self.start_loading();
                (Effect::None, Task::none())
            }
            Message::ClearMedia => {
                // Clear all media state - used when no media is available
                // (e.g., after deleting the last media in directory)

                // Stop any video playback via cluster
                self.video_playback
                    .handle(clusters::video_playback::Message::ResetForNewMedia);
                self.video_shader.clear_frame();

                // Delegate to media_lifecycle cluster (clears media, error, loading)
                self.media_lifecycle.handle(
                    clusters::media_lifecycle::Message::ClearMedia,
                    &I18n::default(),
                );
                self.current_media_path = None;

                // Reset image transformation state (zoom, drag, rotation)
                self.image_transform = clusters::image_transform::State::default();
                self.viewport = ViewportState::default();

                (Effect::None, Task::none())
            }
            Message::MediaLoaded(result) => {
                // Clear loading state via cluster
                self.media_lifecycle.handle(
                    clusters::media_lifecycle::Message::StopLoading,
                    &I18n::default(),
                );

                // Clean up previous video state before loading new media
                // This is important when navigating from one media to another
                if self.video_playback.has_player() {
                    // Reset video playback state via cluster
                    self.video_playback
                        .handle(clusters::video_playback::Message::ResetForNewMedia);
                    self.video_shader.clear(); // Clear frame to release memory
                    // Increment session ID to ensure old subscription is dropped
                    self.video_playback.increment_session_id();
                }

                // Reset image transformation for new media
                self.image_transform
                    .handle(clusters::image_transform::Message::ResetForNewMedia);

                match result {
                    Ok(media) => {
                        // Store the path before passing media to cluster
                        let path = self.current_media_path.clone().unwrap_or_default();

                        // Set media via cluster (also clears any error)
                        self.media_lifecycle.handle(
                            clusters::media_lifecycle::Message::MediaLoaded {
                                data: media,
                                path: path.clone(),
                            },
                            &I18n::default(),
                        );

                        // Create VideoPlayer if this is a video (after media is stored)
                        if let Some(MediaData::Video(ref video_data)) = self.media_lifecycle.media()
                        {
                            if let Err(e) = self.video_playback.create_player(
                                video_data,
                                self.current_media_path.clone(),
                                self.diagnostics.clone(),
                            ) {
                                eprintln!("Failed to create video player: {e}");
                            }
                        }

                        // Extract skipped files from navigation origin (if any)
                        let skipped_files =
                            if let LoadOrigin::Navigation { skipped_files, .. } =
                                std::mem::take(&mut self.load_origin)
                            {
                                skipped_files
                            } else {
                                Vec::new()
                            };

                        // Confirm navigation with the path and any skipped files
                        let effect = if let Some(ref path) = self.current_media_path {
                            Effect::ConfirmNavigation {
                                path: path.clone(),
                                skipped_files,
                            }
                        } else {
                            // Fallback: no path, just show skipped files if any
                            if skipped_files.is_empty() {
                                Effect::None
                            } else {
                                Effect::ShowSkippedFilesNotification { skipped_files }
                            }
                        };

                        // Reset viewport offset for new media (ensures proper centering)
                        self.viewport.reset_offset();

                        // Reset zoom to 100% for images when fit-to-window is disabled
                        if !self.is_video() && !self.image_fit_to_window() {
                            self.image_transform
                                .zoom
                                .apply_manual_zoom(crate::ui::state::zoom::DEFAULT_ZOOM_PERCENT);
                        }

                        self.refresh_fit_zoom();

                        // Scroll the widget to origin to match the reset offset
                        let scroll_task = operation::snap_to(
                            Id::new(SCROLLABLE_ID),
                            RelativeOffset { x: 0.0, y: 0.0 },
                        );
                        (effect, scroll_task)
                    }
                    Err(error) => {
                        // loading_media_type/file_size cleared by StopLoading message

                        // Get the failed filename for the notification
                        let failed_filename = self
                            .current_media_path
                            .as_ref()
                            .and_then(|p| p.file_name())
                            .map_or_else(
                                || "unknown".to_string(),
                                |n| n.to_string_lossy().to_string(),
                            );

                        // Handle based on load origin
                        match std::mem::take(&mut self.load_origin) {
                            LoadOrigin::Navigation {
                                direction,
                                skip_attempts,
                                mut skipped_files,
                            } => {
                                // Add failed file to the list
                                skipped_files.push(failed_filename);
                                let new_attempts = skip_attempts + 1;

                                if new_attempts <= self.max_skip_attempts.value() {
                                    // Auto-skip: retry navigation in the same direction
                                    // Keep current_media_path so handle_retry_navigation knows
                                    // which file failed and can advance past it
                                    (
                                        Effect::RetryNavigation {
                                            direction,
                                            skip_attempts: new_attempts,
                                            skipped_files,
                                        },
                                        Task::none(),
                                    )
                                } else {
                                    // Max attempts reached: clear path and show notification
                                    self.current_media_path = None;
                                    (
                                        Effect::ShowSkippedFilesNotification { skipped_files },
                                        Task::none(),
                                    )
                                }
                            }
                            LoadOrigin::DirectOpen => {
                                // Direct open: clear path and show error notification
                                self.current_media_path = None;
                                let notification_key = match &error {
                                    Error::Svg(_) => "notification-load-error-svg",
                                    Error::Video(_) => "notification-load-error-video",
                                    Error::Io(_) | Error::Config(_) => "notification-load-error-io",
                                };
                                (
                                    Effect::ShowErrorNotification {
                                        key: notification_key,
                                        args: vec![],
                                    },
                                    Task::none(),
                                )
                            }
                        }
                    }
                }
            }
            Message::ToggleErrorDetails => {
                self.media_lifecycle.handle(
                    clusters::media_lifecycle::Message::ToggleErrorDetails,
                    &I18n::default(),
                );
                (Effect::None, Task::none())
            }
            Message::Controls(control) => {
                if matches!(control, controls::Message::DeleteCurrentImage) {
                    return (Effect::RequestDelete, Task::none());
                }
                // No need to sync shader scale - pane calculates display size from zoom at render time
                self.handle_controls(control)
            }
            Message::ViewportChanged { bounds, offset } => {
                let bounds_changed = self.viewport.update(bounds, offset);
                // When viewport size changes significantly (e.g., sidebar toggle), reset to recenter
                if bounds_changed {
                    self.viewport.reset_offset();
                    // Recalculate fit zoom for new viewport size
                    self.refresh_fit_zoom();
                    // Scroll the widget to origin to match the reset offset
                    let scroll_task = operation::snap_to(
                        Id::new(SCROLLABLE_ID),
                        RelativeOffset { x: 0.0, y: 0.0 },
                    );
                    return (Effect::None, scroll_task);
                }
                (Effect::None, Task::none())
            }
            Message::RawEvent { event, .. } => self.handle_raw_event(event),
            Message::NavigateNext => {
                // Stop video playback immediately to prevent rendering issues during navigation
                self.video_playback
                    .handle(clusters::video_playback::Message::Pause);
                // Cancel any ongoing drag (user clicked on navigation overlay)
                self.image_transform.drag.inner.stop();
                // Reset overlay timer on navigation
                self.overlay
                    .handle(subcomponents::overlay::Message::OverlayInteraction);
                // Emit effect to let App handle navigation with MediaNavigator
                (Effect::NavigateNext, Task::none())
            }
            Message::NavigatePrevious => {
                // Stop video playback immediately to prevent rendering issues during navigation
                self.video_playback
                    .handle(clusters::video_playback::Message::Pause);
                // Cancel any ongoing drag (user clicked on navigation overlay)
                self.image_transform.drag.inner.stop();
                // Reset overlay timer on navigation
                self.overlay
                    .handle(subcomponents::overlay::Message::OverlayInteraction);
                // Emit effect to let App handle navigation with MediaNavigator
                (Effect::NavigatePrevious, Task::none())
            }
            Message::DeleteCurrentImage => (Effect::RequestDelete, Task::none()),
            Message::OpenSettings => (Effect::OpenSettings, Task::none()),
            Message::EnterEditor => (Effect::EnterEditor, Task::none()),
            Message::OpenFileRequested => (Effect::OpenFileDialog, Task::none()),
            Message::RotateClockwise => {
                // Rotation only applies to images, not videos
                if self.is_current_media_image() {
                    let effect = self
                        .image_transform
                        .handle(clusters::image_transform::Message::RotateClockwise);
                    if matches!(effect, clusters::image_transform::Effect::RotationChanged) {
                        self.handle_rotation_changed();
                    }
                }
                (Effect::None, Task::none())
            }
            Message::RotateCounterClockwise => {
                // Rotation only applies to images, not videos
                if self.is_current_media_image() {
                    let effect = self
                        .image_transform
                        .handle(clusters::image_transform::Message::RotateCounterClockwise);
                    if matches!(effect, clusters::image_transform::Effect::RotationChanged) {
                        self.handle_rotation_changed();
                    }
                }
                (Effect::None, Task::none())
            }
            Message::InitiatePlayback => {
                // Reset overlay timer on interaction
                self.overlay
                    .handle(subcomponents::overlay::Message::OverlayInteraction);

                // Toggle playback if player already exists
                if self.video_playback.has_player() {
                    self.video_playback
                        .handle(clusters::video_playback::Message::TogglePlayback);
                } else {
                    // Check if current media is a video and get data for player creation
                    let video_data = self.media().and_then(|m| {
                        if let MediaData::Video(ref v) = m {
                            Some(v.clone())
                        } else {
                            None
                        }
                    });
                    if let Some(ref data) = video_data {
                        // Create video player and start playback
                        if let Err(e) = self.video_playback.create_player(
                            data,
                            self.current_media_path.clone(),
                            self.diagnostics.clone(),
                        ) {
                            eprintln!("Failed to create video player: {e}");
                        } else {
                            // Start playback
                            self.video_playback
                                .handle(clusters::video_playback::Message::Play);
                        }
                    }
                }

                (Effect::None, Task::none())
            }
            Message::SpinnerTick => {
                // Delegate to media_lifecycle cluster
                let effect = self.media_lifecycle.handle(
                    clusters::media_lifecycle::Message::SpinnerTick,
                    &I18n::default(),
                );

                // Handle timeout effect
                if matches!(effect, clusters::media_lifecycle::Effect::LoadingTimedOut) {
                    self.handle_loading_timeout();
                    return (Effect::LoadingTimedOut, Task::none());
                }
                (Effect::None, Task::none())
            }
            Message::VideoControls(video_msg) => {
                use super::video_controls::Message as VM;

                // Reset overlay timer on video control interaction
                self.overlay
                    .handle(subcomponents::overlay::Message::OverlayInteraction);

                match video_msg {
                    VM::TogglePlayback => {
                        // Logging now handled at App layer (R1: collect at handler level)
                        if self.video_playback.has_player() {
                            self.video_playback
                                .handle(clusters::video_playback::Message::TogglePlayback);
                        } else {
                            // Check if current media is a video
                            let video_data = self.media().and_then(|m| {
                                if let MediaData::Video(ref v) = m {
                                    Some(v.clone())
                                } else {
                                    None
                                }
                            });
                            if let Some(ref data) = video_data {
                                // Create player if it doesn't exist yet and start playback
                                if let Err(e) = self.video_playback.create_player(
                                    data,
                                    self.current_media_path.clone(),
                                    self.diagnostics.clone(),
                                ) {
                                    eprintln!("Failed to create video player: {e}");
                                } else {
                                    self.video_playback
                                        .handle(clusters::video_playback::Message::Play);
                                }
                            }
                        }
                    }
                    VM::SeekPreview(position) => {
                        self.video_playback
                            .handle(clusters::video_playback::Message::SeekPreview(position));
                    }
                    VM::SeekCommit => {
                        self.video_playback
                            .handle(clusters::video_playback::Message::SeekCommit);
                    }
                    VM::SeekRelative(delta_secs) => {
                        self.video_playback
                            .handle(clusters::video_playback::Message::SeekRelative(delta_secs));
                    }
                    VM::SetVolume(volume) => {
                        let effect = self
                            .video_playback
                            .handle(clusters::video_playback::Message::SetVolume(volume));
                        if matches!(effect, clusters::video_playback::Effect::PersistPreferences) {
                            return (Effect::PersistPreferences, Task::none());
                        }
                    }
                    VM::ToggleMute => {
                        let effect = self
                            .video_playback
                            .handle(clusters::video_playback::Message::ToggleMute);
                        if matches!(effect, clusters::video_playback::Effect::PersistPreferences) {
                            return (Effect::PersistPreferences, Task::none());
                        }
                    }
                    VM::ToggleLoop => {
                        let effect = self
                            .video_playback
                            .handle(clusters::video_playback::Message::ToggleLoop);
                        if matches!(effect, clusters::video_playback::Effect::PersistPreferences) {
                            return (Effect::PersistPreferences, Task::none());
                        }
                    }
                    VM::CaptureFrame => {
                        // Pause the video if playing
                        self.video_playback
                            .handle(clusters::video_playback::Message::Pause);

                        // Capture current frame and open editor
                        if let Some(video_path) = self.video_playback.current_path() {
                            if let Some(frame) = self.exportable_frame() {
                                let position_secs =
                                    self.video_playback.position().unwrap_or(0.0);
                                return (
                                    Effect::CaptureFrame {
                                        frame,
                                        video_path: video_path.clone(),
                                        position_secs,
                                    },
                                    Task::none(),
                                );
                            }
                        }
                    }
                    VM::StepForward => {
                        self.video_playback
                            .handle(clusters::video_playback::Message::StepForward);
                    }
                    VM::StepBackward => {
                        self.video_playback
                            .handle(clusters::video_playback::Message::StepBackward);
                    }
                    VM::ToggleOverflowMenu => {
                        self.video_playback
                            .handle(clusters::video_playback::Message::ToggleOverflowMenu);
                    }
                    VM::IncreasePlaybackSpeed => {
                        self.video_playback
                            .handle(clusters::video_playback::Message::IncreaseSpeed);
                    }
                    VM::DecreasePlaybackSpeed => {
                        self.video_playback
                            .handle(clusters::video_playback::Message::DecreaseSpeed);
                    }
                }
                (Effect::None, Task::none())
            }
            Message::PlaybackEvent(event) => {
                // Check for seeking timeout BEFORE processing event
                if let Some(player) = self.video_playback.player_mut() {
                    if matches!(
                        player.state(),
                        crate::video_player::PlaybackState::Seeking { .. }
                    ) {
                        if let Some(duration) = player.seeking_duration() {
                            if duration
                                > std::time::Duration::from_secs(
                                    crate::config::SEEKING_TIMEOUT_SECS,
                                )
                            {
                                #[cfg(debug_assertions)]
                                eprintln!(
                                    "[seeking] Timeout reached ({:.2}s), forcing state transition",
                                    duration.as_secs_f32()
                                );

                                // Force complete the seek operation
                                player.force_complete_seek();
                            }
                        }
                    }
                }

                match event {
                    PlaybackMessage::Started(command_sender) => {
                        // Get all settings BEFORE borrowing player_mut (avoids borrow conflict)
                        let autoplay = self.video_playback.is_autoplay();
                        let volume = self.video_playback.volume();
                        let muted = self.video_playback.is_muted();
                        let loop_enabled = self.video_playback.is_loop_enabled();

                        // Store the command sender in the player for pause/play/seek
                        if let Some(player) = self.video_playback.player_mut() {
                            player.set_command_sender(command_sender);

                            // Apply current volume, mute, and loop state
                            player.set_volume(Volume::new(volume));
                            player.set_muted(muted);
                            player.set_loop(loop_enabled);

                            // Load the first frame immediately so capture and step work
                            // without requiring play+pause first.
                            // This seeks to 0 and decodes the first frame without starting playback.
                            if matches!(player.state(), crate::video_player::PlaybackState::Stopped)
                            {
                                player.seek(0.0);
                            }

                            // Auto-play if enabled
                            if autoplay {
                                player.play();
                            }
                        }
                    }
                    PlaybackMessage::FrameReady {
                        rgba_data,
                        width,
                        height,
                        pts_secs,
                    } => {
                        // Update canvas with new frame
                        // The shader only stores the frame data - display size is calculated
                        // by the pane at render time based on current zoom state
                        self.video_shader.set_frame(rgba_data, width, height);

                        // Update zoom display for fit-to-window mode
                        // This keeps the zoom textbox in sync, but doesn't affect the shader
                        // (pane calculates display size from zoom at render time)
                        if self.video_playback.fit_to_window() {
                            if let Some(fit_zoom) = self.compute_fit_zoom_percent() {
                                self.image_transform.zoom.update_zoom_display(fit_zoom);
                            }
                        }

                        // Update player position
                        if let Some(player) = self.video_playback.player_mut() {
                            player.update_position(pts_secs);
                        }

                        // Clear seek preview if we received a frame near the seek target
                        // This ensures the slider stays at the new position after seek completes
                        self.video_playback.maybe_clear_seek_preview(pts_secs);
                    }
                    PlaybackMessage::Buffering => {
                        // Update player to buffering state, but not if we're seeking
                        // (Seeking state needs to be preserved to know whether to resume playing)
                        if let Some(player) = self.video_playback.player_mut() {
                            if !matches!(
                                player.state(),
                                crate::video_player::PlaybackState::Seeking { .. }
                            ) {
                                let position = player.state().position().unwrap_or(0.0);
                                player.set_buffering(position);
                            }
                        }
                    }
                    PlaybackMessage::EndOfStream => {
                        // Handle end of stream via cluster
                        self.video_playback.handle_end_of_stream();
                    }
                    PlaybackMessage::Error(msg) => {
                        // Parse error message into typed VideoError for i18n support
                        let video_error = VideoError::from_message(&msg);

                        // Store error in player state for display
                        if let Some(player) = self.video_playback.player_mut() {
                            player.set_error(msg);
                        }

                        // Return notification effect
                        return (
                            Effect::ShowErrorNotification {
                                key: video_error.i18n_key(),
                                args: video_error.i18n_args(),
                            },
                            Task::none(),
                        );
                    }
                    PlaybackMessage::AudioPts(pts_secs) => {
                        // Update sync clock with audio PTS for A/V synchronization
                        if let Some(player) = self.video_playback.player() {
                            player.update_audio_pts(pts_secs);
                        }
                    }
                    PlaybackMessage::HistoryExhausted => {
                        // Frame history buffer is exhausted - reset history position
                        // so the step backward button gets disabled
                        if let Some(player) = self.video_playback.player_mut() {
                            player.reset_history_position();
                        }
                    }
                }

                (Effect::None, Task::none())
            }
            Message::FilterDropdown(msg) => {
                use filter_dropdown::Message as FdMsg;
                match msg {
                    FdMsg::ToggleDropdown => {
                        // Handle locally - toggle dropdown open/close
                        self.filter_dropdown.toggle();
                        (Effect::None, Task::none())
                    }
                    FdMsg::CloseDropdown => {
                        // Handle locally - close dropdown (e.g., click outside)
                        self.filter_dropdown.close();
                        (Effect::None, Task::none())
                    }
                    FdMsg::ConsumeClick => {
                        // No-op - just consume the click to prevent propagation
                        (Effect::None, Task::none())
                    }
                    FdMsg::DateSegmentChanged {
                        target,
                        segment,
                        value,
                    } => {
                        // Handle locally - update segment value
                        let _ = self.filter_dropdown.set_segment(target, segment, &value);

                        // Check if the complete date is now valid and submit automatically
                        let date_state = self.filter_dropdown.date_state(target);
                        if date_state.is_complete_and_valid() {
                            return (
                                Effect::FilterChanged(FdMsg::DateSubmit(target)),
                                Task::none(),
                            );
                        }

                        (Effect::None, Task::none())
                    }
                    FdMsg::DateSubmit(target) => {
                        // Forward to app if valid, otherwise just update locally
                        let date_state = self.filter_dropdown.date_state(target);
                        if date_state.is_complete_and_valid() {
                            (
                                Effect::FilterChanged(FdMsg::DateSubmit(target)),
                                Task::none(),
                            )
                        } else {
                            (Effect::None, Task::none())
                        }
                    }
                    FdMsg::ClearDate(target) => {
                        // Clear local input and forward to app
                        self.filter_dropdown.clear_date(target);
                        (
                            Effect::FilterChanged(FdMsg::ClearDate(target)),
                            Task::none(),
                        )
                    }
                    // Forward other filter changes to app (it owns the navigator/filter)
                    other => (Effect::FilterChanged(other), Task::none()),
                }
            }
        }
    }

    #[allow(clippy::too_many_lines)] // Complex UI view with many contextual elements
    #[allow(clippy::needless_pass_by_value)] // ViewEnv is small (references only)
    pub fn view<'a>(&'a self, env: ViewEnv<'a>) -> Element<'a, Message> {
        let geometry_state = self.geometry_state();

        let error = self.error().map(|error| viewer::ErrorContext {
            friendly_text: error.friendly_text(),
            details: error.details(),
            show_details: error.show_details(),
        });

        let position_line = geometry_state
            .scroll_position_percentage()
            .map(|(px, py)| format_position_indicator(env.i18n, px, py));

        let zoom_line = if (self.image_transform.zoom.zoom_percent
            - crate::ui::state::zoom::DEFAULT_ZOOM_PERCENT)
            .abs()
            > f32::EPSILON
        {
            Some(format_zoom_indicator(
                env.i18n,
                self.image_transform.zoom.zoom_percent,
            ))
        } else {
            None
        };

        let rotation_line = if self.current_rotation().is_rotated() {
            Some(format_rotation_indicator(self.current_rotation()))
        } else {
            None
        };

        let media_type_line = self
            .media()
            .and_then(|m| format_media_indicator(env.i18n, m));

        let hud_lines = position_line
            .into_iter()
            .chain(zoom_line)
            .chain(rotation_line)
            .chain(media_type_line)
            .collect::<Vec<HudLine>>();

        // In fullscreen, overlay auto-hides after delay
        // In windowed mode, controls stay visible but center overlay (pause button) can hide
        let overlay_should_be_visible = self.overlay.should_show_controls(env.overlay_hide_delay);

        // For center video overlay (play/pause button), use auto-hide in both modes when playing
        let is_currently_playing = self.video_playback.has_player()
            && matches!(
                self.video_playback.player().map(VideoPlayer::state),
                Some(
                    crate::video_player::PlaybackState::Playing { .. }
                        | crate::video_player::PlaybackState::Buffering { .. }
                )
            );

        let center_overlay_visible = if is_currently_playing {
            // When playing, center overlay (pause button) auto-hides after delay
            overlay_should_be_visible
        } else {
            // When paused/stopped, play button always visible
            true
        };

        let effective_fit_to_window = self.fit_to_window();
        let image = self.media().map(|image_data| viewer::ImageContext {
            i18n: env.i18n,
            controls_context: controls::ViewContext {
                i18n: env.i18n,
                metadata_editor_has_changes: env.metadata_editor_has_changes,
                is_video: self.is_video(),
            },
            zoom: &self.image_transform.zoom,
            effective_fit_to_window,
            pane_context: pane::ViewContext {
                background_theme: env.background_theme,
                hud_lines,
                scrollable_id: SCROLLABLE_ID,
                i18n: env.i18n,
            },
            pane_model: pane::ViewModel {
                media: image_data,
                zoom_percent: self.image_transform.zoom.zoom_percent,
                manual_zoom_percent: self.image_transform.zoom.zoom_percent,
                fit_to_window: effective_fit_to_window,
                is_dragging: self.image_transform.is_dragging(),
                cursor_over_media: geometry_state.is_cursor_over_media(),
                arrows_visible: if env.is_fullscreen {
                    // In fullscreen, arrows use same auto-hide logic as controls
                    self.overlay.arrows_visible
                        && env.navigation.total_count > 0
                        && overlay_should_be_visible
                } else {
                    // In windowed mode, arrows visible on hover (current behavior)
                    self.overlay.arrows_visible && env.navigation.total_count > 0
                },
                overlay_visible: center_overlay_visible,
                has_next: env.navigation.has_next,
                has_previous: env.navigation.has_previous,
                at_first: env.navigation.at_first,
                at_last: env.navigation.at_last,
                current_index: env.navigation.current_index,
                total_count: env.navigation.total_count,
                position_counter_visible: if env.is_fullscreen {
                    // In fullscreen, use same auto-hide logic as arrows and controls
                    env.navigation.total_count > 0 && overlay_should_be_visible
                } else {
                    // In windowed mode, always visible
                    true
                },
                hud_visible: if env.is_fullscreen {
                    // In fullscreen, auto-hide HUD with other overlay elements
                    overlay_should_be_visible
                } else {
                    // In windowed mode, always visible
                    true
                },
                video_shader: Some(&self.video_shader),
                // Use is_playing_or_will_resume() to include Seeking state
                // This prevents the play button from flashing during seek operations
                is_video_playing: self.is_video_playing_or_will_resume(),
                is_loading_media: self.media_lifecycle.is_loading(),
                spinner_rotation: self.media_lifecycle.spinner_rotation(),
                video_error: self
                    .video_playback
                    .player()
                    .and_then(|p| p.state().error_message()),
                metadata_editor_has_changes: env.metadata_editor_has_changes,
                rotation: self.current_rotation(),
                rotated_image_cache: self.rotated_image_cache(),
            },
            controls_visible: if env.is_fullscreen {
                // In fullscreen, auto-hide controls after configured delay
                overlay_should_be_visible
            } else {
                // In windowed mode, always show controls
                true
            },
            is_fullscreen: env.is_fullscreen,
            is_video: self.is_video(),
            video_playback_state: self.media().and_then(|media| {
                // Build PlaybackState for video controls
                // Show controls for any video, not just when VideoPlayer exists
                if let MediaData::Video(ref video_data) = media {
                    let (
                        is_playing,
                        position_secs,
                        loop_enabled,
                        can_step_backward,
                        can_step_forward,
                        playback_speed,
                        speed_auto_muted,
                    ) = if let Some(player) = self.video_playback.player() {
                        let state = player.state();
                        let can_step_back = player.can_step_backward();
                        let can_step_fwd = player.can_step_forward();
                        let speed = player.playback_speed();
                        let auto_muted = player.is_speed_auto_muted();
                        match state {
                            crate::video_player::PlaybackState::Playing { position_secs }
                            | crate::video_player::PlaybackState::Buffering { position_secs } => (
                                true,
                                *position_secs,
                                self.video_playback.is_loop_enabled(),
                                false,
                                false,
                                speed,
                                auto_muted,
                            ),
                            crate::video_player::PlaybackState::Paused { position_secs } => (
                                false,
                                *position_secs,
                                self.video_playback.is_loop_enabled(),
                                can_step_back,
                                can_step_fwd,
                                speed,
                                auto_muted,
                            ),
                            _ => (false, 0.0, self.video_playback.is_loop_enabled(), false, false, 1.0, false),
                        }
                    } else {
                        // No player yet - show initial state (paused at 0)
                        (false, 0.0, false, false, false, 1.0, false)
                    };

                    Some(video_controls::PlaybackState {
                        is_playing,
                        position_secs,
                        duration_secs: video_data.duration_secs,
                        volume: self.video_playback.volume(),
                        muted: self.video_playback.is_muted(),
                        loop_enabled,
                        seek_preview_position: self.video_playback.seek_preview_position(),
                        overflow_menu_open: self.video_playback.is_overflow_menu_open(),
                        can_step_backward,
                        can_step_forward,
                        playback_speed,
                        speed_auto_muted,
                        has_audio: video_data.has_audio,
                    })
                } else {
                    None
                }
            }),
        });

        viewer::view(viewer::ViewContext {
            i18n: env.i18n,
            error,
            image,
            is_loading: self.media_lifecycle.is_loading(),
            spinner_rotation: self.media_lifecycle.spinner_rotation(),
        })
    }

    fn handle_controls(&mut self, message: controls::Message) -> (Effect, Task<Message>) {
        #[allow(clippy::enum_glob_use)] // Match ergonomics for many Message variants
        use controls::Message::*;

        match message {
            ZoomInputChanged(value) => {
                self.image_transform.zoom.zoom_input = value;
                self.image_transform.zoom.zoom_input_dirty = true;
                self.image_transform.zoom.zoom_input_error_key = None;
                (Effect::None, Task::none())
            }
            ZoomInputSubmitted => {
                self.image_transform.zoom.zoom_input_dirty = false;

                if let Some(value) = parse_number(&self.image_transform.zoom.zoom_input) {
                    self.image_transform.zoom.apply_manual_zoom(value);
                    // Also disable video fit-to-window when manually setting zoom
                    if self.is_video() {
                        self.video_playback
                            .handle(clusters::video_playback::Message::SetFitToWindow(false));
                    }
                    (Effect::PersistPreferences, Task::none())
                } else {
                    self.image_transform.zoom.zoom_input_error_key =
                        Some(crate::ui::state::zoom::ZOOM_INPUT_INVALID_KEY);
                    (Effect::None, Task::none())
                }
            }
            ResetZoom => {
                self.image_transform
                    .zoom
                    .apply_manual_zoom(crate::ui::state::zoom::DEFAULT_ZOOM_PERCENT);
                // Also disable video fit-to-window when resetting zoom
                if self.is_video() {
                    self.video_playback
                            .handle(clusters::video_playback::Message::SetFitToWindow(false));
                }
                (Effect::PersistPreferences, Task::none())
            }
            ZoomIn => {
                let new_zoom = self.image_transform.zoom.zoom_percent
                    + self.image_transform.zoom.zoom_step.value();
                self.image_transform.zoom.apply_manual_zoom(new_zoom);
                // Also disable video fit-to-window when zooming on a video
                if self.is_video() {
                    self.video_playback
                            .handle(clusters::video_playback::Message::SetFitToWindow(false));
                }
                (Effect::PersistPreferences, Task::none())
            }
            ZoomOut => {
                let new_zoom = self.image_transform.zoom.zoom_percent
                    - self.image_transform.zoom.zoom_step.value();
                self.image_transform.zoom.apply_manual_zoom(new_zoom);
                // Also disable video fit-to-window when zooming on a video
                if self.is_video() {
                    self.video_playback
                            .handle(clusters::video_playback::Message::SetFitToWindow(false));
                }
                (Effect::PersistPreferences, Task::none())
            }
            SetFitToWindow(fit) => {
                // For videos, use video_fit_to_window (not persisted)
                // For images, use zoom.fit_to_window (persisted)
                let is_video = self.is_video();

                if fit {
                    self.enable_fit_to_window();
                    self.refresh_fit_zoom();
                } else {
                    self.disable_fit_to_window();
                }

                // Only persist preferences for images, not videos
                let effect = if is_video {
                    Effect::None
                } else {
                    Effect::PersistPreferences
                };
                (effect, Task::none())
            }
            ToggleFullscreen => {
                // Clear overlay timer and position when entering fullscreen to hide controls
                self.overlay
                    .handle(subcomponents::overlay::Message::EnteredFullscreen);
                (Effect::ToggleFullscreen, Task::none())
            }
            DeleteCurrentImage => (Effect::None, Task::none()),
            RotateClockwise => {
                // Rotation only applies to images, not videos
                if self.is_current_media_image() {
                    let effect = self
                        .image_transform
                        .handle(clusters::image_transform::Message::RotateClockwise);
                    if matches!(effect, clusters::image_transform::Effect::RotationChanged) {
                        self.handle_rotation_changed();
                    }
                }
                (Effect::None, Task::none())
            }
            RotateCounterClockwise => {
                // Rotation only applies to images, not videos
                if self.is_current_media_image() {
                    let effect = self
                        .image_transform
                        .handle(clusters::image_transform::Message::RotateCounterClockwise);
                    if matches!(effect, clusters::image_transform::Effect::RotationChanged) {
                        self.handle_rotation_changed();
                    }
                }
                (Effect::None, Task::none())
            }
        }
    }

    #[allow(clippy::too_many_lines)] // Event handler for multiple event types
    fn handle_raw_event(&mut self, event: event::Event) -> (Effect, Task<Message>) {
        match event {
            event::Event::Window(window_event) => {
                if let window::Event::Resized(size) = window_event {
                    let bounds = Rectangle::new(Point::new(0.0, 0.0), size);
                    self.viewport.update(bounds, self.viewport.offset);
                    self.refresh_fit_zoom();
                }
                (Effect::None, Task::none())
            }
            event::Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::WheelScrolled { delta } => {
                    let effect = if self.handle_wheel_zoom(delta) {
                        Effect::PersistPreferences
                    } else {
                        Effect::None
                    };
                    (effect, Task::none())
                }
                mouse::Event::ButtonPressed(button) => {
                    let effect = if let Some(position) = self.cursor_position() {
                        self.handle_mouse_button_pressed(button, position)
                    } else {
                        Effect::None
                    };
                    (effect, Task::none())
                }
                mouse::Event::ButtonReleased(button) => {
                    self.handle_mouse_button_released(button);
                    (Effect::None, Task::none())
                }
                mouse::Event::CursorMoved { position } => {
                    self.image_transform.drag.cursor_position = Some(position);

                    // Delegate overlay visibility logic to sub-component
                    // (filters micro-movements, handles fullscreen entry delay, etc.)
                    self.overlay.handle(subcomponents::overlay::Message::MouseMoved(
                        Point::new(position.x, position.y),
                    ));

                    if self.image_transform.is_dragging() {
                        let task = self.handle_cursor_moved_during_drag(position);
                        (Effect::None, task)
                    } else {
                        (Effect::None, Task::none())
                    }
                }
                mouse::Event::CursorLeft => {
                    self.image_transform.drag.cursor_position = None;
                    self.overlay.cursor_left();
                    if self.image_transform.is_dragging() {
                        self.image_transform.drag.inner.stop();
                    }
                    (Effect::None, Task::none())
                }
                mouse::Event::CursorEntered => (Effect::None, Task::none()),
            },
            event::Event::Keyboard(keyboard_event) => match keyboard_event {
                keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::F11),
                    ..
                } => {
                    // Clear overlay timer and position when entering fullscreen to hide controls
                    self.overlay
                        .handle(subcomponents::overlay::Message::EnteredFullscreen);
                    (Effect::ToggleFullscreen, Task::none())
                }
                keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::Escape),
                    ..
                } => (Effect::ExitFullscreen, Task::none()),
                keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::Space),
                    ..
                } => {
                    // Space: Toggle play/pause (video only)
                    if self.has_active_video_session() {
                        self.dispatch_message(Message::VideoControls(
                            video_controls::Message::TogglePlayback,
                        ))
                    } else if matches!(self.media(), Some(MediaData::Video(_))) {
                        // Video loaded but not playing yet - initiate playback
                        self.dispatch_message(Message::InitiatePlayback)
                    } else {
                        (Effect::None, Task::none())
                    }
                }
                keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::ArrowRight),
                    ..
                } => {
                    // ArrowRight: Seek forward if video is playing, otherwise navigate to next media
                    // Uses is_playing_or_will_resume() to handle rapid key repeats during seek
                    if self.is_video_playing_or_will_resume() {
                        let step = self.video_playback.keyboard_seek_step().value();
                        self.dispatch_message(Message::VideoControls(
                            video_controls::Message::SeekRelative(step),
                        ))
                    } else {
                        self.dispatch_message(Message::NavigateNext)
                    }
                }
                keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::ArrowLeft),
                    ..
                } => {
                    // ArrowLeft: Seek backward if video is playing, otherwise navigate to previous media
                    // Uses is_playing_or_will_resume() to handle rapid key repeats during seek
                    if self.is_video_playing_or_will_resume() {
                        let step = self.video_playback.keyboard_seek_step().value();
                        self.dispatch_message(Message::VideoControls(
                            video_controls::Message::SeekRelative(-step),
                        ))
                    } else {
                        self.dispatch_message(Message::NavigatePrevious)
                    }
                }
                keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::ArrowUp),
                    ..
                } => {
                    // ArrowUp: Increase volume (only during video playback)
                    if self.has_active_video_session() {
                        let new_volume = Volume::new(self.video_playback.volume()).increase();
                        self.dispatch_message(Message::VideoControls(
                            video_controls::Message::SetVolume(new_volume),
                        ))
                    } else {
                        (Effect::None, Task::none())
                    }
                }
                keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::ArrowDown),
                    ..
                } => {
                    // ArrowDown: Decrease volume (only during video playback)
                    if self.has_active_video_session() {
                        let new_volume = Volume::new(self.video_playback.volume()).decrease();
                        self.dispatch_message(Message::VideoControls(
                            video_controls::Message::SetVolume(new_volume),
                        ))
                    } else {
                        (Effect::None, Task::none())
                    }
                }
                keyboard::Event::KeyPressed {
                    key: keyboard::Key::Character(ref c),
                    modifiers,
                    ..
                } if (c.as_str() == "m" || c.as_str() == "M")
                    && !modifiers.command()
                    && !modifiers.alt() =>
                {
                    // M key: Toggle mute (only during video playback)
                    if self.has_active_video_session() {
                        self.dispatch_message(Message::VideoControls(
                            video_controls::Message::ToggleMute,
                        ))
                    } else {
                        (Effect::None, Task::none())
                    }
                }
                keyboard::Event::KeyPressed {
                    key: keyboard::Key::Character(ref c),
                    modifiers,
                    ..
                } if c.as_str() == "e"
                    && !modifiers.command()
                    && !modifiers.alt()
                    && !modifiers.shift() =>
                {
                    // E key: Enter edit mode (only if image is loaded and not a video)
                    // Video editing is not supported in v0.2
                    if self.current_media_path.is_some() && !self.is_video() {
                        (Effect::EnterEditor, Task::none())
                    } else {
                        (Effect::None, Task::none())
                    }
                }
                keyboard::Event::KeyPressed {
                    key: keyboard::Key::Character(ref c),
                    modifiers,
                    ..
                } if c.as_str() == ","
                    && !modifiers.command()
                    && !modifiers.alt()
                    && !modifiers.shift() =>
                {
                    // Comma key: Step backward one frame (only when video is paused)
                    // Route through VideoControls handler for consistent behavior
                    if self.video_playback.has_player() {
                        self.dispatch_message(Message::VideoControls(
                            video_controls::Message::StepBackward,
                        ))
                    } else {
                        (Effect::None, Task::none())
                    }
                }
                keyboard::Event::KeyPressed {
                    key: keyboard::Key::Character(ref c),
                    modifiers,
                    ..
                } if c.as_str() == "."
                    && !modifiers.command()
                    && !modifiers.alt()
                    && !modifiers.shift() =>
                {
                    // Period key: Step forward one frame (only when video is paused)
                    // Route through VideoControls handler for consistent behavior
                    if self.video_playback.has_player() {
                        self.dispatch_message(Message::VideoControls(
                            video_controls::Message::StepForward,
                        ))
                    } else {
                        (Effect::None, Task::none())
                    }
                }
                keyboard::Event::KeyPressed {
                    key: keyboard::Key::Character(ref c),
                    modifiers,
                    ..
                } if (c.as_str() == "j" || c.as_str() == "J")
                    && !modifiers.command()
                    && !modifiers.alt() =>
                {
                    // J key: Decrease playback speed (YouTube/VLC style)
                    if self.video_playback.has_player() {
                        self.dispatch_message(Message::VideoControls(
                            video_controls::Message::DecreasePlaybackSpeed,
                        ))
                    } else {
                        (Effect::None, Task::none())
                    }
                }
                keyboard::Event::KeyPressed {
                    key: keyboard::Key::Character(ref c),
                    modifiers,
                    ..
                } if (c.as_str() == "l" || c.as_str() == "L")
                    && !modifiers.command()
                    && !modifiers.alt() =>
                {
                    // L key: Increase playback speed (YouTube/VLC style)
                    if self.video_playback.has_player() {
                        self.dispatch_message(Message::VideoControls(
                            video_controls::Message::IncreasePlaybackSpeed,
                        ))
                    } else {
                        (Effect::None, Task::none())
                    }
                }
                keyboard::Event::KeyPressed {
                    key: keyboard::Key::Character(ref c),
                    modifiers,
                    ..
                } if (c.as_str() == "i" || c.as_str() == "I")
                    && !modifiers.command()
                    && !modifiers.alt() =>
                {
                    // I key: Toggle info/metadata panel
                    (Effect::ToggleInfoPanel, Task::none())
                }
                keyboard::Event::KeyPressed {
                    key: keyboard::Key::Character(ref c),
                    modifiers,
                    ..
                } if (c.as_str() == "r" || c.as_str() == "R")
                    && !modifiers.command()
                    && !modifiers.alt() =>
                {
                    // R key: Rotate clockwise
                    // Shift+R: Rotate counter-clockwise
                    if modifiers.shift() {
                        self.dispatch_message(Message::RotateCounterClockwise)
                    } else {
                        self.dispatch_message(Message::RotateClockwise)
                    }
                }
                keyboard::Event::ModifiersChanged(modifiers) => {
                    if modifiers.command() {
                        // no-op currently, but keep placeholder for shortcut support
                    }
                    (Effect::None, Task::none())
                }
                _ => (Effect::None, Task::none()),
            },
            _ => (Effect::None, Task::none()),
        }
    }

    fn handle_mouse_button_pressed(&mut self, button: mouse::Button, position: Point) -> Effect {
        if button == mouse::Button::Left {
            // Delegate click handling (including double-click detection) to image_transform cluster
            let click_effect = self
                .image_transform
                .handle(clusters::image_transform::Message::Click(position));

            // Reset overlay timer on any left click, even on UI controls
            // This keeps controls visible when user is interacting
            self.overlay
                .handle(subcomponents::overlay::Message::OverlayInteraction);

            if self.geometry_state().is_cursor_over_media() {
                if matches!(click_effect, clusters::image_transform::Effect::DoubleClick) {
                    // Clear overlay timer when entering fullscreen (will hide controls initially)
                    self.overlay
                        .handle(subcomponents::overlay::Message::EnteredFullscreen);
                    return Effect::ToggleFullscreen;
                }

                self.image_transform.drag.inner.start(position, self.viewport.offset);
            }
        }

        Effect::None
    }

    fn handle_mouse_button_released(&mut self, button: mouse::Button) {
        if button == mouse::Button::Left {
            self.image_transform.drag.inner.stop();
        }
    }

    /// Updates the viewport when the user drags the image. Clamps the offset to
    /// the scaled image bounds and mirrors the change to the scrollable widget
    /// so keyboard/scroll interactions stay in sync.
    fn handle_cursor_moved_during_drag(&mut self, position: Point) -> Task<Message> {
        let Some(proposed_offset) = self.image_transform.drag.inner.calculate_offset(position)
        else {
            return Task::none();
        };

        let geometry_state = self.geometry_state();
        // Use rotation-aware size for correct clamping when image is rotated
        if let (Some(viewport), Some(size)) = (
            self.viewport.bounds,
            geometry_state.scaled_media_size_rotated(self.current_rotation()),
        ) {
            let max_offset_x = (size.width - viewport.width).max(0.0);
            let max_offset_y = (size.height - viewport.height).max(0.0);

            let clamped_offset = AbsoluteOffset {
                x: if max_offset_x > 0.0 {
                    proposed_offset.x.clamp(0.0, max_offset_x)
                } else {
                    0.0
                },
                y: if max_offset_y > 0.0 {
                    proposed_offset.y.clamp(0.0, max_offset_y)
                } else {
                    0.0
                },
            };

            self.viewport.offset = clamped_offset;

            let relative_x = if max_offset_x > 0.0 {
                clamped_offset.x / max_offset_x
            } else {
                0.0
            };

            let relative_y = if max_offset_y > 0.0 {
                clamped_offset.y / max_offset_y
            } else {
                0.0
            };

            operation::snap_to(
                Id::new(SCROLLABLE_ID),
                RelativeOffset {
                    x: relative_x,
                    y: relative_y,
                },
            )
        } else {
            self.viewport.offset = proposed_offset;
            Task::none()
        }
    }

    /// Applies wheel-based zoom while the cursor is over the image, returning a
    /// boolean so callers can decide whether to stop event propagation.
    fn handle_wheel_zoom(&mut self, delta: mouse::ScrollDelta) -> bool {
        if !self.geometry_state().is_cursor_over_media() {
            return false;
        }

        let steps = scroll_steps(&delta);
        if steps.abs() < f32::EPSILON {
            return false;
        }

        let new_zoom = self.image_transform.zoom.zoom_percent
            + steps * self.image_transform.zoom.zoom_step.value();
        self.image_transform.zoom.apply_manual_zoom(new_zoom);

        // Also disable video fit-to-window when zooming on a video
        if self.is_video() {
            self.video_playback
                .handle(clusters::video_playback::Message::SetFitToWindow(false));
        }
        // No need to sync shader scale - pane calculates display size from zoom at render time

        true
    }

    /// Recomputes the fit-to-window zoom when layout-affecting events occur so
    /// the zoom textbox always mirrors the actual fit percentage.
    ///
    /// Note: This only updates the zoom display state. The actual display size
    /// is calculated by the pane at render time based on the zoom state.
    fn refresh_fit_zoom(&mut self) {
        // Use effective fit_to_window (considers video vs image)
        let effective_fit_to_window = self.fit_to_window();
        if effective_fit_to_window {
            if let Some(fit_zoom) = self.compute_fit_zoom_percent() {
                self.image_transform.zoom.update_zoom_display(fit_zoom);
                self.image_transform.zoom.zoom_input_dirty = false;
                self.image_transform.zoom.zoom_input_error_key = None;
                // No need to sync shader scale - pane calculates display size at render time
            }
        }
    }

    /// Calculates the zoom percentage needed to fit the current image inside
    /// the viewport. Returns `None` until viewport bounds are known.
    #[allow(clippy::cast_precision_loss)] // u32 to f32 for image dimensions is acceptable
    pub fn compute_fit_zoom_percent(&self) -> Option<f32> {
        let media = self.media()?;
        let viewport = self.viewport.bounds?;

        if media.width() == 0 || media.height() == 0 {
            return Some(crate::ui::state::zoom::DEFAULT_ZOOM_PERCENT);
        }

        if viewport.width <= 0.0 || viewport.height <= 0.0 {
            return None;
        }

        let media_width = media.width() as f32;
        let media_height = media.height() as f32;

        let scale_x = viewport.width / media_width;
        let scale_y = viewport.height / media_height;

        let scale = scale_x.min(scale_y);

        if !scale.is_finite() || scale <= 0.0 {
            return Some(crate::ui::state::zoom::DEFAULT_ZOOM_PERCENT);
        }

        Some(crate::ui::state::zoom::clamp_zoom(scale * 100.0))
    }

    /// Provides a lightweight view of geometry-dependent state for hit-testing
    /// and layout helpers.
    fn geometry_state(&self) -> geometry::ViewerState<'_> {
        geometry::ViewerState::new(
            self.media(),
            &self.viewport,
            self.image_transform.zoom.zoom_percent,
            self.cursor_position(),
        )
    }
}

fn parse_number(input: &str) -> Option<f32> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    let without_percent = trimmed.trim_end_matches('%').trim();
    if without_percent.is_empty() {
        return None;
    }

    let normalized = without_percent.replace(',', ".");
    let candidate = normalized.trim();
    let value = candidate.parse::<f32>().ok()?;

    if !value.is_finite() {
        return None;
    }

    Some(value)
}

/// Normalizes mouse wheel units (lines vs. pixels) into our abstract step
/// values so zooming feels consistent across platforms.
fn scroll_steps(delta: &mouse::ScrollDelta) -> f32 {
    match delta {
        mouse::ScrollDelta::Lines { y, .. } => *y,
        mouse::ScrollDelta::Pixels { y, .. } => *y / 120.0,
    }
}

fn format_position_indicator(_i18n: &I18n, px: f32, py: f32) -> HudLine {
    HudLine {
        icon: HudIconKind::Position,
        text: format!("{px:.0}% x {py:.0}%"),
    }
}

fn format_zoom_indicator(_i18n: &I18n, zoom_percent: f32) -> HudLine {
    HudLine {
        icon: HudIconKind::Zoom,
        text: format!("{zoom_percent:.0}%"),
    }
}

fn format_rotation_indicator(rotation: RotationAngle) -> HudLine {
    HudLine {
        icon: HudIconKind::Rotation,
        text: format!("{}°", rotation.degrees()),
    }
}

/// Generates HUD indicator for videos without audio.
///
/// Only shows an indicator when a video has no audio track.
/// Returns None for images and videos with audio to avoid cluttering the UI.
fn format_media_indicator(i18n: &I18n, media: &MediaData) -> Option<HudLine> {
    match media {
        MediaData::Video(video_data) => {
            if video_data.has_audio {
                None // No indicator needed for videos with audio
            } else {
                Some(HudLine {
                    icon: HudIconKind::Video { has_audio: false },
                    text: i18n.tr("hud-video-no-audio"),
                })
            }
        }
        MediaData::Image(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// Creates a test diagnostics handle for use in tests.
    fn test_diagnostics() -> crate::diagnostics::DiagnosticsHandle {
        use crate::diagnostics::{BufferCapacity, DiagnosticsCollector};
        DiagnosticsCollector::new(BufferCapacity::default()).handle()
    }

    #[test]
    fn scroll_indicator_formats_hud_lines() {
        let i18n = I18n::default();
        let position = format_position_indicator(&i18n, 12.4, 56.7);
        let zoom = format_zoom_indicator(&i18n, 135.2);

        assert!(matches!(position.icon, HudIconKind::Position));
        assert!(position.text.contains("12%"));
        assert!(position.text.contains("57%"));

        assert!(matches!(zoom.icon, HudIconKind::Zoom));
        assert!(zoom.text.contains("135%"));
    }

    #[test]
    fn format_media_indicator_shows_no_audio_for_silent_video() {
        use crate::media::{ImageData, VideoData};

        let i18n = I18n::default();
        let pixels = vec![255_u8; 4];
        let thumbnail = ImageData::from_rgba(1, 1, pixels);

        let video_data = VideoData {
            thumbnail,
            width: 1920,
            height: 1080,
            duration_secs: 125.0,
            fps: 30.0,
            has_audio: false,
        };

        let media = MediaData::Video(video_data);
        let indicator = format_media_indicator(&i18n, &media);

        let hud = indicator.expect("expected HUD line for video without audio");
        assert!(matches!(hud.icon, HudIconKind::Video { has_audio: false }));
    }

    #[test]
    fn format_media_indicator_returns_none_for_video_with_audio() {
        use crate::media::{ImageData, VideoData};

        let i18n = I18n::default();
        let pixels = vec![255_u8; 4];
        let thumbnail = ImageData::from_rgba(1, 1, pixels);

        let video_data = VideoData {
            thumbnail,
            width: 1920,
            height: 1080,
            duration_secs: 65.0,
            fps: 30.0,
            has_audio: true,
        };

        let media = MediaData::Video(video_data);
        let indicator = format_media_indicator(&i18n, &media);

        assert!(
            indicator.is_none(),
            "should not show indicator for video with audio"
        );
    }

    #[test]
    fn loading_state_resets_on_successful_load() {
        use crate::media::ImageData;

        let i18n = I18n::default();
        let mut state = State::new();

        // Simulate loading state via start_loading()
        state.current_media_path = Some(std::path::PathBuf::from("/test/image.jpg"));
        state.start_loading();
        assert!(state.is_loading_media(), "loading should be active");

        // Simulate successful load (MediaLoaded with Ok result)
        let pixels = vec![255_u8; 100 * 100 * 4];
        let image_data = ImageData::from_rgba(100, 100, pixels);

        let (_effect, _task) = state.handle_message(
            Message::MediaLoaded(Ok(MediaData::Image(image_data))),
            &i18n,
            &test_diagnostics(),
        );

        assert!(
            !state.is_loading_media(),
            "loading flag should be cleared after successful load"
        );
        assert!(state.error().is_none(), "no error should be set");
    }

    #[test]
    fn format_media_indicator_returns_none_for_images() {
        use crate::media::ImageData;

        let i18n = I18n::default();
        let pixels = vec![255_u8; 100 * 100 * 4];
        let image_data = ImageData::from_rgba(100, 100, pixels);

        let media = MediaData::Image(image_data);
        let indicator = format_media_indicator(&i18n, &media);
        assert!(indicator.is_none());
    }

    // NOTE: Detailed overlay visibility tests have been moved to the overlay sub-component
    // (src/ui/viewer/subcomponents/overlay.rs). The tests below verify integration behavior.

    #[test]
    fn overlay_fullscreen_toggle_via_controls() {
        let mut state = State::new();
        assert!(!state.overlay.is_fullscreen());

        // Toggle fullscreen (via button)
        let (effect, _) = state.handle_controls(controls::Message::ToggleFullscreen);

        assert_eq!(effect, Effect::ToggleFullscreen);
        assert!(state.overlay.is_fullscreen());
        // EnteredFullscreen clears the interaction timer initially
        assert!(!state.overlay.should_show_controls(Duration::from_secs(3)));
    }

    #[test]
    fn arrows_visible_after_mouse_movement() {
        let mut state = State::new();

        // Simulate mouse movement to show arrows
        let event = event::Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(100.0, 100.0),
        });
        let (_effect, _task) = state.handle_raw_event(event);

        // In windowed mode, arrows should be visible
        assert!(
            state.overlay.arrows_visible,
            "Arrows should be visible after mouse movement"
        );
    }

    #[test]
    fn keyboard_navigation_always_works() {
        let mut state = State::new();

        // Enter fullscreen with arrows hidden
        state
            .overlay
            .handle(subcomponents::overlay::Message::EnteredFullscreen);
        state.overlay.cursor_left(); // Hide arrows

        // Keyboard navigation should still work
        let (effect, _) =
            state.handle_message(Message::NavigateNext, &I18n::default(), &test_diagnostics());

        assert_eq!(
            effect,
            Effect::NavigateNext,
            "Keyboard navigation should work even when arrows are hidden"
        );
    }

    #[test]
    fn play_button_interaction_records_overlay_interaction() {
        let mut state = State::new();

        // Initially no controls visible in fullscreen
        state
            .overlay
            .handle(subcomponents::overlay::Message::EnteredFullscreen);
        assert!(!state.overlay.should_show_controls(Duration::from_secs(3)));

        // Send a playback message
        let (effect, _) = state.handle_message(
            Message::InitiatePlayback,
            &I18n::default(),
            &test_diagnostics(),
        );

        // Effect should be None, and controls should now be visible
        assert_eq!(effect, Effect::None);
        assert!(
            state.overlay.should_show_controls(Duration::from_secs(3)),
            "Controls should be visible after initiating playback"
        );
    }
}
