// SPDX-License-Identifier: MPL-2.0
//! Viewer component encapsulating state and update logic.

use crate::directory_scanner::ImageList;
use crate::error::Error;
use crate::i18n::fluent::I18n;
use crate::media::MediaData;
use crate::ui::state::{DragState, ViewportState, ZoomState};
use crate::ui::viewer::{
    self, controls, pane, state as geometry, video_controls, HudIconKind, HudLine,
};
use crate::ui::widgets::VideoCanvas;
use crate::video_player::{subscription::PlaybackMessage, SharedLufsCache, VideoPlayer};
use iced::widget::scrollable::{self, AbsoluteOffset, Id, RelativeOffset};
use iced::{event, keyboard, mouse, window, Element, Point, Rectangle, Task};
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Identifier used for the viewer scrollable widget.
pub const SCROLLABLE_ID: &str = "viewer-image-scrollable";
const DOUBLE_CLICK_THRESHOLD: Duration = Duration::from_millis(350);
const MOUSE_MOVEMENT_THRESHOLD: f32 = 10.0; // Minimum pixels to consider real movement (filter sensor noise)
const FULLSCREEN_ENTRY_IGNORE_DELAY: Duration = Duration::from_millis(500); // Ignore mouse movements for 500ms after entering fullscreen
const LOADING_TIMEOUT: Duration = Duration::from_secs(10); // Timeout for media loading

/// Messages emitted by viewer-related widgets.
#[derive(Debug, Clone)]
pub enum Message {
    StartLoadingMedia,
    ImageLoaded(Result<MediaData, Error>),
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
    /// Capture current frame and export to file.
    /// Contains the video path and current position for default filename generation.
    CaptureFrame {
        video_path: PathBuf,
        position_secs: f64,
    },
}

#[derive(Debug, Clone)]
pub struct ErrorState {
    friendly_key: &'static str,
    friendly_text: String,
    details: String,
    show_details: bool,
}

impl ErrorState {
    fn from_error(error: &Error, i18n: &I18n) -> Self {
        let friendly_key = match error {
            Error::Io(_) => "error-load-image-io",
            Error::Svg(_) => "error-load-image-svg",
            #[allow(unreachable_patterns)]
            _ => "error-load-image-general",
        };

        Self {
            friendly_key,
            friendly_text: i18n.tr(friendly_key),
            details: error.to_string(),
            show_details: false,
        }
    }

    fn refresh_translation(&mut self, i18n: &I18n) {
        self.friendly_text = i18n.tr(self.friendly_key);
    }

    pub fn details(&self) -> &str {
        &self.details
    }
}

/// Environment information required to render the viewer.
pub struct ViewEnv<'a> {
    pub i18n: &'a I18n,
    pub background_theme: crate::config::BackgroundTheme,
    pub is_fullscreen: bool,
    pub overlay_hide_delay: std::time::Duration,
}

/// Complete viewer component state.
pub struct State {
    media: Option<MediaData>,
    error: Option<ErrorState>,
    pub zoom: ZoomState,
    pub viewport: ViewportState,
    pub drag: DragState,
    cursor_position: Option<Point>,
    last_click: Option<Instant>,
    pub current_image_path: Option<PathBuf>,
    pub image_list: ImageList,
    arrows_visible: bool,
    last_mouse_move: Option<Instant>,
    last_overlay_interaction: Option<Instant>,
    last_mouse_position: Option<Point>, // Track last position to filter micro-movements
    fullscreen_entered_at: Option<Instant>, // Track when fullscreen was entered to ignore initial movements

    // Loading state
    pub is_loading_media: bool,
    pub loading_started_at: Option<Instant>,
    spinner_rotation: f32, // Rotation angle for animated spinner (in radians)

    // Video playback state
    video_player: Option<VideoPlayer>,
    video_canvas: VideoCanvas<Message>,
    current_video_path: Option<PathBuf>,
    playback_session_id: u64, // Incremented each time playback starts, ensures unique subscription ID

    /// Fit-to-window setting for videos (separate from images).
    /// Always defaults to true for videos and is NOT persisted.
    video_fit_to_window: bool,

    /// Preview position for seek slider (0.0 to 1.0).
    /// Set during slider drag, cleared on release.
    seek_preview_position: Option<f32>,

    /// Whether videos should auto-play when loaded.
    video_autoplay: bool,

    /// Video volume level (0.0 to 1.0).
    video_volume: f32,

    /// Whether video audio is muted.
    video_muted: bool,
}

#[allow(clippy::derivable_impls)]
impl Default for State {
    fn default() -> Self {
        Self {
            media: None,
            error: None,
            zoom: ZoomState::default(),
            viewport: ViewportState::default(),
            drag: DragState::default(),
            cursor_position: None,
            last_click: None,
            current_image_path: None,
            image_list: ImageList::default(),
            arrows_visible: false,
            last_mouse_move: None,
            last_overlay_interaction: None,
            last_mouse_position: None,
            fullscreen_entered_at: None,
            is_loading_media: false,
            loading_started_at: None,
            spinner_rotation: 0.0,
            video_player: None,
            video_canvas: VideoCanvas::new(),
            current_video_path: None,
            playback_session_id: 0,
            video_fit_to_window: true, // Videos always fit-to-window by default
            seek_preview_position: None,
            video_autoplay: false, // Default to no autoplay
            video_volume: crate::config::DEFAULT_VOLUME,
            video_muted: false,
        }
    }
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has_media(&self) -> bool {
        self.media.is_some()
    }

    pub fn media(&self) -> Option<&MediaData> {
        self.media.as_ref()
    }

    pub fn error(&self) -> Option<&ErrorState> {
        self.error.as_ref()
    }

    pub fn zoom_state(&self) -> &ZoomState {
        &self.zoom
    }

    pub fn zoom_state_mut(&mut self) -> &mut ZoomState {
        &mut self.zoom
    }

    pub fn viewport_state(&self) -> &ViewportState {
        &self.viewport
    }

    pub fn viewport_state_mut(&mut self) -> &mut ViewportState {
        &mut self.viewport
    }

    pub fn drag_state(&self) -> &DragState {
        &self.drag
    }

    pub fn drag_state_mut(&mut self) -> &mut DragState {
        &mut self.drag
    }

    pub fn set_cursor_position(&mut self, position: Option<Point>) {
        self.cursor_position = position;
    }

    pub fn zoom_step_percent(&self) -> f32 {
        self.zoom.zoom_step_percent
    }

    pub fn set_zoom_step_percent(&mut self, value: f32) {
        self.zoom.zoom_step_percent = value;
    }

    /// Returns the effective fit-to-window setting.
    /// For videos, uses the separate video_fit_to_window (not persisted).
    /// For images, uses zoom.fit_to_window (persisted).
    pub fn fit_to_window(&self) -> bool {
        if self.is_video() {
            self.video_fit_to_window
        } else {
            self.zoom.fit_to_window
        }
    }

    /// Returns the image fit-to-window setting (persisted).
    /// Use this when saving preferences - only saves image setting.
    pub fn image_fit_to_window(&self) -> bool {
        self.zoom.fit_to_window
    }

    /// Returns true if the current media is a video.
    pub fn is_video(&self) -> bool {
        matches!(self.media, Some(MediaData::Video(_)))
    }

    /// Returns true if a video is playing or will resume playing after seek/buffer.
    ///
    /// This determines if arrow keys should seek (true) vs navigate (false).
    /// Uses the state machine's `is_playing_or_will_resume()` to correctly handle
    /// the Seeking state during rapid key repeats.
    fn is_video_playing_or_will_resume(&self) -> bool {
        self.video_player
            .as_ref()
            .is_some_and(|p| p.state().is_playing_or_will_resume())
    }

    /// Returns true if a video player exists and has an active session.
    ///
    /// An active session means the player is not stopped or in error state.
    /// This is used to determine if Space should toggle playback vs initiate.
    fn has_active_video_session(&self) -> bool {
        self.video_player.as_ref().is_some_and(|p| {
            !matches!(
                p.state(),
                crate::video_player::PlaybackState::Stopped
                    | crate::video_player::PlaybackState::Error { .. }
            )
        })
    }

    pub fn enable_fit_to_window(&mut self) {
        if self.is_video() {
            self.video_fit_to_window = true;
        } else {
            self.zoom.enable_fit_to_window();
        }
    }

    pub fn disable_fit_to_window(&mut self) {
        if self.is_video() {
            self.video_fit_to_window = false;
        } else {
            self.zoom.disable_fit_to_window();
        }
    }

    pub fn refresh_error_translation(&mut self, i18n: &I18n) {
        if let Some(error) = &mut self.error {
            error.refresh_translation(i18n);
        }
    }

    /// Sets whether videos should auto-play when loaded.
    pub fn set_video_autoplay(&mut self, enabled: bool) {
        self.video_autoplay = enabled;
    }

    pub fn scan_directory(&mut self) -> crate::error::Result<()> {
        if let Some(path) = &self.current_image_path {
            let config = crate::config::load().unwrap_or_default();
            let sort_order = config.sort_order.unwrap_or_default();
            self.image_list = ImageList::scan_directory(path, sort_order)?;
        }
        Ok(())
    }

    /// Returns an exportable frame from the video canvas, if available.
    pub fn exportable_frame(&self) -> Option<crate::media::frame_export::ExportableFrame> {
        self.video_canvas.exportable_frame()
    }

    /// Returns true if media is currently being loaded.
    pub fn is_loading_media(&self) -> bool {
        self.is_loading_media
    }

    /// Checks if loading has timed out and converts to error state if necessary.
    pub fn check_loading_timeout(&mut self, i18n: &I18n) {
        if self.is_loading_media {
            if let Some(started_at) = self.loading_started_at {
                if started_at.elapsed() > LOADING_TIMEOUT {
                    // Loading timed out, convert to error state
                    self.is_loading_media = false;
                    self.loading_started_at = None;
                    self.error = Some(ErrorState {
                        friendly_key: "error-loading-timeout",
                        friendly_text: i18n.tr("error-loading-timeout"),
                        details: format!(
                            "Media loading timed out after {} seconds",
                            LOADING_TIMEOUT.as_secs()
                        ),
                        show_details: false,
                    });
                }
            }
        }
    }

    /// Returns the subscriptions for video playback and spinner animation.
    ///
    /// # Arguments
    /// * `lufs_cache` - Optional shared cache for LUFS measurements (audio normalization)
    /// * `normalization_enabled` - Whether to apply audio normalization
    pub fn subscription(
        &self,
        lufs_cache: Option<SharedLufsCache>,
        normalization_enabled: bool,
        frame_cache_mb: u32,
    ) -> iced::Subscription<Message> {
        // Keep subscription active for ALL playback states including Stopped
        // This ensures the decoder stays alive and can receive pause/resume commands
        // The subscription only gets recreated when playback_session_id changes
        // (which happens when navigating to a different video or starting fresh)
        let video_subscription = if let (Some(_player), Some(ref path)) =
            (&self.video_player, &self.current_video_path)
        {
            // Create cache config from MB setting
            let cache_config = crate::video_player::CacheConfig::new(
                (frame_cache_mb as usize) * 1024 * 1024,
                crate::video_player::frame_cache::DEFAULT_MAX_FRAMES,
            );

            // Always create subscription when we have a video player and path
            // The decoder will handle pause/resume via commands
            crate::video_player::subscription::video_playback(
                path.clone(),
                self.playback_session_id,
                lufs_cache,
                normalization_enabled,
                cache_config,
            )
            .map(Message::PlaybackEvent)
        } else {
            iced::Subscription::none()
        };

        let spinner_subscription = if self.is_loading_media {
            // Animate spinner at 60 FPS while loading
            iced::time::every(std::time::Duration::from_millis(16)).map(|_| Message::SpinnerTick)
        } else {
            iced::Subscription::none()
        };

        iced::Subscription::batch([video_subscription, spinner_subscription])
    }

    pub fn handle_message(&mut self, message: Message, i18n: &I18n) -> (Effect, Task<Message>) {
        match message {
            Message::StartLoadingMedia => {
                // Set loading state
                self.is_loading_media = true;
                self.loading_started_at = Some(Instant::now());
                self.error = None;
                (Effect::None, Task::none())
            }
            Message::ImageLoaded(result) => {
                // Clear loading state
                self.is_loading_media = false;
                self.loading_started_at = None;

                // Clean up previous video state before loading new media
                // This is important when navigating from one media to another
                if self.video_player.is_some() {
                    // Stop the current video player (sends Stop command to decoder)
                    if let Some(ref mut player) = self.video_player {
                        player.stop();
                    }
                    self.video_player = None;
                    self.current_video_path = None;
                    self.video_canvas.clear(); // Clear frame to release memory
                    self.seek_preview_position = None;
                    self.playback_session_id += 1; // Ensure old subscription is dropped
                }
                // Reset video fit-to-window to default for new media
                self.video_fit_to_window = true;

                match result {
                    Ok(media) => {
                        // Create VideoPlayer if this is a video
                        if let MediaData::Video(ref video_data) = media {
                            match VideoPlayer::new(video_data) {
                                Ok(player) => {
                                    self.video_player = Some(player);
                                    self.current_video_path = self.current_image_path.clone();
                                }
                                Err(e) => {
                                    eprintln!("Failed to create video player: {}", e);
                                }
                            }
                        }

                        self.media = Some(media);
                        self.error = None;
                        self.refresh_fit_zoom();
                        // Scan directory on successful image load
                        let _ = self.scan_directory();
                        (Effect::None, Task::none())
                    }
                    Err(error) => {
                        self.media = None;
                        self.error = Some(ErrorState::from_error(&error, i18n));
                        (Effect::None, Task::none())
                    }
                }
            }
            Message::ToggleErrorDetails => {
                if let Some(error) = &mut self.error {
                    error.show_details = !error.show_details;
                }
                (Effect::None, Task::none())
            }
            Message::Controls(control) => {
                if matches!(control, controls::Message::DeleteCurrentImage) {
                    return self.delete_current_image(i18n);
                }
                let result = self.handle_controls(control);

                // Sync video canvas scale with zoom changes
                if self.video_player.is_some() {
                    let zoom_scale = self.zoom.zoom_percent / 100.0;
                    self.video_canvas.set_scale(zoom_scale);
                }

                result
            }
            Message::ViewportChanged { bounds, offset } => {
                self.viewport.update(bounds, offset);
                self.refresh_fit_zoom();
                (Effect::None, Task::none())
            }
            Message::RawEvent { event, .. } => self.handle_raw_event(event),
            Message::NavigateNext => {
                // Reset overlay timer on navigation
                self.last_overlay_interaction = Some(Instant::now());
                // Emit effect to let App handle navigation with ImageNavigator
                (Effect::NavigateNext, Task::none())
            }
            Message::NavigatePrevious => {
                // Reset overlay timer on navigation
                self.last_overlay_interaction = Some(Instant::now());
                // Emit effect to let App handle navigation with ImageNavigator
                (Effect::NavigatePrevious, Task::none())
            }
            Message::DeleteCurrentImage => self.delete_current_image(i18n),
            Message::OpenSettings => (Effect::OpenSettings, Task::none()),
            Message::EnterEditor => (Effect::EnterEditor, Task::none()),
            Message::InitiatePlayback => {
                // Reset overlay timer on interaction
                self.last_overlay_interaction = Some(Instant::now());

                // Toggle playback if player already exists
                if let Some(player) = &mut self.video_player {
                    match player.state() {
                        crate::video_player::PlaybackState::Playing { .. }
                        | crate::video_player::PlaybackState::Buffering { .. } => {
                            player.pause();
                        }
                        _ => {
                            // Resume playback - do NOT increment session ID
                            // The existing subscription must stay active to receive commands
                            player.play();
                        }
                    }
                } else if let Some(MediaData::Video(ref video_data)) = self.media {
                    // Create video player and start playback
                    match VideoPlayer::new(video_data) {
                        Ok(mut player) => {
                            // Start playback
                            player.play();
                            self.video_player = Some(player);

                            // Store video path for subscription
                            self.current_video_path = self.current_image_path.clone();

                            // Increment session ID to create a new unique subscription
                            self.playback_session_id = self.playback_session_id.wrapping_add(1);

                            // Update canvas scale to match current zoom
                            let zoom_scale = self.zoom.zoom_percent / 100.0;
                            self.video_canvas.set_scale(zoom_scale);
                        }
                        Err(e) => {
                            eprintln!("Failed to create video player: {}", e);
                        }
                    }
                }

                (Effect::None, Task::none())
            }
            Message::SpinnerTick => {
                // Update spinner rotation (180° per second = π radians per second)
                // At 60 FPS, that's π/60 radians per frame ≈ 0.0524 radians
                const ROTATION_SPEED: f32 = std::f32::consts::PI / 60.0;
                self.spinner_rotation =
                    (self.spinner_rotation + ROTATION_SPEED) % (2.0 * std::f32::consts::PI);
                (Effect::None, Task::none())
            }
            Message::VideoControls(video_msg) => {
                use super::video_controls::Message as VM;

                // Reset overlay timer on video control interaction
                self.last_overlay_interaction = Some(Instant::now());

                match video_msg {
                    VM::TogglePlayback => {
                        if let Some(player) = &mut self.video_player {
                            match player.state() {
                                crate::video_player::PlaybackState::Playing { .. }
                                | crate::video_player::PlaybackState::Buffering { .. } => {
                                    player.pause();
                                }
                                _ => {
                                    // Resume playback - do NOT increment session ID
                                    // The existing subscription must stay active to receive commands
                                    player.play();
                                }
                            }
                        } else if let Some(MediaData::Video(ref video_data)) = self.media {
                            // Create player if it doesn't exist yet and start playback
                            match VideoPlayer::new(video_data) {
                                Ok(mut player) => {
                                    player.play();
                                    self.video_player = Some(player);
                                    self.current_video_path = self.current_image_path.clone();
                                    self.playback_session_id =
                                        self.playback_session_id.wrapping_add(1);

                                    // Update canvas scale to match current zoom
                                    let zoom_scale = self.zoom.zoom_percent / 100.0;
                                    self.video_canvas.set_scale(zoom_scale);
                                }
                                Err(e) => {
                                    eprintln!("Failed to create video player: {}", e);
                                }
                            }
                        }
                    }
                    VM::SeekPreview(position) => {
                        // Just update the preview position for visual feedback
                        // Don't actually seek until release
                        self.seek_preview_position = Some(position);
                    }
                    VM::SeekCommit => {
                        // Perform actual seek to preview position
                        // Don't clear seek_preview_position here - it will be cleared
                        // when we receive a frame near the seek target
                        if let Some(position) = self.seek_preview_position {
                            if let Some(player) = &mut self.video_player {
                                if let Some(MediaData::Video(ref video_data)) = self.media {
                                    let target_secs = position as f64 * video_data.duration_secs;
                                    player.seek(target_secs);
                                }
                            }
                        }
                    }
                    VM::Seek(position) => {
                        // Legacy seek - direct seek without preview
                        // Position is 0.0 to 1.0, convert to seconds
                        if let Some(player) = &mut self.video_player {
                            if let Some(MediaData::Video(ref video_data)) = self.media {
                                let target_secs = position as f64 * video_data.duration_secs;
                                player.seek(target_secs);
                            }
                        }
                    }
                    VM::SeekRelative(delta_secs) => {
                        // Seek relative to current position
                        // Used by keyboard shortcuts (e.g., arrow keys for ±5s)
                        if let Some(player) = &mut self.video_player {
                            if let Some(current_pos) = player.state().position() {
                                let target_secs = current_pos + delta_secs;
                                player.seek(target_secs);
                            }
                        }
                    }
                    VM::SetVolume(volume) => {
                        // Clamp volume to valid range
                        self.video_volume =
                            volume.clamp(crate::config::MIN_VOLUME, crate::config::MAX_VOLUME);
                        // Apply to audio output
                        if let Some(player) = &self.video_player {
                            player.set_volume(self.video_volume);
                        }
                    }
                    VM::ToggleMute => {
                        self.video_muted = !self.video_muted;
                        // Apply to audio output
                        if let Some(player) = &self.video_player {
                            player.set_muted(self.video_muted);
                        }
                    }
                    VM::ToggleLoop => {
                        if let Some(player) = &mut self.video_player {
                            let current = player.is_loop_enabled();
                            player.set_loop(!current);
                        }
                    }
                    VM::CaptureFrame => {
                        // Capture current frame and request export
                        if let Some(video_path) = &self.current_video_path {
                            let position_secs = self
                                .video_player
                                .as_ref()
                                .and_then(|p| p.state().position())
                                .unwrap_or(0.0);
                            return (
                                Effect::CaptureFrame {
                                    video_path: video_path.clone(),
                                    position_secs,
                                },
                                Task::none(),
                            );
                        }
                    }
                    VM::StepForward => {
                        // Step forward one frame (only when paused)
                        if let Some(player) = &mut self.video_player {
                            if player.state().is_paused() {
                                player.step_forward();
                            }
                        }
                    }
                    VM::StepBackward => {
                        // Step backward one frame (only when paused)
                        if let Some(player) = &mut self.video_player {
                            if player.state().is_paused() {
                                player.step_backward();
                            }
                        }
                    }
                }
                (Effect::None, Task::none())
            }
            Message::PlaybackEvent(event) => {
                match event {
                    PlaybackMessage::Started(command_sender) => {
                        // Store the command sender in the player for pause/play/seek
                        if let Some(ref mut player) = self.video_player {
                            player.set_command_sender(command_sender);

                            // Apply current volume and mute state
                            player.set_volume(self.video_volume);
                            player.set_muted(self.video_muted);

                            // Auto-play if enabled
                            if self.video_autoplay {
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
                        self.video_canvas.set_frame(rgba_data, width, height);

                        // Update player position
                        if let Some(ref mut player) = self.video_player {
                            player.update_position(pts_secs);
                        }

                        // Clear seek preview if we received a frame near the seek target
                        // This ensures the slider stays at the new position after seek completes
                        if let Some(preview_pos) = self.seek_preview_position {
                            if let Some(MediaData::Video(ref video_data)) = self.media {
                                let preview_secs = preview_pos as f64 * video_data.duration_secs;
                                // Clear preview if frame is within 0.5 seconds of target
                                if (pts_secs - preview_secs).abs() < 0.5 {
                                    self.seek_preview_position = None;
                                }
                            }
                        }
                    }
                    PlaybackMessage::Buffering => {
                        // Update player to buffering state, but not if we're seeking
                        // (Seeking state needs to be preserved to know whether to resume playing)
                        if let Some(ref mut player) = self.video_player {
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
                        // Handle end of stream
                        if let Some(ref mut player) = self.video_player {
                            if player.is_loop_enabled() {
                                // Restart playback from beginning
                                player.seek(0.0);
                                player.play();
                            } else {
                                // Pause at end (don't stop, so user can seek back)
                                let duration = player.video_data().duration_secs;
                                player.pause_at(duration);
                            }
                        }
                    }
                    PlaybackMessage::Error(msg) => {
                        // Display error
                        eprintln!("Playback error: {}", msg);
                        if let Some(ref mut player) = self.video_player {
                            player.set_error(msg);
                        }
                    }
                    PlaybackMessage::AudioPts(pts_secs) => {
                        // Update sync clock with audio PTS for A/V synchronization
                        if let Some(ref player) = self.video_player {
                            player.update_audio_pts(pts_secs);
                        }
                    }
                }

                (Effect::None, Task::none())
            }
        }
    }

    pub fn view<'a>(&'a self, env: ViewEnv<'a>) -> Element<'a, Message> {
        let geometry_state = self.geometry_state();

        let error = self.error.as_ref().map(|error| viewer::ErrorContext {
            friendly_text: &error.friendly_text,
            details: &error.details,
            show_details: error.show_details,
        });

        let position_line = geometry_state
            .scroll_position_percentage()
            .map(|(px, py)| format_position_indicator(env.i18n, px, py));

        let zoom_line = if (self.zoom.zoom_percent - crate::ui::state::zoom::DEFAULT_ZOOM_PERCENT)
            .abs()
            > f32::EPSILON
        {
            Some(format_zoom_indicator(env.i18n, self.zoom.zoom_percent))
        } else {
            None
        };

        let media_type_line = self.media.as_ref().and_then(format_media_indicator);

        let hud_lines = position_line
            .into_iter()
            .chain(zoom_line)
            .chain(media_type_line)
            .collect::<Vec<HudLine>>();

        // In fullscreen, overlay auto-hides after delay
        // In windowed mode, controls stay visible but center overlay (pause button) can hide
        let overlay_should_be_visible = if env.is_fullscreen {
            self.last_overlay_interaction
                .map(|t| t.elapsed() < env.overlay_hide_delay)
                .unwrap_or(false)
        } else {
            true
        };

        // For center video overlay (play/pause button), use auto-hide in both modes when playing
        let is_currently_playing = self.video_player.is_some()
            && matches!(
                self.video_player.as_ref().map(|p| p.state()),
                Some(crate::video_player::PlaybackState::Playing { .. })
                    | Some(crate::video_player::PlaybackState::Buffering { .. })
            );

        let center_overlay_visible = if is_currently_playing {
            // When playing, center overlay (pause button) auto-hides after delay
            self.last_overlay_interaction
                .map(|t| t.elapsed() < env.overlay_hide_delay)
                .unwrap_or(false)
        } else {
            // When paused/stopped, play button always visible
            true
        };

        let image = self.media.as_ref().map(|image_data| viewer::ImageContext {
            i18n: env.i18n,
            controls_context: controls::ViewContext { i18n: env.i18n },
            zoom: &self.zoom,
            effective_fit_to_window: self.fit_to_window(),
            pane_context: pane::ViewContext {
                background_theme: env.background_theme,
                hud_lines,
                scrollable_id: SCROLLABLE_ID,
                i18n: env.i18n,
            },
            pane_model: pane::ViewModel {
                media: image_data,
                zoom_percent: self.zoom.zoom_percent,
                padding: geometry_state.media_padding(),
                is_dragging: self.drag.is_dragging,
                cursor_over_media: geometry_state.is_cursor_over_media(),
                arrows_visible: if env.is_fullscreen {
                    // In fullscreen, arrows use same auto-hide logic as controls
                    self.arrows_visible && !self.image_list.is_empty() && overlay_should_be_visible
                } else {
                    // In windowed mode, arrows visible on hover (current behavior)
                    self.arrows_visible && !self.image_list.is_empty()
                },
                overlay_visible: center_overlay_visible,
                has_next: self.image_list.next().is_some(),
                has_previous: self.image_list.previous().is_some(),
                at_first: self.image_list.is_at_first(),
                at_last: self.image_list.is_at_last(),
                current_index: self.image_list.current_index(),
                total_count: self.image_list.len(),
                position_counter_visible: if env.is_fullscreen {
                    // In fullscreen, use same auto-hide logic as arrows and controls
                    !self.image_list.is_empty() && overlay_should_be_visible
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
                video_canvas: Some(&self.video_canvas),
                // Use is_playing_or_will_resume() to include Seeking state
                // This prevents the play button from flashing during seek operations
                is_video_playing: self.is_video_playing_or_will_resume(),
                is_loading_media: self.is_loading_media,
                spinner_rotation: self.spinner_rotation,
                video_error: self
                    .video_player
                    .as_ref()
                    .and_then(|p| p.state().error_message()),
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
            video_playback_state: self.media.as_ref().and_then(|media| {
                // Build PlaybackState for video controls
                // Show controls for any video, not just when VideoPlayer exists
                if let MediaData::Video(ref video_data) = media {
                    let (is_playing, position_secs, loop_enabled) =
                        if let Some(player) = &self.video_player {
                            let state = player.state();
                            match state {
                                crate::video_player::PlaybackState::Playing { position_secs } => {
                                    (true, *position_secs, player.is_loop_enabled())
                                }
                                crate::video_player::PlaybackState::Paused { position_secs } => {
                                    (false, *position_secs, player.is_loop_enabled())
                                }
                                crate::video_player::PlaybackState::Buffering { position_secs } => {
                                    (true, *position_secs, player.is_loop_enabled())
                                }
                                _ => (false, 0.0, player.is_loop_enabled()),
                            }
                        } else {
                            // No player yet - show initial state (paused at 0)
                            (false, 0.0, false)
                        };

                    Some(video_controls::PlaybackState {
                        is_playing,
                        position_secs,
                        duration_secs: video_data.duration_secs,
                        volume: self.video_volume,
                        muted: self.video_muted,
                        loop_enabled,
                        seek_preview_position: self.seek_preview_position,
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
            is_loading: self.is_loading_media,
            spinner_rotation: self.spinner_rotation,
        })
    }

    fn handle_controls(&mut self, message: controls::Message) -> (Effect, Task<Message>) {
        use controls::Message::*;

        match message {
            ZoomInputChanged(value) => {
                self.zoom.zoom_input = value;
                self.zoom.zoom_input_dirty = true;
                self.zoom.zoom_input_error_key = None;
                (Effect::None, Task::none())
            }
            ZoomInputSubmitted => {
                self.zoom.zoom_input_dirty = false;

                if let Some(value) = parse_number(&self.zoom.zoom_input) {
                    self.zoom.apply_manual_zoom(value);
                    // Also disable video fit-to-window when manually setting zoom
                    if self.is_video() {
                        self.video_fit_to_window = false;
                    }
                    (Effect::PersistPreferences, Task::none())
                } else {
                    self.zoom.zoom_input_error_key =
                        Some(crate::ui::state::zoom::ZOOM_INPUT_INVALID_KEY);
                    (Effect::None, Task::none())
                }
            }
            ResetZoom => {
                self.zoom
                    .apply_manual_zoom(crate::ui::state::zoom::DEFAULT_ZOOM_PERCENT);
                // Also disable video fit-to-window when resetting zoom
                if self.is_video() {
                    self.video_fit_to_window = false;
                }
                (Effect::PersistPreferences, Task::none())
            }
            ZoomIn => {
                self.zoom
                    .apply_manual_zoom(self.zoom.zoom_percent + self.zoom.zoom_step_percent);
                // Also disable video fit-to-window when zooming on a video
                if self.is_video() {
                    self.video_fit_to_window = false;
                }
                (Effect::PersistPreferences, Task::none())
            }
            ZoomOut => {
                self.zoom
                    .apply_manual_zoom(self.zoom.zoom_percent - self.zoom.zoom_step_percent);
                // Also disable video fit-to-window when zooming on a video
                if self.is_video() {
                    self.video_fit_to_window = false;
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
                self.last_overlay_interaction = None;
                self.last_mouse_position = None;
                self.fullscreen_entered_at = Some(Instant::now());
                (Effect::ToggleFullscreen, Task::none())
            }
            DeleteCurrentImage => (Effect::None, Task::none()),
        }
    }

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
                    let effect = if let Some(position) = self.cursor_position {
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
                    self.cursor_position = Some(position);

                    // Calculate distance from last recorded position to filter sensor noise
                    let (_distance, is_real_movement) = self
                        .last_mouse_position
                        .map(|last_pos| {
                            let dx = position.x - last_pos.x;
                            let dy = position.y - last_pos.y;
                            let dist = (dx * dx + dy * dy).sqrt();
                            (dist, dist >= MOUSE_MOVEMENT_THRESHOLD)
                        })
                        .unwrap_or((f32::MAX, true)); // First movement is always real

                    // Only process if real movement (not sensor noise)
                    if is_real_movement {
                        self.last_mouse_move = Some(Instant::now());
                        self.last_mouse_position = Some(position);
                        // Show arrows when cursor is anywhere in the viewer
                        self.arrows_visible = true;

                        // Ignore mouse movements shortly after entering fullscreen to avoid
                        // triggering controls from window resize events
                        let ignore_due_to_fullscreen_entry = self
                            .fullscreen_entered_at
                            .map(|entered| entered.elapsed() < FULLSCREEN_ENTRY_IGNORE_DELAY)
                            .unwrap_or(false);

                        if ignore_due_to_fullscreen_entry {
                            // Ignoring movement within 500ms of fullscreen entry
                        } else {
                            // Record interaction time for overlay auto-hide (fullscreen)
                            // Reset timer on EVERY real mouse movement to keep controls visible
                            // This follows the standard video player pattern (YouTube, VLC, etc.)
                            self.last_overlay_interaction = Some(Instant::now());
                        }
                    }

                    if self.drag.is_dragging {
                        let task = self.handle_cursor_moved_during_drag(position);
                        (Effect::None, task)
                    } else {
                        (Effect::None, Task::none())
                    }
                }
                mouse::Event::CursorLeft => {
                    self.cursor_position = None;
                    self.arrows_visible = false;
                    if self.drag.is_dragging {
                        self.drag.stop();
                    }
                    (Effect::None, Task::none())
                }
                _ => (Effect::None, Task::none()),
            },
            event::Event::Keyboard(keyboard_event) => match keyboard_event {
                keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::F11),
                    ..
                } => {
                    // Clear overlay timer and position when entering fullscreen to hide controls
                    self.last_overlay_interaction = None;
                    self.last_mouse_position = None;
                    self.fullscreen_entered_at = Some(Instant::now());
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
                        self.handle_message(
                            Message::VideoControls(video_controls::Message::TogglePlayback),
                            &I18n::default(),
                        )
                    } else if matches!(self.media, Some(MediaData::Video(_))) {
                        // Video loaded but not playing yet - initiate playback
                        self.handle_message(Message::InitiatePlayback, &I18n::default())
                    } else {
                        (Effect::None, Task::none())
                    }
                }
                keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::ArrowRight),
                    ..
                } => {
                    // ArrowRight: Seek +5s if video is playing, otherwise navigate to next media
                    // Uses is_playing_or_will_resume() to handle rapid key repeats during seek
                    if self.is_video_playing_or_will_resume() {
                        self.handle_message(
                            Message::VideoControls(video_controls::Message::SeekRelative(5.0)),
                            &I18n::default(),
                        )
                    } else {
                        self.handle_message(Message::NavigateNext, &I18n::default())
                    }
                }
                keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::ArrowLeft),
                    ..
                } => {
                    // ArrowLeft: Seek -5s if video is playing, otherwise navigate to previous media
                    // Uses is_playing_or_will_resume() to handle rapid key repeats during seek
                    if self.is_video_playing_or_will_resume() {
                        self.handle_message(
                            Message::VideoControls(video_controls::Message::SeekRelative(-5.0)),
                            &I18n::default(),
                        )
                    } else {
                        self.handle_message(Message::NavigatePrevious, &I18n::default())
                    }
                }
                keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::ArrowUp),
                    ..
                } => {
                    // ArrowUp: Increase volume (only during video playback)
                    if self.has_active_video_session() {
                        let new_volume = (self.video_volume + crate::config::VOLUME_STEP)
                            .min(crate::config::MAX_VOLUME);
                        self.handle_message(
                            Message::VideoControls(video_controls::Message::SetVolume(new_volume)),
                            &I18n::default(),
                        )
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
                        let new_volume = (self.video_volume - crate::config::VOLUME_STEP)
                            .max(crate::config::MIN_VOLUME);
                        self.handle_message(
                            Message::VideoControls(video_controls::Message::SetVolume(new_volume)),
                            &I18n::default(),
                        )
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
                        self.handle_message(
                            Message::VideoControls(video_controls::Message::ToggleMute),
                            &I18n::default(),
                        )
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
                    if self.current_image_path.is_some() && !self.is_video() {
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
                    if let Some(player) = &mut self.video_player {
                        if !player.state().is_playing_or_will_resume() {
                            player.step_backward();
                        }
                    }
                    (Effect::None, Task::none())
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
                    if let Some(player) = &mut self.video_player {
                        if !player.state().is_playing_or_will_resume() {
                            player.step_forward();
                        }
                    }
                    (Effect::None, Task::none())
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
            let now = Instant::now();
            let double_click = self
                .last_click
                .map(|instant| now.duration_since(instant) <= DOUBLE_CLICK_THRESHOLD)
                .unwrap_or(false);
            self.last_click = Some(now);

            // Reset overlay timer on any left click, even on UI controls
            // This keeps controls visible when user is interacting
            self.last_overlay_interaction = Some(now);

            if self.geometry_state().is_cursor_over_media() {
                if double_click {
                    // Clear overlay timer when entering fullscreen (will hide controls initially)
                    self.last_overlay_interaction = None;
                    self.last_mouse_position = None;
                    self.fullscreen_entered_at = Some(Instant::now());
                    return Effect::ToggleFullscreen;
                }

                self.drag.start(position, self.viewport.offset);
            }
        }

        Effect::None
    }

    fn handle_mouse_button_released(&mut self, button: mouse::Button) {
        if button == mouse::Button::Left {
            self.drag.stop();
        }
    }

    /// Updates the viewport when the user drags the image. Clamps the offset to
    /// the scaled image bounds and mirrors the change to the scrollable widget
    /// so keyboard/scroll interactions stay in sync.
    fn handle_cursor_moved_during_drag(&mut self, position: Point) -> Task<Message> {
        let proposed_offset = match self.drag.calculate_offset(position) {
            Some(offset) => offset,
            None => return Task::none(),
        };

        let geometry_state = self.geometry_state();
        if let (Some(viewport), Some(size)) =
            (self.viewport.bounds, geometry_state.scaled_media_size())
        {
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

            scrollable::snap_to(
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

        let new_zoom = self.zoom.zoom_percent + steps * self.zoom.zoom_step_percent;
        self.zoom.apply_manual_zoom(new_zoom);

        // Also disable video fit-to-window when zooming on a video
        if self.is_video() {
            self.video_fit_to_window = false;
        }

        true
    }

    /// Recomputes the fit-to-window zoom when layout-affecting events occur so
    /// the zoom textbox always mirrors the actual fit percentage.
    fn refresh_fit_zoom(&mut self) {
        // Use effective fit_to_window (considers video vs image)
        let effective_fit_to_window = self.fit_to_window();
        if effective_fit_to_window {
            if let Some(fit_zoom) = self.compute_fit_zoom_percent() {
                self.zoom.update_zoom_display(fit_zoom);
                self.zoom.zoom_input_dirty = false;
                self.zoom.zoom_input_error_key = None;

                // Sync video canvas scale when fit-to-window zoom changes
                if self.video_player.is_some() {
                    let zoom_scale = fit_zoom / 100.0;
                    self.video_canvas.set_scale(zoom_scale);
                }
            }
        }
    }

    /// Calculates the zoom percentage needed to fit the current image inside
    /// the viewport. Returns `None` until viewport bounds are known.
    pub fn compute_fit_zoom_percent(&self) -> Option<f32> {
        let media = self.media.as_ref()?;
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
            self.media.as_ref(),
            &self.viewport,
            self.zoom.zoom_percent,
            self.cursor_position,
        )
    }

    fn load_image_task(path: PathBuf) -> Task<Message> {
        Task::perform(
            async move { crate::media::load_media(&path) },
            Message::ImageLoaded,
        )
    }

    fn delete_current_image(&mut self, i18n: &I18n) -> (Effect, Task<Message>) {
        let Some(current_path) = self.current_image_path.clone() else {
            return (Effect::None, Task::none());
        };

        let has_multiple = self.image_list.len() > 1;
        let next_candidate = if has_multiple {
            self.image_list
                .next()
                .map(|path| path.to_path_buf())
                .filter(|next| next != &current_path)
        } else {
            None
        };

        match std::fs::remove_file(&current_path) {
            Ok(()) => {
                self.media = None;
                self.error = None;

                let scan_seed = next_candidate
                    .as_ref()
                    .cloned()
                    .unwrap_or_else(|| current_path.clone());
                self.current_image_path = Some(scan_seed);
                let _ = self.scan_directory();

                if let Some(next_path) = next_candidate {
                    self.current_image_path = Some(next_path.clone());
                    self.image_list.set_current(&next_path);
                    (Effect::None, Self::load_image_task(next_path))
                } else {
                    self.current_image_path = None;
                    (Effect::None, Task::none())
                }
            }
            Err(err) => {
                self.error = Some(ErrorState {
                    friendly_key: "error-delete-image-io",
                    friendly_text: i18n.tr("error-delete-image-io"),
                    details: err.to_string(),
                    show_details: false,
                });
                (Effect::None, Task::none())
            }
        }
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
        text: format!("{:.0}% x {:.0}%", px, py),
    }
}

fn format_zoom_indicator(_i18n: &I18n, zoom_percent: f32) -> HudLine {
    HudLine {
        icon: HudIconKind::Zoom,
        text: format!("{:.0}%", zoom_percent),
    }
}

/// Formats video duration in HH:MM:SS or MM:SS format.
///
/// Hours are only shown if duration is >= 1 hour to keep the display compact.
fn format_duration(duration_secs: f64) -> String {
    let total_secs = duration_secs as u64;
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{:02}:{:02}", minutes, seconds)
    }
}

/// Generates HUD indicator text for media type.
///
/// Returns formatted string for videos (with duration and optional audio badge),
/// or None for images to avoid cluttering the UI with redundant information.
fn format_media_indicator(media: &MediaData) -> Option<HudLine> {
    match media {
        MediaData::Video(video_data) => {
            let duration_str = format_duration(video_data.duration_secs);
            let text = if video_data.has_audio {
                format!("Video {} (audio)", duration_str)
            } else {
                format!("Video {}", duration_str)
            };

            Some(HudLine {
                icon: HudIconKind::Video {
                    has_audio: video_data.has_audio,
                },
                text,
            })
        }
        MediaData::Image(_) => None, // Don't show indicator for images
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn format_duration_formats_correctly() {
        assert_eq!(format_duration(0.0), "00:00");
        assert_eq!(format_duration(5.0), "00:05");
        assert_eq!(format_duration(65.0), "01:05");
        assert_eq!(format_duration(125.0), "02:05");
        assert_eq!(format_duration(3665.0), "01:01:05");
        assert_eq!(format_duration(7384.0), "02:03:04");
    }

    #[test]
    fn format_media_indicator_shows_video_with_duration() {
        use crate::media::{ImageData, VideoData};
        use iced::widget::image::Handle;

        let pixels = vec![255_u8; 4];
        let thumbnail = ImageData {
            handle: Handle::from_rgba(1, 1, pixels),
            width: 1,
            height: 1,
        };

        let video_data = VideoData {
            thumbnail,
            width: 1920,
            height: 1080,
            duration_secs: 125.0,
            fps: 30.0,
            has_audio: false,
        };

        let media = MediaData::Video(video_data);
        let indicator = format_media_indicator(&media);

        let hud = indicator.expect("expected HUD line for video");
        assert!(matches!(hud.icon, HudIconKind::Video { has_audio: false }));
        assert!(hud.text.contains("02:05"));
    }

    #[test]
    fn format_media_indicator_shows_audio_badge() {
        use crate::media::{ImageData, VideoData};
        use iced::widget::image::Handle;

        let pixels = vec![255_u8; 4];
        let thumbnail = ImageData {
            handle: Handle::from_rgba(1, 1, pixels),
            width: 1,
            height: 1,
        };

        let video_data = VideoData {
            thumbnail,
            width: 1920,
            height: 1080,
            duration_secs: 65.0,
            fps: 30.0,
            has_audio: true,
        };

        let media = MediaData::Video(video_data);
        let indicator = format_media_indicator(&media);

        let hud = indicator.expect("expected HUD line for video with audio");
        assert!(matches!(hud.icon, HudIconKind::Video { has_audio: true }));
        assert!(hud.text.contains("01:05"));
        assert!(hud.text.contains("audio"));
    }

    #[test]
    fn loading_state_timeout_converts_to_error() {
        let i18n = I18n::default();
        let mut state = State::new();

        // Simulate starting to load media
        state.is_loading_media = true;
        state.loading_started_at = Some(Instant::now() - LOADING_TIMEOUT - Duration::from_secs(1));

        // Check timeout should convert to error
        state.check_loading_timeout(&i18n);

        assert!(!state.is_loading_media, "loading flag should be cleared");
        assert!(
            state.loading_started_at.is_none(),
            "loading timestamp should be cleared"
        );
        assert!(state.error.is_some(), "error should be set");

        let error = state.error.unwrap();
        assert_eq!(error.friendly_key, "error-loading-timeout");
    }

    #[test]
    fn loading_state_timeout_does_not_trigger_before_timeout() {
        let i18n = I18n::default();
        let mut state = State::new();

        // Simulate starting to load media (but not timed out yet)
        state.is_loading_media = true;
        state.loading_started_at = Some(Instant::now() - Duration::from_secs(5));

        // Check timeout should NOT convert to error yet
        state.check_loading_timeout(&i18n);

        assert!(state.is_loading_media, "loading flag should still be set");
        assert!(
            state.loading_started_at.is_some(),
            "loading timestamp should still be set"
        );
        assert!(state.error.is_none(), "error should not be set");
    }

    #[test]
    fn loading_state_resets_on_successful_load() {
        let i18n = I18n::default();
        let mut state = State::new();

        // Simulate loading state
        state.is_loading_media = true;
        state.loading_started_at = Some(Instant::now());

        // Simulate successful load (ImageLoaded with Ok result)
        use crate::media::ImageData;
        use iced::widget::image::Handle;

        let pixels = vec![255_u8; 4];
        let image_data = ImageData {
            handle: Handle::from_rgba(1, 1, pixels),
            width: 100,
            height: 100,
        };

        let (_effect, _task) = state.handle_message(
            Message::ImageLoaded(Ok(MediaData::Image(image_data))),
            &i18n,
        );

        assert!(
            !state.is_loading_media,
            "loading flag should be cleared after successful load"
        );
        assert!(
            state.loading_started_at.is_none(),
            "loading timestamp should be cleared"
        );
        assert!(state.error.is_none(), "no error should be set");
    }

    #[test]
    fn format_media_indicator_returns_none_for_images() {
        use crate::media::ImageData;
        use iced::widget::image::Handle;

        let pixels = vec![255_u8; 4];
        let image_data = ImageData {
            handle: Handle::from_rgba(1, 1, pixels),
            width: 100,
            height: 100,
        };

        let media = MediaData::Image(image_data);
        let indicator = format_media_indicator(&media);
        assert!(indicator.is_none());
    }

    #[test]
    fn overlay_timer_resets_on_real_mouse_movement() {
        use std::thread::sleep;

        let mut state = State::new();

        // Simulate entering fullscreen - timer should be None initially
        state.fullscreen_entered_at = Some(Instant::now());
        assert!(state.last_overlay_interaction.is_none());

        // Wait for fullscreen entry delay to pass
        sleep(Duration::from_millis(501));

        // First real mouse movement (distance > threshold)
        let event1 = event::Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(100.0, 100.0),
        });
        let (_effect, _task) = state.handle_raw_event(event1);

        // Timer should now be set
        let first_timer = state.last_overlay_interaction;
        assert!(
            first_timer.is_some(),
            "Timer should be set after first movement"
        );

        // Small delay
        sleep(Duration::from_millis(100));

        // Second real mouse movement (distance > threshold from last position)
        let event2 = event::Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(115.0, 115.0),
        });
        let (_effect, _task) = state.handle_raw_event(event2);

        // Timer should be RESET (updated to a new value)
        let second_timer = state.last_overlay_interaction;
        assert!(second_timer.is_some(), "Timer should still be set");
        assert!(
            second_timer.unwrap() > first_timer.unwrap(),
            "Timer should be reset (newer timestamp) after second movement"
        );
    }

    #[test]
    fn overlay_ignores_micro_movements() {
        let mut state = State::new();
        state.fullscreen_entered_at = Some(Instant::now());

        // Wait for fullscreen entry delay
        std::thread::sleep(Duration::from_millis(501));

        // First movement - establishes baseline
        let event1 = event::Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(100.0, 100.0),
        });
        let (_effect, _task) = state.handle_raw_event(event1);
        assert!(state.last_overlay_interaction.is_some());

        let timer_before = state.last_overlay_interaction;

        // Small movement (< threshold) - should be ignored
        std::thread::sleep(Duration::from_millis(10));
        let event2 = event::Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(101.0, 101.0), // ~1.4 pixels distance
        });
        let (_effect, _task) = state.handle_raw_event(event2);

        // Timer should NOT change (micro-movement filtered)
        assert_eq!(
            state.last_overlay_interaction, timer_before,
            "Timer should not reset for micro-movements"
        );
    }

    #[test]
    fn overlay_ignores_movements_during_fullscreen_entry() {
        let mut state = State::new();

        // Simulate entering fullscreen
        state.fullscreen_entered_at = Some(Instant::now());

        // Immediate mouse movement (within 500ms window)
        let event = event::Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(200.0, 200.0),
        });
        let (_effect, _task) = state.handle_raw_event(event);

        // Timer should NOT be set (movement ignored during entry period)
        assert!(
            state.last_overlay_interaction.is_none(),
            "Should ignore mouse movements during fullscreen entry delay"
        );
    }

    #[test]
    fn overlay_clears_on_fullscreen_toggle() {
        let mut state = State::new();

        // Set up some state
        state.last_overlay_interaction = Some(Instant::now());
        state.last_mouse_position = Some(Point::new(50.0, 50.0));

        // Toggle fullscreen (via button)
        let (effect, _) = state.handle_controls(controls::Message::ToggleFullscreen);

        assert_eq!(effect, Effect::ToggleFullscreen);
        assert!(state.last_overlay_interaction.is_none());
        assert!(state.last_mouse_position.is_none());
        assert!(state.fullscreen_entered_at.is_some());
    }

    #[test]
    fn arrows_always_visible_in_windowed_mode() {
        let mut state = State::new();

        // Simulate mouse movement to show arrows
        let event = event::Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(100.0, 100.0),
        });
        let (_effect, _task) = state.handle_raw_event(event);

        // In windowed mode, arrows should be visible
        assert!(
            state.arrows_visible,
            "Arrows should be visible after mouse movement"
        );
    }

    #[test]
    fn arrows_auto_hide_in_fullscreen_after_delay() {
        use std::thread::sleep;

        let mut state = State::new();

        // Enter fullscreen
        state.fullscreen_entered_at = Some(Instant::now());

        // Wait for fullscreen entry delay
        sleep(Duration::from_millis(501));

        // Move mouse to show arrows and controls
        let event = event::Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(100.0, 100.0),
        });
        let (_effect, _task) = state.handle_raw_event(event);

        assert!(
            state.arrows_visible,
            "Arrows should be visible after movement"
        );
        assert!(
            state.last_overlay_interaction.is_some(),
            "Timer should be set"
        );

        // Check that arrows would be hidden after delay (using default 3s)
        let timer = state.last_overlay_interaction.unwrap();
        let default_delay = Duration::from_secs(crate::config::DEFAULT_OVERLAY_TIMEOUT_SECS as u64);

        // Simulate 2 seconds elapsed (arrows still visible)
        sleep(Duration::from_millis(2000));
        let should_show_at_2s = timer.elapsed() < default_delay;
        assert!(should_show_at_2s, "Arrows should still be visible at 2s");

        // Simulate 3+ seconds elapsed (arrows should hide)
        sleep(Duration::from_millis(1100));
        let should_hide_at_3s = timer.elapsed() >= default_delay;
        assert!(should_hide_at_3s, "Arrows should be hidden after 3s");
    }

    #[test]
    fn keyboard_navigation_always_works() {
        let mut state = State::new();

        // In fullscreen with arrows hidden (no timer set)
        state.fullscreen_entered_at = Some(Instant::now());
        state.arrows_visible = false;
        state.last_overlay_interaction = None;

        // Keyboard navigation should still work
        let (effect, _) = state.handle_message(Message::NavigateNext, &I18n::default());

        assert_eq!(
            effect,
            Effect::NavigateNext,
            "Keyboard navigation should work even when arrows are hidden"
        );
    }

    #[test]
    fn play_button_interaction_resets_overlay_timer() {
        use std::thread::sleep;

        let mut state = State::new();

        // Timer is initially None
        assert!(state.last_overlay_interaction.is_none());

        // Send a playback message
        let (effect, _) = state.handle_message(Message::InitiatePlayback, &I18n::default());

        // Effect should be None, and timer should now be set
        assert_eq!(effect, Effect::None);
        assert!(
            state.last_overlay_interaction.is_some(),
            "Timer should be set after initiating playback"
        );

        let timer_before = state.last_overlay_interaction;
        sleep(Duration::from_millis(50));

        // Send another playback message
        let (effect, _) = state.handle_message(Message::InitiatePlayback, &I18n::default());
        assert_eq!(effect, Effect::None);

        // Timer should be updated to a newer timestamp
        assert!(
            state.last_overlay_interaction > timer_before,
            "Timer should be reset to a newer time"
        );
    }
}
