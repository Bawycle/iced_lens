// SPDX-License-Identifier: MPL-2.0
//! Viewer component encapsulating state and update logic.

use crate::error::Error;
use crate::i18n::fluent::I18n;
use crate::image_handler::ImageData;
use crate::ui::state::{DragState, ViewportState, ZoomState};
use crate::ui::viewer::{self, controls, pane, state as geometry};
use iced::widget::scrollable::{self, AbsoluteOffset, Id, RelativeOffset};
use iced::{event, keyboard, mouse, window, Element, Point, Rectangle, Task};

/// Identifier used for the viewer scrollable widget.
pub const SCROLLABLE_ID: &str = "viewer-image-scrollable";

/// Messages emitted by viewer-related widgets.
#[derive(Debug, Clone)]
pub enum Message {
    ImageLoaded(Result<ImageData, Error>),
    ToggleErrorDetails,
    Controls(controls::Message),
    ViewportChanged {
        bounds: Rectangle,
        offset: AbsoluteOffset,
    },
    RawEvent(event::Event),
}

/// Side effects the application should perform after handling a viewer message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    None,
    PersistPreferences,
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
}

#[derive(Default)]
/// Complete viewer component state.
pub struct State {
    image: Option<ImageData>,
    error: Option<ErrorState>,
    pub zoom: ZoomState,
    pub viewport: ViewportState,
    pub drag: DragState,
    cursor_position: Option<Point>,
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has_image(&self) -> bool {
        self.image.is_some()
    }

    pub fn image(&self) -> Option<&ImageData> {
        self.image.as_ref()
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

    pub fn fit_to_window(&self) -> bool {
        self.zoom.fit_to_window
    }

    pub fn enable_fit_to_window(&mut self) {
        self.zoom.enable_fit_to_window();
    }

    pub fn disable_fit_to_window(&mut self) {
        self.zoom.disable_fit_to_window();
    }

    pub fn refresh_error_translation(&mut self, i18n: &I18n) {
        if let Some(error) = &mut self.error {
            error.refresh_translation(i18n);
        }
    }

    pub fn handle_message(&mut self, message: Message, i18n: &I18n) -> (Effect, Task<Message>) {
        match message {
            Message::ImageLoaded(result) => match result {
                Ok(image) => {
                    self.image = Some(image);
                    self.error = None;
                    self.refresh_fit_zoom();
                    (Effect::None, Task::none())
                }
                Err(error) => {
                    self.image = None;
                    self.error = Some(ErrorState::from_error(&error, i18n));
                    (Effect::None, Task::none())
                }
            },
            Message::ToggleErrorDetails => {
                if let Some(error) = &mut self.error {
                    error.show_details = !error.show_details;
                }
                (Effect::None, Task::none())
            }
            Message::Controls(control) => self.handle_controls(control),
            Message::ViewportChanged { bounds, offset } => {
                self.viewport.update(bounds, offset);
                self.refresh_fit_zoom();
                (Effect::None, Task::none())
            }
            Message::RawEvent(event) => self.handle_raw_event(event),
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

        let hud_lines = position_line
            .into_iter()
            .chain(zoom_line.into_iter())
            .collect::<Vec<_>>();

        let image = self.image.as_ref().map(|image_data| viewer::ImageContext {
            controls_context: controls::ViewContext { i18n: env.i18n },
            zoom: &self.zoom,
            pane_context: pane::ViewContext {
                background_theme: env.background_theme,
                hud_lines,
                scrollable_id: SCROLLABLE_ID,
            },
            pane_model: pane::ViewModel {
                image: image_data,
                zoom_percent: self.zoom.zoom_percent,
                padding: geometry_state.image_padding(),
                is_dragging: self.drag.is_dragging,
                cursor_over_image: geometry_state.is_cursor_over_image(),
            },
        });

        viewer::view(viewer::ViewContext {
            i18n: env.i18n,
            error,
            image,
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
                (Effect::PersistPreferences, Task::none())
            }
            ZoomIn => {
                self.zoom
                    .apply_manual_zoom(self.zoom.zoom_percent + self.zoom.zoom_step_percent);
                (Effect::PersistPreferences, Task::none())
            }
            ZoomOut => {
                self.zoom
                    .apply_manual_zoom(self.zoom.zoom_percent - self.zoom.zoom_step_percent);
                (Effect::PersistPreferences, Task::none())
            }
            SetFitToWindow(fit) => {
                if fit {
                    self.zoom.enable_fit_to_window();
                    self.refresh_fit_zoom();
                } else {
                    self.zoom.disable_fit_to_window();
                }
                (Effect::PersistPreferences, Task::none())
            }
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
                    if let Some(position) = self.cursor_position {
                        self.handle_mouse_button_pressed(button, position);
                    }
                    (Effect::None, Task::none())
                }
                mouse::Event::ButtonReleased(button) => {
                    self.handle_mouse_button_released(button);
                    (Effect::None, Task::none())
                }
                mouse::Event::CursorMoved { position } => {
                    self.cursor_position = Some(position);
                    if self.drag.is_dragging {
                        let task = self.handle_cursor_moved_during_drag(position);
                        (Effect::None, task)
                    } else {
                        (Effect::None, Task::none())
                    }
                }
                mouse::Event::CursorLeft => {
                    self.cursor_position = None;
                    if self.drag.is_dragging {
                        self.drag.stop();
                    }
                    (Effect::None, Task::none())
                }
                _ => (Effect::None, Task::none()),
            },
            event::Event::Keyboard(keyboard_event) => {
                if let keyboard::Event::ModifiersChanged(modifiers) = keyboard_event {
                    if modifiers.command() {
                        // no-op currently, but keep placeholder for shortcut support
                    }
                }
                (Effect::None, Task::none())
            }
            _ => (Effect::None, Task::none()),
        }
    }

    fn handle_mouse_button_pressed(&mut self, button: mouse::Button, position: Point) {
        if button == mouse::Button::Left && self.geometry_state().is_cursor_over_image() {
            self.drag.start(position, self.viewport.offset);
        }
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
            (self.viewport.bounds, geometry_state.scaled_image_size())
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
        if !self.geometry_state().is_cursor_over_image() {
            return false;
        }

        let steps = scroll_steps(&delta);
        if steps.abs() < f32::EPSILON {
            return false;
        }

        let new_zoom = self.zoom.zoom_percent + steps * self.zoom.zoom_step_percent;
        self.zoom.apply_manual_zoom(new_zoom);
        true
    }

    /// Recomputes the fit-to-window zoom when layout-affecting events occur so
    /// the zoom textbox always mirrors the actual fit percentage.
    fn refresh_fit_zoom(&mut self) {
        if self.zoom.fit_to_window {
            if let Some(fit_zoom) = self.compute_fit_zoom_percent() {
                self.zoom.update_zoom_display(fit_zoom);
                self.zoom.zoom_input_dirty = false;
                self.zoom.zoom_input_error_key = None;
            }
        }
    }

    /// Calculates the zoom percentage needed to fit the current image inside
    /// the viewport. Returns `None` until viewport bounds are known.
    pub fn compute_fit_zoom_percent(&self) -> Option<f32> {
        let image = self.image.as_ref()?;
        let viewport = self.viewport.bounds?;

        if image.width == 0 || image.height == 0 {
            return Some(crate::ui::state::zoom::DEFAULT_ZOOM_PERCENT);
        }

        if viewport.width <= 0.0 || viewport.height <= 0.0 {
            return None;
        }

        let image_width = image.width as f32;
        let image_height = image.height as f32;

        let scale_x = viewport.width / image_width;
        let scale_y = viewport.height / image_height;

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
            self.image.as_ref(),
            &self.viewport,
            self.zoom.zoom_percent,
            self.cursor_position,
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

fn format_position_indicator(i18n: &I18n, px: f32, py: f32) -> String {
    format!(
        "{}: {:.0}% x {:.0}%",
        i18n.tr("viewer-position-label"),
        px,
        py
    )
}

fn format_zoom_indicator(i18n: &I18n, zoom_percent: f32) -> String {
    format!(
        "{}: {:.0}%",
        i18n.tr("viewer-zoom-indicator-label"),
        zoom_percent
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scroll_indicator_uses_translation() {
        let i18n = I18n::default();
        let position = format_position_indicator(&i18n, 12.4, 56.7);
        let zoom = format_zoom_indicator(&i18n, 135.2);
        assert!(position.starts_with(&i18n.tr("viewer-position-label")));
        assert!(position.contains("12%"));
        assert!(position.contains("57%"));
        assert!(zoom.starts_with(&i18n.tr("viewer-zoom-indicator-label")));
        assert!(zoom.contains("135%"));
    }
}
