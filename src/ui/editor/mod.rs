// SPDX-License-Identifier: MPL-2.0
//! Image editor module with rotate, crop, and resize capabilities.
//!
//! This module follows a "state down, messages up" pattern similar to the settings
//! and viewer modules. The editor operates on a copy of the original image and only
//! modifies the source file when the user explicitly saves.

use crate::config::BackgroundTheme;
use crate::error::{Error, Result};
use crate::image_handler::transform;
use crate::image_handler::ImageData;
use crate::ui::theme;
use iced::Rectangle;

mod state;

pub use self::state::{
    CropDragState, CropOverlay, CropRatio, CropState, HandlePosition, ResizeOverlay, ResizeState,
};
use image_rs::DynamicImage;
use std::path::PathBuf;

const ROTATE_LEFT_SVG: &str = r#"<svg viewBox='0 0 24 24' xmlns='http://www.w3.org/2000/svg'>
<path d='M11 5v-3l-4 4 4 4V7c3.309 0 6 2.691 6 6 0 1.262-.389 2.432-1.053 3.403l1.553 1.234C18.42 16.299 19 14.729 19 13c0-4.411-3.589-8-8-8z' fill='currentColor'/>
</svg>"#;

const ROTATE_RIGHT_SVG: &str = r#"<svg viewBox='0 0 24 24' xmlns='http://www.w3.org/2000/svg'>
<path d='M13 5V3l4 4-4 4V7c-3.309 0-6 2.691-6 6 0 1.262.389 2.432 1.053 3.403l-1.553 1.234C5.58 16.299 5 14.729 5 13c0-4.411 3.589-8 8-8z' fill='currentColor'/>
</svg>"#;

const SIDEBAR_WIDTH: f32 = 290.0;

/// Contextual data needed to render the editor view.
pub struct ViewContext<'a> {
    pub i18n: &'a crate::i18n::fluent::I18n,
    pub background_theme: BackgroundTheme,
}

/// Local UI state for the editor screen.
#[derive(Clone)]
pub struct State {
    /// Path to the image being edited
    image_path: PathBuf,
    /// Current edited image (after applying transformations, for display)
    current_image: ImageData,
    /// Working image for transformations (DynamicImage from image_rs crate)
    working_image: DynamicImage,
    /// Currently active editing tool
    active_tool: Option<EditorTool>,
    /// History of transformations for undo/redo
    transformation_history: Vec<Transformation>,
    /// Current position in history (for undo/redo)
    history_index: usize,
    /// Whether the sidebar is expanded
    sidebar_expanded: bool,
    /// Crop tool state
    crop_state: CropState,
    /// Track if crop state has been modified (to avoid auto-commit on tool close)
    crop_modified: bool,
    /// Image state when crop tool was opened (to calculate ratios from original, not from previous crops)
    crop_base_image: Option<DynamicImage>,
    crop_base_width: u32,
    crop_base_height: u32,
    /// Resize state
    resize_state: ResizeState,
    /// Optional preview image (used for live adjustments)
    preview_image: Option<ImageData>,
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State")
            .field("image_path", &self.image_path)
            .field("active_tool", &self.active_tool)
            .field("transformation_history", &self.transformation_history)
            .field("history_index", &self.history_index)
            .field("sidebar_expanded", &self.sidebar_expanded)
            .finish()
    }
}

/// Available editing tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorTool {
    Rotate,
    Crop,
    Resize,
}

/// Image transformations that can be applied and undone.
#[derive(Debug, Clone, PartialEq)]
pub enum Transformation {
    RotateLeft,
    RotateRight,
    Crop { rect: Rectangle },
    Resize { width: u32, height: u32 },
}

/// Messages emitted directly by the editor widgets.
#[derive(Debug, Clone)]
pub enum Message {
    /// Toggle sidebar expanded/collapsed
    ToggleSidebar,
    /// Select an editing tool
    SelectTool(EditorTool),
    /// Apply rotation transformation
    RotateLeft,
    RotateRight,
    /// Crop-related messages
    SetCropRatio(CropRatio),
    UpdateCropSelection(Rectangle),
    ApplyCrop,
    /// Crop overlay interaction messages
    CropOverlayMouseDown {
        x: f32,
        y: f32,
    },
    CropOverlayMouseMove {
        x: f32,
        y: f32,
    },
    CropOverlayMouseUp,
    /// Resize-related messages
    ScaleChanged(f32),
    WidthInputChanged(String),
    HeightInputChanged(String),
    ToggleLockAspect,
    ApplyResizePreset(f32), // Preset percentage (50%, 75%, 150%, 200%)
    ApplyResize,
    /// Undo/redo
    Undo,
    Redo,
    /// Navigation
    NavigateNext,
    NavigatePrevious,
    /// Save/cancel/back
    Save,
    SaveAs,
    Cancel,       // Discard changes but stay in editor
    BackToViewer, // Return to viewer (only if no unsaved changes)
    /// Raw event for keyboard shortcuts
    RawEvent {
        window: iced::window::Id,
        event: iced::Event,
    },
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
}

impl State {
    /// Render the editor view.
    pub fn view<'a>(&'a self, ctx: ViewContext<'a>) -> iced::Element<'a, Message> {
        use iced::widget::{button, center, container, image, text, Column, Row};
        use iced::{Alignment, Background, Border, ContentFit, Length};

        let ViewContext {
            i18n,
            background_theme,
        } = ctx;

        // Top toolbar with "Back to Viewer" button
        let has_changes = self.has_unsaved_changes();
        let back_btn = button(text(format!("← {}", i18n.tr("editor-back-to-viewer"))).size(14))
            .padding([8, 12]);
        let back_btn = if has_changes {
            back_btn // Disabled if unsaved changes
        } else {
            back_btn.on_press(Message::BackToViewer)
        };

        let toolbar = container(
            Row::new()
                .push(back_btn)
                .align_y(Alignment::Center)
                .padding(8),
        )
        .width(Length::Fill)
        .style(|_theme: &iced::Theme| iced::widget::container::Style {
            background: Some(Background::Color(theme::viewer_toolbar_background())),
            border: Border {
                width: 0.0,
                ..Default::default()
            },
            ..Default::default()
        });

        // Main layout: Row with sidebar + image area
        let mut main_row = Row::new().spacing(0);

        // Sidebar (always visible, but can be collapsed in future)
        if self.sidebar_expanded {
            let sidebar = self.build_sidebar(ViewContext {
                i18n,
                background_theme,
            });
            main_row = main_row.push(sidebar);
        } else {
            // Collapsed sidebar - just the hamburger button
            let toggle_button = button(text("☰").size(24))
                .on_press(Message::ToggleSidebar)
                .padding(12);

            let collapsed_bg = theme::viewer_toolbar_background();
            let collapsed_sidebar = container(toggle_button)
                .width(Length::Fixed(60.0))
                .height(Length::Fill)
                .padding(10)
                .style(move |_theme: &iced::Theme| iced::widget::container::Style {
                    background: Some(Background::Color(collapsed_bg)),
                    border: Border {
                        width: 0.0,
                        ..Default::default()
                    },
                    ..Default::default()
                });

            main_row = main_row.push(collapsed_sidebar);
        }

        // Image area with current preview and crop overlay
        let current_display = self.display_image().handle.clone();
        let img_width = self.display_image().width;
        let img_height = self.display_image().height;

        let image_widget = image(current_display.clone()).content_fit(ContentFit::Contain);

        // If crop overlay is visible, wrap in Stack with overlay
        let image_with_overlay: iced::Element<'_, Message> = if self.crop_state.overlay.visible {
            use iced::widget::{Canvas, Stack};

            // Create crop overlay canvas
            let overlay: Canvas<_, Message> = Canvas::new(CropOverlayRenderer {
                crop_x: self.crop_state.x,
                crop_y: self.crop_state.y,
                crop_width: self.crop_state.width,
                crop_height: self.crop_state.height,
                img_width,
                img_height,
            })
            .width(Length::Fill)
            .height(Length::Fill);

            // Stack image and overlay
            Stack::new().push(image_widget).push(overlay).into()
        } else if self.resize_state.overlay.visible {
            use iced::widget::{Canvas, Stack};

            // Create resize overlay canvas
            // Overlay calculates scale based on max(original, new) to fit both rectangles
            let overlay: Canvas<_, Message> = Canvas::new(ResizeOverlayRenderer {
                original_width: self.resize_state.overlay.original_width,
                original_height: self.resize_state.overlay.original_height,
                new_width: self.resize_state.width,
                new_height: self.resize_state.height,
            })
            .width(Length::Fill)
            .height(Length::Fill);

            // Stack image and overlay
            Stack::new().push(image_widget).push(overlay).into()
        } else {
            image_widget.into()
        };

        let build_image_surface = || {
            container(center(image_with_overlay))
                .width(Length::Fill)
                .height(Length::Fill)
        };

        let image_area: iced::Element<'_, Message> = if theme::is_checkerboard(background_theme) {
            theme::wrap_with_checkerboard(build_image_surface())
        } else {
            let bg_color = match background_theme {
                BackgroundTheme::Light => theme::viewer_light_surface_color(),
                BackgroundTheme::Dark => theme::viewer_dark_surface_color(),
                BackgroundTheme::Checkerboard => unreachable!(),
            };

            build_image_surface()
                .style(move |_theme: &iced::Theme| iced::widget::container::Style {
                    background: Some(Background::Color(bg_color)),
                    ..Default::default()
                })
                .into()
        };

        main_row = main_row.push(image_area);

        // Combine toolbar and main content in a Column
        let content = Column::new().push(toolbar).push(main_row);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Build the expanded sidebar with tools and controls.
    fn build_sidebar<'a>(&'a self, ctx: ViewContext<'a>) -> iced::Element<'a, Message> {
        use iced::widget::scrollable::{Direction, Scrollbar};
        use iced::widget::{button, container, svg, text, tooltip, Column, Row, Scrollable};
        use iced::{alignment::Vertical, Background, Border, Length};

        let mut header_section = Column::new().spacing(8);

        // Hamburger toggle button
        let toggle_button = button(text("☰").size(20))
            .on_press(Message::ToggleSidebar)
            .padding(8)
            .style(iced::widget::button::secondary);

        header_section = header_section.push(
            Row::new()
                .push(toggle_button)
                .push(text("Tools").size(18))
                .spacing(8)
                .align_y(Vertical::Center),
        );

        header_section = header_section.push(iced::widget::horizontal_rule(1));

        // Undo/Redo buttons
        let undo_btn = button(text(ctx.i18n.tr("editor-undo")).size(16))
            .padding(8)
            .width(Length::Fill)
            .style(iced::widget::button::secondary);
        let undo_btn = if self.can_undo() {
            undo_btn.on_press(Message::Undo)
        } else {
            undo_btn
        };

        let redo_btn = button(text(ctx.i18n.tr("editor-redo")).size(16))
            .padding(8)
            .width(Length::Fill)
            .style(iced::widget::button::secondary);
        let redo_btn = if self.can_redo() {
            redo_btn.on_press(Message::Redo)
        } else {
            redo_btn
        };

        let undo_redo_row = Row::new().spacing(8).push(undo_btn).push(redo_btn);

        let undo_redo_section_title = text(ctx.i18n.tr("editor-undo-redo-section-title")).size(14);

        let undo_redo_section = container(
            Column::new()
                .spacing(6)
                .push(undo_redo_section_title)
                .push(undo_redo_row),
        )
        .padding(12)
        .width(Length::Fill)
        .style(theme::settings_panel_style);

        let mut scrollable_section = Column::new().spacing(8);

        scrollable_section = scrollable_section.push(undo_redo_section);

        scrollable_section = scrollable_section.push(iced::widget::horizontal_rule(1));

        // Rotate tools
        let rotate_left_icon = svg::Svg::new(svg::Handle::from_memory(ROTATE_LEFT_SVG.as_bytes()))
            .width(Length::Fixed(28.0))
            .height(Length::Fixed(28.0));

        let rotate_right_icon =
            svg::Svg::new(svg::Handle::from_memory(ROTATE_RIGHT_SVG.as_bytes()))
                .width(Length::Fixed(28.0))
                .height(Length::Fixed(28.0));

        let rotate_left_btn = tooltip::Tooltip::new(
            button(rotate_left_icon)
                .on_press(Message::RotateLeft)
                .padding(8)
                .width(Length::Fill)
                .style(iced::widget::button::secondary),
            text(ctx.i18n.tr("editor-rotate-left-tooltip")),
            tooltip::Position::FollowCursor,
        )
        .gap(4)
        .padding(6);

        let rotate_right_btn = tooltip::Tooltip::new(
            button(rotate_right_icon)
                .on_press(Message::RotateRight)
                .padding(8)
                .width(Length::Fill)
                .style(iced::widget::button::secondary),
            text(ctx.i18n.tr("editor-rotate-right-tooltip")),
            tooltip::Position::FollowCursor,
        )
        .gap(4)
        .padding(6);

        let rotate_row = Row::new()
            .spacing(8)
            .push(rotate_left_btn)
            .push(rotate_right_btn);

        let rotate_section_title = text(ctx.i18n.tr("editor-rotate-section-title")).size(14);

        let rotate_section = container(
            Column::new()
                .spacing(6)
                .push(rotate_section_title)
                .push(rotate_row),
        )
        .padding(12)
        .width(Length::Fill)
        .style(theme::settings_panel_style);

        scrollable_section = scrollable_section.push(rotate_section);

        scrollable_section = scrollable_section.push(iced::widget::horizontal_rule(1));

        // Main tool buttons
        let crop_btn = button(text(ctx.i18n.tr("editor-tool-crop")).size(16))
            .on_press(Message::SelectTool(EditorTool::Crop))
            .padding(12)
            .width(Length::Fill)
            .style(if self.active_tool == Some(EditorTool::Crop) {
                iced::widget::button::primary
            } else {
                iced::widget::button::secondary
            });

        let resize_btn = button(text(ctx.i18n.tr("editor-tool-resize")).size(16))
            .on_press(Message::SelectTool(EditorTool::Resize))
            .padding(12)
            .width(Length::Fill)
            .style(if self.active_tool == Some(EditorTool::Resize) {
                iced::widget::button::primary
            } else {
                iced::widget::button::secondary
            });

        scrollable_section = scrollable_section.push(crop_btn);

        // Show crop panel immediately below crop button when active
        if self.active_tool == Some(EditorTool::Crop) {
            scrollable_section = scrollable_section.push(self.build_crop_panel(&ctx));
        }

        scrollable_section = scrollable_section.push(resize_btn);

        // Show resize panel immediately below resize button when active
        if self.active_tool == Some(EditorTool::Resize) {
            scrollable_section = scrollable_section.push(self.build_resize_panel(&ctx));
        }

        let scrollable_content = Scrollable::new(scrollable_section)
            .direction(Direction::Vertical(Scrollbar::new()))
            .height(Length::Fill)
            .width(Length::Fill);

        let mut footer_section = Column::new().spacing(8);

        footer_section = footer_section.push(iced::widget::horizontal_rule(1));

        // Navigation arrows (disabled if there are unsaved changes)
        let has_changes = self.has_unsaved_changes();

        let prev_btn = button(text("◀").size(20))
            .padding([8, 16])
            .width(Length::Fill);
        let prev_btn = if has_changes {
            prev_btn // No on_press = disabled appearance
        } else {
            prev_btn.on_press(Message::NavigatePrevious)
        };

        let next_btn = button(text("▶").size(20))
            .padding([8, 16])
            .width(Length::Fill);
        let next_btn = if has_changes {
            next_btn // No on_press = disabled appearance
        } else {
            next_btn.on_press(Message::NavigateNext)
        };

        let nav_row = Row::new().spacing(8).push(prev_btn).push(next_btn);

        footer_section = footer_section.push(nav_row);

        footer_section = footer_section.push(iced::widget::horizontal_rule(1));

        // Action buttons (Cancel/Save/Save As)
        let cancel_btn = button(text(ctx.i18n.tr("editor-cancel")).size(16))
            .padding(12)
            .width(Length::Fill)
            .style(iced::widget::button::secondary);
        let cancel_btn = if has_changes {
            cancel_btn.on_press(Message::Cancel) // Active if changes exist
        } else {
            cancel_btn // Disabled if no changes
        };

        let save_btn = button(text(ctx.i18n.tr("editor-save")).size(16))
            .padding(12)
            .width(Length::Fill)
            .style(iced::widget::button::primary);
        let save_btn = if has_changes {
            save_btn.on_press(Message::Save) // Active if changes exist
        } else {
            save_btn // Disabled if no changes
        };

        let save_as_btn = button(text(ctx.i18n.tr("editor-save-as")).size(16))
            .padding(12)
            .width(Length::Fill)
            .style(iced::widget::button::secondary);
        let save_as_btn = if has_changes {
            save_as_btn.on_press(Message::SaveAs) // Active if changes exist
        } else {
            save_as_btn // Disabled if no changes
        };

        footer_section = footer_section.push(cancel_btn);
        footer_section = footer_section.push(save_btn);
        footer_section = footer_section.push(save_as_btn);

        let sidebar_stack = Column::new()
            .spacing(8)
            .padding(12)
            .width(Length::Fixed(SIDEBAR_WIDTH))
            .push(header_section)
            .push(scrollable_content)
            .push(footer_section);

        // Container with the same background as the viewer toolbar for visual continuity
        container(sidebar_stack)
            .width(Length::Fixed(SIDEBAR_WIDTH))
            .height(Length::Fill)
            .style(|_theme: &iced::Theme| iced::widget::container::Style {
                background: Some(Background::Color(theme::viewer_toolbar_background())),
                border: Border {
                    width: 0.0,
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
    }

    fn build_resize_panel<'a>(&'a self, ctx: &ViewContext<'a>) -> iced::Element<'a, Message> {
        use iced::widget::{button, checkbox, container, slider, text, text_input, Column, Row};
        use iced::Length;

        let scale_section = Column::new()
            .spacing(6)
            .push(text(ctx.i18n.tr("editor-resize-section-title")).size(14))
            .push(text(ctx.i18n.tr("editor-resize-scale-label")).size(13))
            .push(
                slider(
                    10.0..=200.0,
                    self.resize_state.scale_percent,
                    Message::ScaleChanged,
                )
                .step(1.0),
            )
            .push(text(format!("{:.0}%", self.resize_state.scale_percent)).size(13));

        let mut presets_row = Row::new().spacing(8);
        for preset in [50.0, 75.0, 150.0, 200.0] {
            let label = format!("{preset:.0}%");
            presets_row = presets_row.push(
                button(text(label))
                    .on_press(Message::ApplyResizePreset(preset))
                    .padding([6, 8])
                    .style(iced::widget::button::secondary),
            );
        }

        let presets_section = Column::new()
            .spacing(6)
            .push(text(ctx.i18n.tr("editor-resize-presets-label")).size(13))
            .push(presets_row);

        let width_label = text(ctx.i18n.tr("editor-resize-width-label")).size(13);
        let width_placeholder = ctx.i18n.tr("editor-resize-width-label");
        let width_input = text_input(width_placeholder.as_str(), &self.resize_state.width_input)
            .on_input(Message::WidthInputChanged)
            .padding(6)
            .size(14)
            .width(Length::Fill);

        let height_label = text(ctx.i18n.tr("editor-resize-height-label")).size(13);
        let height_placeholder = ctx.i18n.tr("editor-resize-height-label");
        let height_input = text_input(height_placeholder.as_str(), &self.resize_state.height_input)
            .on_input(Message::HeightInputChanged)
            .padding(6)
            .size(14)
            .width(Length::Fill);

        let width_column = Column::new()
            .spacing(4)
            .width(Length::Fill)
            .push(width_label)
            .push(width_input);

        let height_column = Column::new()
            .spacing(4)
            .width(Length::Fill)
            .push(height_label)
            .push(height_input);

        let dimensions_row = Row::new().spacing(8).push(width_column).push(height_column);

        let lock_checkbox = checkbox(
            ctx.i18n.tr("editor-resize-lock-aspect"),
            self.resize_state.lock_aspect,
        )
        .on_toggle(|_| Message::ToggleLockAspect);

        // Apply button
        let apply_btn = button(text(ctx.i18n.tr("editor-resize-apply")).size(16))
            .padding(10)
            .width(Length::Fill)
            .style(iced::widget::button::primary)
            .on_press(Message::ApplyResize);

        container(
            Column::new()
                .spacing(12)
                .push(scale_section)
                .push(presets_section)
                .push(text(ctx.i18n.tr("editor-resize-dimensions-label")).size(13))
                .push(dimensions_row)
                .push(lock_checkbox)
                .push(apply_btn),
        )
        .padding(12)
        .width(Length::Fill)
        .style(theme::settings_panel_style)
        .into()
    }

    fn build_crop_panel<'a>(&'a self, ctx: &ViewContext<'a>) -> iced::Element<'a, Message> {
        use iced::widget::{container, text, Column, Row};
        use iced::Length;

        let title = text(ctx.i18n.tr("editor-crop-section-title")).size(14);

        let ratio_label = text(ctx.i18n.tr("editor-crop-ratio-label")).size(13);

        // Build aspect ratio buttons in rows
        let ratio_buttons_row1 =
            Row::new()
                .spacing(4)
                .push(self.build_crop_ratio_button(
                    ctx.i18n.tr("editor-crop-ratio-free"),
                    CropRatio::Free,
                ))
                .push(self.build_crop_ratio_button(
                    ctx.i18n.tr("editor-crop-ratio-square"),
                    CropRatio::Square,
                ));

        let ratio_buttons_row2 = Row::new()
            .spacing(4)
            .push(self.build_crop_ratio_button(
                ctx.i18n.tr("editor-crop-ratio-landscape"),
                CropRatio::Landscape,
            ))
            .push(self.build_crop_ratio_button(
                ctx.i18n.tr("editor-crop-ratio-portrait"),
                CropRatio::Portrait,
            ));

        let ratio_buttons_row3 =
            Row::new()
                .spacing(4)
                .push(self.build_crop_ratio_button(
                    ctx.i18n.tr("editor-crop-ratio-photo"),
                    CropRatio::Photo,
                ))
                .push(self.build_crop_ratio_button(
                    ctx.i18n.tr("editor-crop-ratio-photo-portrait"),
                    CropRatio::PhotoPortrait,
                ));

        let crop_info = text(format!(
            "{}×{} px",
            self.crop_state.width, self.crop_state.height
        ))
        .size(12);

        // Apply crop button (only enabled when overlay is visible)
        let apply_btn = {
            use iced::widget::button;
            let btn = button(text(ctx.i18n.tr("editor-crop-apply")).size(14))
                .padding(8)
                .width(Length::Fill)
                .style(iced::widget::button::primary);

            if self.crop_state.overlay.visible {
                btn.on_press(Message::ApplyCrop)
            } else {
                btn
            }
        };

        container(
            Column::new()
                .spacing(8)
                .push(title)
                .push(ratio_label)
                .push(ratio_buttons_row1)
                .push(ratio_buttons_row2)
                .push(ratio_buttons_row3)
                .push(crop_info)
                .push(apply_btn),
        )
        .padding(12)
        .width(Length::Fill)
        .style(theme::settings_panel_style)
        .into()
    }

    fn build_crop_ratio_button<'a>(
        &'a self,
        label: String,
        ratio: CropRatio,
    ) -> iced::Element<'a, Message> {
        use iced::widget::{button, text};
        use iced::Length;

        let is_active = self.crop_state.ratio == ratio;
        let btn = button(text(label).size(11))
            .on_press(Message::SetCropRatio(ratio))
            .padding([4, 6])
            .width(Length::Fill)
            .style(if is_active {
                iced::widget::button::primary
            } else {
                iced::widget::button::secondary
            });

        btn.into()
    }

    /// Update the state and emit an [`Event`] for the parent when needed.
    pub fn update(&mut self, message: Message) -> Event {
        match message {
            Message::ToggleSidebar => {
                self.sidebar_expanded = !self.sidebar_expanded;
                Event::None
            }
            Message::SelectTool(tool) => {
                if self.active_tool == Some(tool) {
                    self.commit_active_tool_changes();
                    self.active_tool = None;
                    self.preview_image = None;
                    // Reset crop modified flag, hide overlay and clear base image when closing crop tool
                    if tool == EditorTool::Crop {
                        self.crop_modified = false;
                        self.crop_base_image = None;
                        self.crop_state.overlay.visible = false;
                        self.crop_state.overlay.drag_state = CropDragState::None;
                    }
                    // Hide resize overlay when closing resize tool
                    if tool == EditorTool::Resize {
                        self.resize_state.overlay.visible = false;
                    }
                } else {
                    self.commit_active_tool_changes();
                    // Hide overlay when leaving crop tool
                    if self.active_tool == Some(EditorTool::Crop) {
                        self.crop_state.overlay.visible = false;
                        self.crop_state.overlay.drag_state = CropDragState::None;
                    }
                    // Hide resize overlay when leaving resize tool
                    if self.active_tool == Some(EditorTool::Resize) {
                        self.resize_state.overlay.visible = false;
                    }
                    self.active_tool = Some(tool);
                    // Clear preview when switching tools
                    self.preview_image = None;

                    // When opening crop tool, memorize current image state
                    if tool == EditorTool::Crop {
                        self.crop_base_image = Some(self.working_image.clone());
                        self.crop_base_width = self.current_image.width;
                        self.crop_base_height = self.current_image.height;
                        // Initialize crop rectangle to full image size
                        self.crop_state.x = 0;
                        self.crop_state.y = 0;
                        self.crop_state.width = self.current_image.width;
                        self.crop_state.height = self.current_image.height;
                        self.crop_state.ratio = CropRatio::None;
                        // Don't show overlay until user selects a ratio
                        self.crop_state.overlay.visible = false;
                    }

                    // When opening resize tool, show overlay with current image dimensions as baseline
                    if tool == EditorTool::Resize {
                        self.resize_state.overlay.visible = true;
                        self.resize_state.overlay.set_original_dimensions(
                            self.current_image.width,
                            self.current_image.height,
                        );
                    }
                }
                Event::None
            }
            Message::RotateLeft => {
                self.commit_active_tool_changes();
                self.apply_dynamic_transformation(
                    Transformation::RotateLeft,
                    transform::rotate_left,
                );
                Event::None
            }
            Message::RotateRight => {
                self.commit_active_tool_changes();
                self.apply_dynamic_transformation(
                    Transformation::RotateRight,
                    transform::rotate_right,
                );
                Event::None
            }
            Message::SetCropRatio(ratio) => {
                self.crop_state.ratio = ratio;
                self.adjust_crop_to_ratio(ratio);
                // Show the overlay so user can position the crop
                self.crop_state.overlay.visible = true;
                self.crop_modified = true;
                Event::None
            }
            Message::UpdateCropSelection(_rect) => {
                // TODO: Implement interactive crop selection with handles
                Event::None
            }
            Message::ApplyCrop => {
                // Apply the crop from the overlay
                if self.crop_state.overlay.visible {
                    self.apply_crop_from_base();
                    self.crop_state.overlay.visible = false;
                    self.crop_state.overlay.drag_state = CropDragState::None;
                    self.crop_modified = false;
                    // Reset to None (deselect all ratio buttons)
                    self.crop_state.ratio = CropRatio::None;
                    // Reset crop rectangle to full new image size for next crop
                    self.crop_state.x = 0;
                    self.crop_state.y = 0;
                    self.crop_state.width = self.current_image.width;
                    self.crop_state.height = self.current_image.height;
                    // Update base image to the newly cropped image
                    self.crop_base_image = Some(self.working_image.clone());
                    self.crop_base_width = self.current_image.width;
                    self.crop_base_height = self.current_image.height;
                    // Overlay stays hidden until user selects a ratio
                }
                Event::None
            }
            Message::CropOverlayMouseDown { x, y } => {
                self.handle_crop_overlay_mouse_down(x, y);
                Event::None
            }
            Message::CropOverlayMouseMove { x, y } => {
                self.handle_crop_overlay_mouse_move(x, y);
                Event::None
            }
            Message::CropOverlayMouseUp => {
                self.crop_state.overlay.drag_state = CropDragState::None;
                Event::None
            }
            Message::ScaleChanged(percent) => {
                self.set_resize_percent(percent);
                Event::None
            }
            Message::WidthInputChanged(value) => {
                self.handle_width_input_change(value);
                Event::None
            }
            Message::HeightInputChanged(value) => {
                self.handle_height_input_change(value);
                Event::None
            }
            Message::ToggleLockAspect => {
                self.toggle_resize_lock();
                Event::None
            }
            Message::ApplyResizePreset(percent) => {
                self.set_resize_percent(percent);
                Event::None
            }
            Message::ApplyResize => {
                // Apply the resize transformation
                self.apply_resize_dimensions();
                Event::None
            }
            Message::Undo => {
                self.commit_active_tool_changes();
                if self.can_undo() {
                    self.history_index -= 1;
                    self.replay_transformations_up_to_index();
                }
                Event::None
            }
            Message::Redo => {
                self.commit_active_tool_changes();
                if self.can_redo() {
                    self.history_index += 1;
                    self.replay_transformations_up_to_index();
                }
                Event::None
            }
            Message::NavigateNext => {
                // Block navigation if there are unsaved changes
                if self.has_unsaved_changes() {
                    Event::None
                } else {
                    self.commit_active_tool_changes();
                    Event::NavigateNext
                }
            }
            Message::NavigatePrevious => {
                // Block navigation if there are unsaved changes
                if self.has_unsaved_changes() {
                    Event::None
                } else {
                    self.commit_active_tool_changes();
                    Event::NavigatePrevious
                }
            }
            Message::Save => {
                self.commit_active_tool_changes();
                // Save overwrites the original file (confirmation may be added later)
                Event::SaveRequested {
                    path: self.image_path.clone(),
                    overwrite: true,
                }
            }
            Message::SaveAs => {
                self.commit_active_tool_changes();
                // Request file picker dialog from parent
                Event::SaveAsRequested
            }
            Message::Cancel => {
                // Discard all changes but STAY in editor
                self.discard_changes();
                Event::None
            }
            Message::BackToViewer => {
                // Return to viewer (only allowed if no unsaved changes)
                if self.has_unsaved_changes() {
                    Event::None // Blocked
                } else {
                    Event::ExitEditor
                }
            }
            Message::RawEvent { event, .. } => {
                use iced::keyboard;

                match event {
                    iced::Event::Keyboard(keyboard::Event::KeyPressed {
                        key: keyboard::Key::Named(keyboard::key::Named::Escape),
                        ..
                    }) => {
                        // Esc: Cancel if has changes, otherwise exit editor
                        if self.has_unsaved_changes() {
                            self.discard_changes();
                            Event::None
                        } else {
                            Event::ExitEditor
                        }
                    }
                    iced::Event::Keyboard(keyboard::Event::KeyPressed {
                        key, modifiers, ..
                    }) if modifiers.command() => {
                        // Ctrl+S, Ctrl+Z, Ctrl+Y (or Cmd on macOS)
                        match key {
                            keyboard::Key::Character(ref c) if c.as_str() == "s" => {
                                // Ctrl+S: Save
                                if self.has_unsaved_changes() {
                                    Event::SaveRequested {
                                        path: self.image_path.clone(),
                                        overwrite: true,
                                    }
                                } else {
                                    Event::None
                                }
                            }
                            keyboard::Key::Character(ref c) if c.as_str() == "z" => {
                                // Ctrl+Z: Undo
                                self.commit_active_tool_changes();
                                if self.can_undo() {
                                    self.history_index -= 1;
                                    self.replay_transformations_up_to_index();
                                }
                                Event::None
                            }
                            keyboard::Key::Character(ref c) if c.as_str() == "y" => {
                                // Ctrl+Y: Redo
                                self.commit_active_tool_changes();
                                if self.can_redo() {
                                    self.history_index += 1;
                                    self.replay_transformations_up_to_index();
                                }
                                Event::None
                            }
                            _ => Event::None,
                        }
                    }
                    _ => Event::None,
                }
            }
        }
    }

    /// Create a new editor state for the given image.
    pub fn new(image_path: PathBuf, image: ImageData) -> Result<Self> {
        let working_image =
            image_rs::open(&image_path).map_err(|err| Error::Io(err.to_string()))?;

        Ok(Self {
            image_path,
            current_image: image.clone(),
            working_image,
            active_tool: None,
            transformation_history: Vec::new(),
            history_index: 0,
            sidebar_expanded: true,
            crop_state: CropState::from_image(&image),
            crop_modified: false,
            resize_state: ResizeState::from_image(&image),
            crop_base_image: None,
            crop_base_width: image.width,
            crop_base_height: image.height,
            preview_image: None,
        })
    }
    /// Check if there are unsaved changes based on transformation history.
    pub fn has_unsaved_changes(&self) -> bool {
        !self.transformation_history.is_empty()
    }

    /// Check if undo is available.
    pub fn can_undo(&self) -> bool {
        self.history_index > 0
    }

    /// Check if redo is available.
    pub fn can_redo(&self) -> bool {
        self.history_index < self.transformation_history.len()
    }

    /// Save the edited image to a file, preserving the original format.
    pub fn save_image(&mut self, path: &std::path::Path) -> Result<()> {
        use image_rs::ImageFormat;

        // Detect format from file extension
        let format = match path.extension().and_then(|s| s.to_str()) {
            Some("jpg") | Some("jpeg") => ImageFormat::Jpeg,
            Some("png") => ImageFormat::Png,
            Some("gif") => ImageFormat::Gif,
            Some("bmp") => ImageFormat::Bmp,
            Some("ico") => ImageFormat::Ico,
            Some("tiff") | Some("tif") => ImageFormat::Tiff,
            Some("webp") => ImageFormat::WebP,
            _ => ImageFormat::Png, // Default fallback
        };

        // Save the working image
        self.working_image
            .save_with_format(path, format)
            .map_err(|err| Error::Io(format!("Failed to save image: {}", err)))?;

        // Clear transformation history after successful save
        self.transformation_history.clear();
        self.history_index = 0;

        Ok(())
    }

    /// Get the current image data.
    pub fn current_image(&self) -> &ImageData {
        &self.current_image
    }

    /// Get the image file path.
    pub fn image_path(&self) -> &std::path::Path {
        &self.image_path
    }

    fn display_image(&self) -> &ImageData {
        self.preview_image.as_ref().unwrap_or(&self.current_image)
    }

    /// Get the active tool.
    pub fn active_tool(&self) -> Option<EditorTool> {
        self.active_tool
    }

    /// Check if sidebar is expanded.
    pub fn is_sidebar_expanded(&self) -> bool {
        self.sidebar_expanded
    }

    fn apply_dynamic_transformation<F>(&mut self, transformation: Transformation, operation: F)
    where
        F: Fn(&DynamicImage) -> DynamicImage,
    {
        let updated = operation(&self.working_image);
        match transform::dynamic_to_image_data(&updated) {
            Ok(image_data) => {
                self.working_image = updated;
                self.current_image = image_data;
                self.sync_resize_state_dimensions();
                self.preview_image = None;
                self.record_transformation(transformation);
            }
            Err(err) => {
                eprintln!("Failed to apply transformation: {err:?}");
            }
        }
    }

    fn sync_resize_state_dimensions(&mut self) {
        self.resize_state.sync_from_image(&self.current_image);
    }

    fn record_transformation(&mut self, transformation: Transformation) {
        if self.history_index < self.transformation_history.len() {
            self.transformation_history.truncate(self.history_index);
        }
        self.transformation_history.push(transformation);
        self.history_index = self.transformation_history.len();
    }

    fn base_width(&self) -> f32 {
        self.current_image.width.max(1) as f32
    }

    fn base_height(&self) -> f32 {
        self.current_image.height.max(1) as f32
    }

    fn commit_active_tool_changes(&mut self) {
        if matches!(self.active_tool, Some(EditorTool::Crop))
            && self.crop_modified
            && self.crop_state.overlay.visible
        {
            self.finalize_crop_overlay();
        }
    }

    fn set_resize_percent(&mut self, percent: f32) {
        let clamped = percent.clamp(10.0, 200.0);
        self.resize_state.scale_percent = clamped;
        let width = (self.base_width() * clamped / 100.0).round().max(1.0) as u32;
        let height = (self.base_height() * clamped / 100.0).round().max(1.0) as u32;

        if self.resize_state.lock_aspect {
            self.set_width_preserving_aspect(width);
        } else {
            self.resize_state.width = width;
            self.resize_state.height = height;
            self.resize_state.width_input = width.to_string();
            self.resize_state.height_input = height.to_string();
        }

        self.update_resize_preview();
    }

    fn handle_width_input_change(&mut self, value: String) {
        self.resize_state.width_input = value.clone();
        if let Some(width) = parse_dimension_input(&value) {
            if self.resize_state.lock_aspect {
                self.set_width_preserving_aspect(width);
            } else {
                let width = width.max(1);
                self.resize_state.width = width;
                self.resize_state.width_input = width.to_string();
            }
            self.update_scale_percent_from_width();
        }
    }

    fn handle_height_input_change(&mut self, value: String) {
        self.resize_state.height_input = value.clone();
        if let Some(height) = parse_dimension_input(&value) {
            if self.resize_state.lock_aspect {
                self.set_height_preserving_aspect(height);
                self.update_scale_percent_from_width();
            } else {
                let height = height.max(1);
                self.resize_state.height = height;
                self.resize_state.height_input = height.to_string();
            }
            self.update_resize_preview();
        }
    }

    fn toggle_resize_lock(&mut self) {
        self.resize_state.lock_aspect = !self.resize_state.lock_aspect;
        if self.resize_state.lock_aspect {
            let width = self.resize_state.width;
            self.set_width_preserving_aspect(width);
        }
        self.update_resize_preview();
    }

    fn set_width_preserving_aspect(&mut self, width: u32) {
        let width = width.max(1);
        let aspect = self.resize_state.original_aspect.max(f32::EPSILON);
        let height = (width as f32 / aspect).round().max(1.0) as u32;
        self.resize_state.width = width;
        self.resize_state.height = height;
        self.resize_state.width_input = width.to_string();
        self.resize_state.height_input = height.to_string();
    }

    fn set_height_preserving_aspect(&mut self, height: u32) {
        let height = height.max(1);
        let aspect = self.resize_state.original_aspect.max(f32::EPSILON);
        let width = (height as f32 * aspect).round().max(1.0) as u32;
        self.resize_state.height = height;
        self.resize_state.width = width.max(1);
        self.resize_state.width_input = self.resize_state.width.to_string();
        self.resize_state.height_input = height.to_string();
    }

    fn update_scale_percent_from_width(&mut self) {
        let base_width = self.base_width();
        if base_width <= 0.0 {
            return;
        }
        let percent = (self.resize_state.width as f32 / base_width) * 100.0;
        let clamped = percent.clamp(10.0, 200.0);
        if (clamped - percent).abs() > f32::EPSILON {
            self.set_resize_percent(clamped);
        } else {
            self.resize_state.scale_percent = clamped;
            self.update_resize_preview();
        }
    }

    fn apply_resize_dimensions(&mut self) {
        let target_width = self.resize_state.width.max(1);
        let target_height = self.resize_state.height.max(1);
        if target_width == self.current_image.width && target_height == self.current_image.height {
            return;
        }

        self.apply_dynamic_transformation(
            Transformation::Resize {
                width: target_width,
                height: target_height,
            },
            move |image| transform::resize(image, target_width, target_height),
        );

        self.resize_state
            .overlay
            .set_original_dimensions(self.current_image.width, self.current_image.height);
    }

    fn update_resize_preview(&mut self) {
        // Don't generate preview when overlay is visible - the overlay will show the preview
        if self.resize_state.overlay.visible {
            self.preview_image = None;
            return;
        }

        let target_width = self.resize_state.width.max(1);
        let target_height = self.resize_state.height.max(1);
        if target_width == self.current_image.width && target_height == self.current_image.height {
            self.preview_image = None;
            return;
        }

        let preview_dynamic = transform::resize(&self.working_image, target_width, target_height);
        match transform::dynamic_to_image_data(&preview_dynamic) {
            Ok(image_data) => {
                self.preview_image = Some(image_data);
            }
            Err(err) => {
                eprintln!("Failed to build resize preview: {err:?}");
                self.preview_image = None;
            }
        }
    }

    fn adjust_crop_to_ratio(&mut self, ratio: CropRatio) {
        // Use base image dimensions (image when crop tool was opened), not current image
        let img_width = self.crop_base_width as f32;
        let img_height = self.crop_base_height as f32;

        let (new_width, new_height) = match ratio {
            CropRatio::None | CropRatio::Free => {
                // No adjustment needed
                return;
            }
            CropRatio::Square => {
                // 1:1 - make square, use smaller dimension
                let size = img_width.min(img_height);
                (size, size)
            }
            CropRatio::Landscape => {
                // 16:9
                let height = img_width * 9.0 / 16.0;
                if height <= img_height {
                    (img_width, height)
                } else {
                    let width = img_height * 16.0 / 9.0;
                    (width, img_height)
                }
            }
            CropRatio::Portrait => {
                // 9:16
                let width = img_height * 9.0 / 16.0;
                if width <= img_width {
                    (width, img_height)
                } else {
                    let height = img_width * 16.0 / 9.0;
                    (img_width, height)
                }
            }
            CropRatio::Photo => {
                // 4:3
                let height = img_width * 3.0 / 4.0;
                if height <= img_height {
                    (img_width, height)
                } else {
                    let width = img_height * 4.0 / 3.0;
                    (width, img_height)
                }
            }
            CropRatio::PhotoPortrait => {
                // 3:4
                let width = img_height * 3.0 / 4.0;
                if width <= img_width {
                    (width, img_height)
                } else {
                    let height = img_width * 4.0 / 3.0;
                    (img_width, height)
                }
            }
        };

        let new_width = new_width.round() as u32;
        let new_height = new_height.round() as u32;

        // Center the crop area (using base image dimensions)
        self.crop_state.width = new_width;
        self.crop_state.height = new_height;
        self.crop_state.x = (self.crop_base_width - new_width) / 2;
        self.crop_state.y = (self.crop_base_height - new_height) / 2;
    }

    fn apply_crop_from_base(&mut self) {
        // Apply crop from the base image (image when crop tool was opened)
        let Some(ref base_image) = self.crop_base_image else {
            eprintln!("No base image available for crop");
            return;
        };

        let x = self.crop_state.x;
        let y = self.crop_state.y;
        let width = self.crop_state.width;
        let height = self.crop_state.height;

        // Validate crop bounds
        if width == 0 || height == 0 || x >= self.crop_base_width || y >= self.crop_base_height {
            eprintln!("Invalid crop bounds: ({}, {}, {}×{})", x, y, width, height);
            return;
        }

        // Apply crop transformation from base image
        if let Some(cropped) = transform::crop(base_image, x, y, width, height) {
            match transform::dynamic_to_image_data(&cropped) {
                Ok(image_data) => {
                    self.working_image = cropped;
                    self.current_image = image_data;
                    self.sync_resize_state_dimensions();

                    // Record transformation for undo/redo
                    self.record_transformation(Transformation::Crop {
                        rect: Rectangle {
                            x: x as f32,
                            y: y as f32,
                            width: width as f32,
                            height: height as f32,
                        },
                    });

                    // Note: Do NOT update crop_base_image here!
                    // All crops within the same session should be relative to the base image
                    // captured when the Crop tool was opened. The base is only updated when
                    // the user closes and reopens the Crop tool.
                }
                Err(err) => {
                    eprintln!("Failed to convert cropped image: {err:?}");
                }
            }
        } else {
            eprintln!("Crop operation returned None");
        }
    }

    fn finalize_crop_overlay(&mut self) {
        if !self.crop_state.overlay.visible {
            return;
        }

        self.apply_crop_from_base();
        self.crop_state.overlay.visible = false;
        self.crop_state.overlay.drag_state = CropDragState::None;
        self.crop_modified = false;
        self.crop_state.ratio = CropRatio::None;
        self.crop_state.x = 0;
        self.crop_state.y = 0;
        self.crop_state.width = self.current_image.width;
        self.crop_state.height = self.current_image.height;
        self.crop_base_image = Some(self.working_image.clone());
        self.crop_base_width = self.current_image.width;
        self.crop_base_height = self.current_image.height;
    }

    /// Handle mouse down on crop overlay to start dragging
    fn handle_crop_overlay_mouse_down(&mut self, x: f32, y: f32) {
        // Check if clicking on a handle
        if let Some(handle) = self.get_handle_at_position(x, y) {
            self.crop_state.overlay.drag_state = CropDragState::DraggingHandle {
                handle,
                start_rect: (
                    self.crop_state.x,
                    self.crop_state.y,
                    self.crop_state.width,
                    self.crop_state.height,
                ),
                start_cursor_x: x,
                start_cursor_y: y,
            };
        } else if self.is_point_in_crop_rect(x, y) {
            // Start dragging the entire rectangle
            self.crop_state.overlay.drag_state = CropDragState::DraggingRectangle {
                start_rect_x: self.crop_state.x,
                start_rect_y: self.crop_state.y,
                start_cursor_x: x,
                start_cursor_y: y,
            };
        }
    }

    /// Handle mouse move on crop overlay to update drag
    fn handle_crop_overlay_mouse_move(&mut self, x: f32, y: f32) {
        match self.crop_state.overlay.drag_state.clone() {
            CropDragState::DraggingRectangle {
                start_rect_x,
                start_rect_y,
                start_cursor_x,
                start_cursor_y,
            } => {
                // Calculate delta
                let delta_x = x - start_cursor_x;
                let delta_y = y - start_cursor_y;

                // Update position with bounds checking
                let new_x = (start_rect_x as f32 + delta_x)
                    .max(0.0)
                    .min((self.crop_base_width - self.crop_state.width) as f32);
                let new_y = (start_rect_y as f32 + delta_y)
                    .max(0.0)
                    .min((self.crop_base_height - self.crop_state.height) as f32);

                self.crop_state.x = new_x as u32;
                self.crop_state.y = new_y as u32;
                self.crop_modified = true;
            }
            CropDragState::DraggingHandle {
                handle,
                start_rect,
                start_cursor_x,
                start_cursor_y,
            } => {
                // Calculate delta
                let delta_x = x - start_cursor_x;
                let delta_y = y - start_cursor_y;

                // Update crop dimensions based on which handle is being dragged
                self.update_crop_from_handle_drag(handle, start_rect, delta_x, delta_y);
                self.crop_modified = true;
                // Switch to Free ratio when manually resizing
                self.crop_state.ratio = CropRatio::Free;
            }
            CropDragState::None => {}
        }
    }

    /// Check if a point is inside the crop rectangle
    fn is_point_in_crop_rect(&self, x: f32, y: f32) -> bool {
        let rect_x = self.crop_state.x as f32;
        let rect_y = self.crop_state.y as f32;
        let rect_w = self.crop_state.width as f32;
        let rect_h = self.crop_state.height as f32;

        x >= rect_x && x <= rect_x + rect_w && y >= rect_y && y <= rect_y + rect_h
    }

    /// Get the handle at the given position, if any
    fn get_handle_at_position(&self, x: f32, y: f32) -> Option<HandlePosition> {
        const HANDLE_SIZE: f32 = 12.0; // Handle click area size

        let rect_x = self.crop_state.x as f32;
        let rect_y = self.crop_state.y as f32;
        let rect_w = self.crop_state.width as f32;
        let rect_h = self.crop_state.height as f32;

        // Define handle positions
        let handles = [
            (HandlePosition::TopLeft, rect_x, rect_y),
            (HandlePosition::Top, rect_x + rect_w / 2.0, rect_y),
            (HandlePosition::TopRight, rect_x + rect_w, rect_y),
            (
                HandlePosition::Right,
                rect_x + rect_w,
                rect_y + rect_h / 2.0,
            ),
            (
                HandlePosition::BottomRight,
                rect_x + rect_w,
                rect_y + rect_h,
            ),
            (
                HandlePosition::Bottom,
                rect_x + rect_w / 2.0,
                rect_y + rect_h,
            ),
            (HandlePosition::BottomLeft, rect_x, rect_y + rect_h),
            (HandlePosition::Left, rect_x, rect_y + rect_h / 2.0),
        ];

        // Check each handle
        for (handle, hx, hy) in handles {
            if (x - hx).abs() <= HANDLE_SIZE && (y - hy).abs() <= HANDLE_SIZE {
                return Some(handle);
            }
        }

        None
    }

    /// Update crop rectangle from handle drag
    fn update_crop_from_handle_drag(
        &mut self,
        handle: HandlePosition,
        start_rect: (u32, u32, u32, u32),
        delta_x: f32,
        delta_y: f32,
    ) {
        let (start_x, start_y, start_w, start_h) = start_rect;

        match handle {
            HandlePosition::TopLeft => {
                let new_x = (start_x as f32 + delta_x)
                    .max(0.0)
                    .min(start_x as f32 + start_w as f32 - 10.0);
                let new_y = (start_y as f32 + delta_y)
                    .max(0.0)
                    .min(start_y as f32 + start_h as f32 - 10.0);
                self.crop_state.width = (start_x + start_w) - new_x as u32;
                self.crop_state.height = (start_y + start_h) - new_y as u32;
                self.crop_state.x = new_x as u32;
                self.crop_state.y = new_y as u32;
            }
            HandlePosition::Top => {
                let new_y = (start_y as f32 + delta_y)
                    .max(0.0)
                    .min(start_y as f32 + start_h as f32 - 10.0);
                self.crop_state.height = (start_y + start_h) - new_y as u32;
                self.crop_state.y = new_y as u32;
            }
            HandlePosition::TopRight => {
                let new_y = (start_y as f32 + delta_y)
                    .max(0.0)
                    .min(start_y as f32 + start_h as f32 - 10.0);
                let new_w = (start_w as f32 + delta_x)
                    .max(10.0)
                    .min((self.crop_base_width - start_x) as f32);
                self.crop_state.width = new_w as u32;
                self.crop_state.height = (start_y + start_h) - new_y as u32;
                self.crop_state.y = new_y as u32;
            }
            HandlePosition::Right => {
                let new_w = (start_w as f32 + delta_x)
                    .max(10.0)
                    .min((self.crop_base_width - start_x) as f32);
                self.crop_state.width = new_w as u32;
            }
            HandlePosition::BottomRight => {
                let new_w = (start_w as f32 + delta_x)
                    .max(10.0)
                    .min((self.crop_base_width - start_x) as f32);
                let new_h = (start_h as f32 + delta_y)
                    .max(10.0)
                    .min((self.crop_base_height - start_y) as f32);
                self.crop_state.width = new_w as u32;
                self.crop_state.height = new_h as u32;
            }
            HandlePosition::Bottom => {
                let new_h = (start_h as f32 + delta_y)
                    .max(10.0)
                    .min((self.crop_base_height - start_y) as f32);
                self.crop_state.height = new_h as u32;
            }
            HandlePosition::BottomLeft => {
                let new_x = (start_x as f32 + delta_x)
                    .max(0.0)
                    .min(start_x as f32 + start_w as f32 - 10.0);
                let new_h = (start_h as f32 + delta_y)
                    .max(10.0)
                    .min((self.crop_base_height - start_y) as f32);
                self.crop_state.width = (start_x + start_w) - new_x as u32;
                self.crop_state.height = new_h as u32;
                self.crop_state.x = new_x as u32;
            }
            HandlePosition::Left => {
                let new_x = (start_x as f32 + delta_x)
                    .max(0.0)
                    .min(start_x as f32 + start_w as f32 - 10.0);
                self.crop_state.width = (start_x + start_w) - new_x as u32;
                self.crop_state.x = new_x as u32;
            }
        }

        // Apply aspect ratio constraint if needed
        if self.crop_state.ratio != CropRatio::Free {
            self.apply_aspect_ratio_constraint_to_current_crop();
        }
    }

    /// Apply aspect ratio constraint to current crop dimensions
    fn apply_aspect_ratio_constraint_to_current_crop(&mut self) {
        let target_ratio = match self.crop_state.ratio {
            CropRatio::None | CropRatio::Free => return, // No constraint
            CropRatio::Square => 1.0,
            CropRatio::Landscape => 16.0 / 9.0,
            CropRatio::Portrait => 9.0 / 16.0,
            CropRatio::Photo => 4.0 / 3.0,
            CropRatio::PhotoPortrait => 3.0 / 4.0,
        };

        // Adjust height to match ratio, keeping width fixed
        let new_height = (self.crop_state.width as f32 / target_ratio).round() as u32;

        // Check if new height fits
        if self.crop_state.y + new_height <= self.crop_base_height {
            self.crop_state.height = new_height;
        } else {
            // Height doesn't fit, adjust width instead
            let available_height = self.crop_base_height - self.crop_state.y;
            self.crop_state.height = available_height;
            self.crop_state.width = (available_height as f32 * target_ratio).round() as u32;
        }
    }

    /// Discard all changes and reset to original image state.
    pub fn discard_changes(&mut self) {
        // Reload the working image from disk
        match image_rs::open(&self.image_path) {
            Ok(fresh_image) => {
                self.working_image = fresh_image;
                match transform::dynamic_to_image_data(&self.working_image) {
                    Ok(image_data) => {
                        self.current_image = image_data.clone();
                        self.sync_resize_state_dimensions();

                        // Reset crop state
                        let crop_width = (self.current_image.width as f32 * 0.75).round() as u32;
                        let crop_height = (self.current_image.height as f32 * 0.75).round() as u32;
                        self.crop_state.x = (self.current_image.width - crop_width) / 2;
                        self.crop_state.y = (self.current_image.height - crop_height) / 2;
                        self.crop_state.width = crop_width;
                        self.crop_state.height = crop_height;
                        self.crop_state.ratio = CropRatio::Free;
                        self.crop_state.overlay.visible = false;
                        self.crop_state.overlay.drag_state = CropDragState::None;
                        self.crop_modified = false;

                        // Clear transformation history
                        self.transformation_history.clear();
                        self.history_index = 0;

                        // Clear active tool and preview
                        self.active_tool = None;
                        self.preview_image = None;
                    }
                    Err(err) => {
                        eprintln!("Failed to convert reloaded image: {err:?}");
                    }
                }
            }
            Err(err) => {
                eprintln!("Failed to reload original image: {err:?}");
            }
        }
    }

    /// Replay transformations from the original image up to the current history_index.
    /// This is used for undo/redo operations.
    fn replay_transformations_up_to_index(&mut self) {
        // Reload the original image from disk
        let Ok(mut working_image) = image_rs::open(&self.image_path) else {
            eprintln!("Failed to reload original image for replay");
            return;
        };

        // Apply transformations up to history_index
        for i in 0..self.history_index {
            if i >= self.transformation_history.len() {
                break;
            }

            working_image = match &self.transformation_history[i] {
                Transformation::RotateLeft => transform::rotate_left(&working_image),
                Transformation::RotateRight => transform::rotate_right(&working_image),
                Transformation::Crop { rect } => {
                    let x = rect.x.max(0.0) as u32;
                    let y = rect.y.max(0.0) as u32;
                    let width = rect.width.max(1.0) as u32;
                    let height = rect.height.max(1.0) as u32;
                    match transform::crop(&working_image, x, y, width, height) {
                        Some(cropped) => cropped,
                        None => {
                            eprintln!("Failed to apply crop during replay: invalid crop area");
                            working_image
                        }
                    }
                }
                Transformation::Resize { width, height } => {
                    transform::resize(&working_image, *width, *height)
                }
            };
        }

        // Update current state with replayed image
        self.working_image = working_image;
        match transform::dynamic_to_image_data(&self.working_image) {
            Ok(image_data) => {
                self.current_image = image_data;
                self.sync_resize_state_dimensions();
                self.preview_image = None;
            }
            Err(err) => {
                eprintln!("Failed to convert replayed image: {err:?}");
            }
        }
    }
}

fn parse_dimension_input(value: &str) -> Option<u32> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    match trimmed.parse::<u32>() {
        Ok(result) if result > 0 => Some(result),
        _ => None,
    }
}

/// Crop overlay renderer for interactive crop selection
struct CropOverlayRenderer {
    crop_x: u32,
    crop_y: u32,
    crop_width: u32,
    crop_height: u32,
    img_width: u32,
    img_height: u32,
}

impl CropOverlayRenderer {
    /// Convert screen coordinates to image coordinates (clamped to image bounds)
    fn screen_to_image_coords(
        &self,
        screen_pos: iced::Point,
        bounds: iced::Rectangle,
    ) -> Option<(f32, f32)> {
        // Calculate image position and scale (ContentFit::Contain logic)
        let img_aspect = self.img_width as f32 / self.img_height as f32;
        let bounds_aspect = bounds.width / bounds.height;

        let (img_display_width, img_display_height, img_offset_x, img_offset_y) =
            if img_aspect > bounds_aspect {
                let display_width = bounds.width;
                let display_height = bounds.width / img_aspect;
                let offset_y = (bounds.height - display_height) / 2.0;
                (display_width, display_height, 0.0, offset_y)
            } else {
                let display_height = bounds.height;
                let display_width = bounds.height * img_aspect;
                let offset_x = (bounds.width - display_width) / 2.0;
                (display_width, display_height, offset_x, 0.0)
            };

        // Clamp screen coordinates to image display area
        let clamped_x = screen_pos
            .x
            .max(img_offset_x)
            .min(img_offset_x + img_display_width);
        let clamped_y = screen_pos
            .y
            .max(img_offset_y)
            .min(img_offset_y + img_display_height);

        // Convert to image coordinates
        let img_x = ((clamped_x - img_offset_x) * (self.img_width as f32 / img_display_width))
            .max(0.0)
            .min(self.img_width as f32);
        let img_y = ((clamped_y - img_offset_y) * (self.img_height as f32 / img_display_height))
            .max(0.0)
            .min(self.img_height as f32);

        Some((img_x, img_y))
    }
}

impl iced::widget::canvas::Program<Message> for CropOverlayRenderer {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: iced::widget::canvas::Event,
        bounds: iced::Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> (iced::widget::canvas::event::Status, Option<Message>) {
        use iced::widget::canvas::event::Status;

        match event {
            // If cursor leaves the canvas, end any drag operation
            iced::widget::canvas::Event::Mouse(iced::mouse::Event::CursorLeft) => {
                return (Status::Captured, Some(Message::CropOverlayMouseUp));
            }
            iced::widget::canvas::Event::Mouse(iced::mouse::Event::ButtonPressed(
                iced::mouse::Button::Left,
            )) => {
                if let Some(cursor_position) = cursor.position_in(bounds) {
                    if let Some((img_x, img_y)) =
                        self.screen_to_image_coords(cursor_position, bounds)
                    {
                        return (
                            Status::Captured,
                            Some(Message::CropOverlayMouseDown { x: img_x, y: img_y }),
                        );
                    }
                }
            }
            iced::widget::canvas::Event::Mouse(iced::mouse::Event::CursorMoved { .. }) => {
                // If cursor is outside bounds during move, end drag
                if cursor.position_in(bounds).is_none() {
                    return (Status::Captured, Some(Message::CropOverlayMouseUp));
                }

                if let Some(cursor_position) = cursor.position_in(bounds) {
                    if let Some((img_x, img_y)) =
                        self.screen_to_image_coords(cursor_position, bounds)
                    {
                        return (
                            Status::Captured,
                            Some(Message::CropOverlayMouseMove { x: img_x, y: img_y }),
                        );
                    }
                }
            }
            iced::widget::canvas::Event::Mouse(iced::mouse::Event::ButtonReleased(
                iced::mouse::Button::Left,
            )) => {
                return (Status::Captured, Some(Message::CropOverlayMouseUp));
            }
            _ => {}
        }

        (Status::Ignored, None)
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry> {
        use iced::widget::canvas::{Frame, Path, Stroke};
        use iced::Color;

        let mut frame = Frame::new(renderer, bounds.size());

        // Calculate image position and scale (ContentFit::Contain logic)
        let img_aspect = self.img_width as f32 / self.img_height as f32;
        let bounds_aspect = bounds.width / bounds.height;

        let (img_display_width, img_display_height, img_offset_x, img_offset_y) =
            if img_aspect > bounds_aspect {
                // Image is wider - fit to width
                let display_width = bounds.width;
                let display_height = bounds.width / img_aspect;
                let offset_y = (bounds.height - display_height) / 2.0;
                (display_width, display_height, 0.0, offset_y)
            } else {
                // Image is taller - fit to height
                let display_height = bounds.height;
                let display_width = bounds.height * img_aspect;
                let offset_x = (bounds.width - display_width) / 2.0;
                (display_width, display_height, offset_x, 0.0)
            };

        // Scale factors
        let scale_x = img_display_width / self.img_width as f32;
        let scale_y = img_display_height / self.img_height as f32;

        // Convert crop coordinates from image space to screen space
        let crop_screen_x = img_offset_x + self.crop_x as f32 * scale_x;
        let crop_screen_y = img_offset_y + self.crop_y as f32 * scale_y;
        let crop_screen_width = self.crop_width as f32 * scale_x;
        let crop_screen_height = self.crop_height as f32 * scale_y;

        // Draw darkened overlay outside crop area
        let dark_overlay = Color::from_rgba8(0, 0, 0, 0.5);

        // Top rectangle
        if crop_screen_y > img_offset_y {
            frame.fill_rectangle(
                iced::Point::new(img_offset_x, img_offset_y),
                iced::Size::new(img_display_width, crop_screen_y - img_offset_y),
                dark_overlay,
            );
        }

        // Bottom rectangle
        let bottom_y = crop_screen_y + crop_screen_height;
        if bottom_y < img_offset_y + img_display_height {
            frame.fill_rectangle(
                iced::Point::new(img_offset_x, bottom_y),
                iced::Size::new(
                    img_display_width,
                    img_offset_y + img_display_height - bottom_y,
                ),
                dark_overlay,
            );
        }

        // Left rectangle
        if crop_screen_x > img_offset_x {
            frame.fill_rectangle(
                iced::Point::new(img_offset_x, crop_screen_y),
                iced::Size::new(crop_screen_x - img_offset_x, crop_screen_height),
                dark_overlay,
            );
        }

        // Right rectangle
        let right_x = crop_screen_x + crop_screen_width;
        if right_x < img_offset_x + img_display_width {
            frame.fill_rectangle(
                iced::Point::new(right_x, crop_screen_y),
                iced::Size::new(
                    img_offset_x + img_display_width - right_x,
                    crop_screen_height,
                ),
                dark_overlay,
            );
        }

        // Draw crop rectangle border
        let crop_rect = Path::rectangle(
            iced::Point::new(crop_screen_x, crop_screen_y),
            iced::Size::new(crop_screen_width, crop_screen_height),
        );
        frame.stroke(
            &crop_rect,
            Stroke::default().with_width(2.0).with_color(Color::WHITE),
        );

        // Draw rule-of-thirds grid
        let grid_color = Color::from_rgba8(255, 255, 255, 0.5);
        let third_width = crop_screen_width / 3.0;
        let third_height = crop_screen_height / 3.0;

        // Vertical lines
        for i in 1..3 {
            let x = crop_screen_x + third_width * i as f32;
            let line = Path::line(
                iced::Point::new(x, crop_screen_y),
                iced::Point::new(x, crop_screen_y + crop_screen_height),
            );
            frame.stroke(
                &line,
                Stroke::default().with_width(1.0).with_color(grid_color),
            );
        }

        // Horizontal lines
        for i in 1..3 {
            let y = crop_screen_y + third_height * i as f32;
            let line = Path::line(
                iced::Point::new(crop_screen_x, y),
                iced::Point::new(crop_screen_x + crop_screen_width, y),
            );
            frame.stroke(
                &line,
                Stroke::default().with_width(1.0).with_color(grid_color),
            );
        }

        // Draw resize handles
        let handle_size = 10.0;
        let handle_color = Color::WHITE;
        let handles = [
            (crop_screen_x, crop_screen_y),                           // TopLeft
            (crop_screen_x + crop_screen_width / 2.0, crop_screen_y), // Top
            (crop_screen_x + crop_screen_width, crop_screen_y),       // TopRight
            (
                crop_screen_x + crop_screen_width,
                crop_screen_y + crop_screen_height / 2.0,
            ), // Right
            (
                crop_screen_x + crop_screen_width,
                crop_screen_y + crop_screen_height,
            ), // BottomRight
            (
                crop_screen_x + crop_screen_width / 2.0,
                crop_screen_y + crop_screen_height,
            ), // Bottom
            (crop_screen_x, crop_screen_y + crop_screen_height),      // BottomLeft
            (crop_screen_x, crop_screen_y + crop_screen_height / 2.0), // Left
        ];

        for (hx, hy) in handles {
            let handle = Path::rectangle(
                iced::Point::new(hx - handle_size / 2.0, hy - handle_size / 2.0),
                iced::Size::new(handle_size, handle_size),
            );
            frame.fill(&handle, handle_color);
            frame.stroke(
                &handle,
                Stroke::default().with_width(1.0).with_color(Color::BLACK),
            );
        }

        vec![frame.into_geometry()]
    }
}

/// Resize overlay renderer showing original dimensions and preview
struct ResizeOverlayRenderer {
    /// Original image dimensions (reference markers - white rectangle)
    original_width: u32,
    original_height: u32,
    /// New dimensions after resize (preview - blue rectangle)
    new_width: u32,
    new_height: u32,
}

impl iced::widget::canvas::Program<Message> for ResizeOverlayRenderer {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        _event: iced::widget::canvas::Event,
        _bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> (iced::widget::canvas::event::Status, Option<Message>) {
        // No interaction needed for resize overlay
        (iced::widget::canvas::event::Status::Ignored, None)
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry> {
        use iced::widget::canvas::{Frame, Path, Stroke, Text};
        use iced::Color;

        let mut frame = Frame::new(renderer, bounds.size());

        // Calculate the bounding box that contains both original and new dimensions
        // This ensures both rectangles fit in the viewport
        let max_width = self.original_width.max(self.new_width);
        let max_height = self.original_height.max(self.new_height);

        // Use the max dimensions for ContentFit::Contain calculation
        let max_aspect = max_width as f32 / max_height as f32;
        let bounds_aspect = bounds.width / bounds.height;

        let (display_width, display_height, offset_x, offset_y) = if max_aspect > bounds_aspect {
            // Wider - fit to width
            let display_width = bounds.width;
            let display_height = display_width / max_aspect;
            let offset_y = (bounds.height - display_height) / 2.0;
            (display_width, display_height, 0.0, offset_y)
        } else {
            // Taller - fit to height
            let display_height = bounds.height;
            let display_width = display_height * max_aspect;
            let offset_x = (bounds.width - display_width) / 2.0;
            (display_width, display_height, offset_x, 0.0)
        };

        // Scale factors (how many screen pixels per image pixel)
        let scale_x = display_width / max_width as f32;
        let scale_y = display_height / max_height as f32;

        // Calculate screen dimensions for original and new sizes using the same scale
        let original_screen_width = self.original_width as f32 * scale_x;
        let original_screen_height = self.original_height as f32 * scale_y;
        let new_screen_width = self.new_width as f32 * scale_x;
        let new_screen_height = self.new_height as f32 * scale_y;

        // Center both rectangles within the display area
        let original_x = offset_x + (display_width - original_screen_width) / 2.0;
        let original_y = offset_y + (display_height - original_screen_height) / 2.0;
        let new_x = offset_x + (display_width - new_screen_width) / 2.0;
        let new_y = offset_y + (display_height - new_screen_height) / 2.0;

        // Draw the original dimensions marker first (white stroke, thick)
        let original_rect = Path::rectangle(
            iced::Point::new(original_x, original_y),
            iced::Size::new(original_screen_width, original_screen_height),
        );
        frame.stroke(
            &original_rect,
            Stroke::default().with_width(3.0).with_color(Color::WHITE),
        );

        // Draw the resized image area on top (blue stroke only, no fill to see through)
        let new_rect = Path::rectangle(
            iced::Point::new(new_x, new_y),
            iced::Size::new(new_screen_width, new_screen_height),
        );
        frame.stroke(
            &new_rect,
            Stroke::default()
                .with_width(3.0)
                .with_color(Color::from_rgb8(100, 150, 255)),
        );

        // Draw dimension labels
        let label_color = Color::WHITE;
        let font_size = 16.0;

        // Original dimensions label (top-left of original rect)
        let original_label = format!("Original: {}×{}", self.original_width, self.original_height);
        frame.fill_text(Text {
            content: original_label,
            position: iced::Point::new(original_x, original_y - 20.0),
            color: label_color,
            size: font_size.into(),
            ..Text::default()
        });

        // New dimensions label (bottom-right of new rect)
        let new_label = format!("New: {}×{}", self.new_width, self.new_height);
        frame.fill_text(Text {
            content: new_label,
            position: iced::Point::new(new_x, new_y + new_screen_height + 5.0),
            color: Color::from_rgb8(100, 150, 255),
            size: font_size.into(),
            ..Text::default()
        });

        vec![frame.into_geometry()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::widget::image;
    use image_rs::{Rgba, RgbaImage};
    use tempfile::tempdir;

    fn create_test_image(width: u32, height: u32) -> (tempfile::TempDir, PathBuf, ImageData) {
        let temp_dir = tempdir().expect("temp dir");
        let path = temp_dir.path().join("test.png");
        let img = RgbaImage::from_pixel(width, height, Rgba([0, 0, 0, 255]));
        img.save(&path).expect("write png");
        let pixels = vec![0; (width * height * 4) as usize];
        let image = ImageData {
            handle: image::Handle::from_rgba(width, height, pixels),
            width,
            height,
        };
        (temp_dir, path, image)
    }

    #[test]
    fn new_editor_state_has_no_changes() {
        let (_dir, path, img) = create_test_image(4, 3);
        let state = State::new(path, img).expect("editor state");

        assert!(!state.has_unsaved_changes());
        assert!(!state.can_undo());
        assert!(!state.can_redo());
        assert_eq!(state.active_tool(), None);
    }

    #[test]
    fn new_editor_state_initializes_resize_state() {
        let (_dir, path, img) = create_test_image(4, 3);
        let state = State::new(path, img).expect("editor state");

        assert_eq!(state.resize_state.width, 4);
        assert_eq!(state.resize_state.height, 3);
        assert_eq!(state.resize_state.scale_percent, 100.0);
        assert!(state.resize_state.lock_aspect);
        assert_eq!(state.resize_state.original_aspect, 4.0 / 3.0);
    }

    #[test]
    fn sidebar_starts_expanded() {
        let (_dir, path, img) = create_test_image(4, 3);
        let state = State::new(path, img).expect("editor state");

        assert!(state.is_sidebar_expanded());
    }

    #[test]
    fn crop_ratio_variants_are_distinct() {
        assert_ne!(CropRatio::Free, CropRatio::Square);
        assert_ne!(CropRatio::Landscape, CropRatio::Portrait);
        assert_ne!(CropRatio::Photo, CropRatio::PhotoPortrait);
    }

    #[test]
    fn apply_resize_updates_image_dimensions() {
        let (_dir, path, img) = create_test_image(8, 6);
        let mut state = State::new(path, img).expect("editor state");

        state.update(Message::SelectTool(EditorTool::Resize));
        state.resize_state.width = 4;
        state.resize_state.height = 3;
        state.resize_state.width_input = "4".into();
        state.resize_state.height_input = "3".into();
        state.update(Message::ApplyResize);

        assert_eq!(state.current_image.width, 4);
        assert_eq!(state.current_image.height, 3);
    }
}
