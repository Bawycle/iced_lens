// SPDX-License-Identifier: MPL-2.0
//! Event subscriptions for the application.
//!
//! This module handles routing of native events (keyboard, mouse, window)
//! to the appropriate screen components based on the current application state.

use super::{Message, Screen};
use crate::ui::diagnostics_screen;
use crate::ui::viewer::component;
use crate::video_player::SharedLufsCache;
use iced::{event, time, Subscription};
use std::time::Duration;

/// Creates the appropriate event subscription based on the current screen.
///
/// Different screens have different event routing needs:
/// - Viewer: Routes all events including wheel scroll for zoom
/// - Editor: Routes keyboard events to editor, window events to viewer
/// - Settings/Help/About: Routes non-wheel events to viewer
///
/// File drop events are only handled on the Viewer screen.
/// Window close requests are handled on all screens for cleanup.
pub fn create_event_subscription(screen: Screen) -> Subscription<Message> {
    match screen {
        Screen::ImageEditor => event::listen_with(|event, status, window_id| {
            // Handle window close request for cleanup
            if let event::Event::Window(iced::window::Event::CloseRequested) = &event {
                return Some(Message::WindowCloseRequested(window_id));
            }

            // File drop is only handled on Viewer screen

            if let event::Event::Window(iced::window::Event::Resized(_)) = &event {
                return Some(Message::Viewer(component::Message::RawEvent {
                    window: window_id,
                    event: event.clone(),
                }));
            }

            // Route mouse wheel scroll to editor for zoom (always, to override scrollable)
            if matches!(
                event,
                event::Event::Mouse(iced::mouse::Event::WheelScrolled { .. })
            ) {
                return Some(Message::ImageEditor(
                    crate::ui::image_editor::Message::RawEvent {
                        window: window_id,
                        event: event.clone(),
                    },
                ));
            }

            // Route mouse events to editor for cursor tracking and pan
            if matches!(
                event,
                event::Event::Mouse(
                    iced::mouse::Event::CursorMoved { .. }
                        | iced::mouse::Event::CursorLeft
                        | iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)
                        | iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)
                )
            ) {
                return Some(Message::ImageEditor(
                    crate::ui::image_editor::Message::RawEvent {
                        window: window_id,
                        event: event.clone(),
                    },
                ));
            }

            // Route keyboard events to editor
            if let event::Event::Keyboard(..) = &event {
                match status {
                    event::Status::Ignored => Some(Message::ImageEditor(
                        crate::ui::image_editor::Message::RawEvent {
                            window: window_id,
                            event: event.clone(),
                        },
                    )),
                    event::Status::Captured => None,
                }
            } else {
                None
            }
        }),
        Screen::Viewer => {
            // In viewer screen, route all events including wheel scroll for zoom
            event::listen_with(|event, status, window_id| {
                // Handle window close request for cleanup
                if let event::Event::Window(iced::window::Event::CloseRequested) = &event {
                    return Some(Message::WindowCloseRequested(window_id));
                }

                // Handle file drop only on Viewer screen
                if let event::Event::Window(iced::window::Event::FileDropped(path)) = &event {
                    return Some(Message::FileDropped(path.clone()));
                }

                // Note: Window::Resized events are passed to the viewer (not intercepted here)
                // and window_size is updated in App::update when processing viewer messages.

                if matches!(
                    event,
                    event::Event::Mouse(iced::mouse::Event::WheelScrolled { .. })
                ) {
                    return Some(Message::Viewer(component::Message::RawEvent {
                        window: window_id,
                        event: event.clone(),
                    }));
                }

                match status {
                    event::Status::Ignored => Some(Message::Viewer(component::Message::RawEvent {
                        window: window_id,
                        event: event.clone(),
                    })),
                    event::Status::Captured => None,
                }
            })
        }
        Screen::Settings | Screen::Help | Screen::About | Screen::Diagnostics => {
            // In settings/help/about/diagnostics screens, only route non-wheel events to viewer
            // (wheel events are used by scrollable content)
            event::listen_with(|event, status, window_id| {
                // Handle window close request for cleanup
                if let event::Event::Window(iced::window::Event::CloseRequested) = &event {
                    return Some(Message::WindowCloseRequested(window_id));
                }

                // File drop is only handled on Viewer screen

                // Don't route wheel scroll to viewer - it's used by scrollable content
                if matches!(
                    event,
                    event::Event::Mouse(iced::mouse::Event::WheelScrolled { .. })
                ) {
                    return None;
                }

                match status {
                    event::Status::Ignored => Some(Message::Viewer(component::Message::RawEvent {
                        window: window_id,
                        event: event.clone(),
                    })),
                    event::Status::Captured => None,
                }
            })
        }
    }
}

/// Creates a periodic tick subscription for overlay auto-hide, loading timeout,
/// and notification auto-dismiss.
pub fn create_tick_subscription(
    fullscreen: bool,
    is_loading: bool,
    has_notifications: bool,
) -> Subscription<Message> {
    if fullscreen || is_loading || has_notifications {
        time::every(std::time::Duration::from_millis(100)).map(Message::Tick)
    } else {
        Subscription::none()
    }
}

/// Creates the video playback subscription with LUFS cache for audio normalization.
pub fn create_video_subscription(
    viewer: &component::State,
    lufs_cache: Option<SharedLufsCache>,
    audio_normalization: bool,
    frame_cache_mb: u32,
    history_mb: u32,
) -> Subscription<Message> {
    viewer
        .subscription(lufs_cache, audio_normalization, frame_cache_mb, history_mb)
        .map(Message::Viewer)
}

/// Creates a subscription for refreshing the diagnostics screen status.
///
/// Polls every 1 second when on the Diagnostics screen to update
/// status, duration, and buffer count displays in real-time.
pub fn create_diagnostics_subscription(screen: Screen) -> Subscription<Message> {
    if screen == Screen::Diagnostics {
        time::every(Duration::from_secs(1))
            .map(|_| Message::Diagnostics(diagnostics_screen::Message::RefreshStatus))
    } else {
        Subscription::none()
    }
}
