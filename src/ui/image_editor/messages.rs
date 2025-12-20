// SPDX-License-Identifier: MPL-2.0
//! Editor message/event types re-exported by the facade.

use crate::media::frame_export::ExportFormat;
use crate::ui::image_editor::{state::CropRatio, EditorTool};
use iced;
use iced::widget::scrollable::AbsoluteOffset;
use iced::Rectangle;
use std::path::PathBuf;

/// Toolbar-specific messages.
#[derive(Debug, Clone)]
pub enum ToolbarMessage {
    BackToViewer,
}

/// Sidebar control messages.
#[derive(Debug, Clone)]
pub enum SidebarMessage {
    ToggleSidebar,
    SelectTool(EditorTool),
    RotateLeft,
    RotateRight,
    FlipHorizontal,
    FlipVertical,
    SetCropRatio(CropRatio),
    ApplyCrop,
    ScaleChanged(f32),
    WidthInputChanged(String),
    HeightInputChanged(String),
    ToggleLockAspect,
    ApplyResizePreset(f32),
    ApplyResize,
    /// Brightness slider changed (live preview)
    BrightnessChanged(i32),
    /// Contrast slider changed (live preview)
    ContrastChanged(i32),
    /// Apply current adjustments to image
    ApplyAdjustments,
    /// Reset adjustments to default
    ResetAdjustments,
    /// Apply AI deblurring to the image
    ApplyDeblur,
    /// Cancel ongoing deblur operation
    CancelDeblur,
    Undo,
    Redo,
    NavigateNext,
    NavigatePrevious,
    Save,
    SaveAs,
    Cancel,
    /// Set the export format for Save As.
    SetExportFormat(ExportFormat),
}

/// Canvas overlay interaction messages.
#[derive(Debug, Clone)]
pub enum CanvasMessage {
    CropOverlayMouseDown {
        x: f32,
        y: f32,
    },
    CropOverlayMouseMove {
        x: f32,
        y: f32,
    },
    CropOverlayMouseUp,
    /// Cursor moved over the canvas area
    CursorMoved {
        position: iced::Point,
    },
    /// Cursor left the canvas area
    CursorLeft,
}

/// Messages emitted directly by the editor widgets.
#[derive(Debug, Clone)]
pub enum Message {
    Toolbar(ToolbarMessage),
    Sidebar(SidebarMessage),
    Canvas(CanvasMessage),
    /// Raw event for keyboard shortcuts
    RawEvent {
        window: iced::window::Id,
        event: iced::Event,
    },
    /// Canvas viewport changed (scrolling or resizing)
    ViewportChanged {
        bounds: Rectangle,
        offset: AbsoluteOffset,
    },
    /// Spinner animation tick (for deblur processing overlay)
    SpinnerTick,
}

impl From<ToolbarMessage> for Message {
    fn from(message: ToolbarMessage) -> Self {
        Message::Toolbar(message)
    }
}

impl From<SidebarMessage> for Message {
    fn from(message: SidebarMessage) -> Self {
        Message::Sidebar(message)
    }
}

impl From<CanvasMessage> for Message {
    fn from(message: CanvasMessage) -> Self {
        Message::Canvas(message)
    }
}

/// Events propagated to the parent application for side effects.
#[derive(Debug, Clone)]
pub enum Event {
    None,
    /// Request to save the edited image
    SaveRequested {
        path: PathBuf,
        overwrite: bool,
    },
    /// Request to open file picker for "Save As"
    SaveAsRequested,
    /// Request to exit editor mode
    ExitEditor,
    /// Request to navigate to next image
    NavigateNext,
    /// Request to navigate to previous image
    NavigatePrevious,
    /// Request to apply AI deblurring to the current image
    DeblurRequested,
    /// Request to cancel ongoing deblur operation
    DeblurCancelRequested,
    /// Request to scroll the canvas to a relative position (for pan)
    ScrollTo {
        /// Relative X offset (0.0 to 1.0)
        x: f32,
        /// Relative Y offset (0.0 to 1.0)
        y: f32,
    },
}
