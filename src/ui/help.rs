// SPDX-License-Identifier: MPL-2.0
//! Help screen module providing in-app documentation.
//!
//! This module displays comprehensive documentation organized by functionality
//! with collapsible sections. Each section explains the role, available tools,
//! and usage instructions.

use crate::i18n::fluent::I18n;
use crate::ui::action_icons;
use crate::ui::design_tokens::{radius, sizing, spacing, typography};
use crate::ui::styles;
use iced::{
    alignment::{Horizontal, Vertical},
    font::Weight,
    widget::{button, container, scrollable, text, Column, Container, Row, Text},
    Border, Element, Font, Length, Theme,
};

/// Help sections that can be expanded/collapsed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HelpSection {
    Viewer,
    Video,
    Capture,
    Editor,
}

impl HelpSection {
    /// All available sections in display order.
    pub const ALL: [HelpSection; 4] = [
        HelpSection::Viewer,
        HelpSection::Video,
        HelpSection::Capture,
        HelpSection::Editor,
    ];
}

/// State for the help screen (tracks which sections are expanded).
#[derive(Debug, Clone)]
pub struct State {
    /// Set of expanded sections.
    expanded: std::collections::HashSet<HelpSection>,
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    /// Create a new help state with all sections collapsed.
    pub fn new() -> Self {
        Self {
            expanded: std::collections::HashSet::new(),
        }
    }

    /// Check if a section is expanded.
    pub fn is_expanded(&self, section: HelpSection) -> bool {
        self.expanded.contains(&section)
    }

    /// Toggle a section's expanded state.
    pub fn toggle(&mut self, section: HelpSection) {
        if self.expanded.contains(&section) {
            self.expanded.remove(&section);
        } else {
            self.expanded.insert(section);
        }
    }
}

/// Contextual data needed to render the help screen.
pub struct ViewContext<'a> {
    pub i18n: &'a I18n,
    pub state: &'a State,
}

/// Messages emitted by the help screen.
#[derive(Debug, Clone)]
pub enum Message {
    BackToViewer,
    ToggleSection(HelpSection),
}

/// Events propagated to the parent application.
#[derive(Debug, Clone)]
pub enum Event {
    None,
    BackToViewer,
}

/// Process a help screen message and return the corresponding event.
pub fn update(state: &mut State, message: Message) -> Event {
    match message {
        Message::BackToViewer => Event::BackToViewer,
        Message::ToggleSection(section) => {
            state.toggle(section);
            Event::None
        }
    }
}

/// Render the help screen.
pub fn view<'a>(ctx: ViewContext<'a>) -> Element<'a, Message> {
    let back_button = button(
        text(format!("← {}", ctx.i18n.tr("help-back-to-viewer-button"))).size(typography::BODY),
    )
    .on_press(Message::BackToViewer);

    let title = Text::new(ctx.i18n.tr("help-title")).size(typography::TITLE_LG);

    // Build collapsible sections
    let viewer_section = build_collapsible_section(
        &ctx,
        HelpSection::Viewer,
        action_icons::sections::viewer(),
        ctx.i18n.tr("help-section-viewer"),
        build_viewer_content(&ctx),
    );

    let video_section = build_collapsible_section(
        &ctx,
        HelpSection::Video,
        action_icons::sections::video(),
        ctx.i18n.tr("help-section-video"),
        build_video_content(&ctx),
    );

    let capture_section = build_collapsible_section(
        &ctx,
        HelpSection::Capture,
        action_icons::sections::capture(),
        ctx.i18n.tr("help-section-capture"),
        build_capture_content(&ctx),
    );

    let editor_section = build_collapsible_section(
        &ctx,
        HelpSection::Editor,
        action_icons::sections::editor(),
        ctx.i18n.tr("help-section-editor"),
        build_editor_content(&ctx),
    );

    let content = Column::new()
        .width(Length::Fill)
        .spacing(spacing::SM)
        .align_x(Horizontal::Left)
        .padding(spacing::MD)
        .push(back_button)
        .push(title)
        .push(viewer_section)
        .push(video_section)
        .push(capture_section)
        .push(editor_section);

    scrollable(content).into()
}

/// Build a collapsible section with header and content.
fn build_collapsible_section<'a>(
    ctx: &ViewContext<'a>,
    section: HelpSection,
    icon: iced::widget::Svg<'a>,
    title: String,
    content: Element<'a, Message>,
) -> Element<'a, Message> {
    let is_expanded = ctx.state.is_expanded(section);
    let icon_sized = action_icons::sized(icon, sizing::ICON_MD).style(styles::tinted_svg);

    // Expand/collapse indicator
    let indicator = Text::new(if is_expanded { "▼" } else { "▶" }).size(typography::BODY);

    let header_content = Row::new()
        .spacing(spacing::SM)
        .align_y(Vertical::Center)
        .push(indicator)
        .push(icon_sized)
        .push(Text::new(title).size(typography::TITLE_SM));

    let header = button(header_content)
        .width(Length::Fill)
        .padding(spacing::SM)
        .style(|theme: &Theme, status| {
            let palette = theme.extended_palette();
            match status {
                button::Status::Hovered | button::Status::Pressed => button::Style {
                    background: Some(palette.background.strong.color.into()),
                    text_color: palette.background.base.text,
                    border: Border {
                        radius: radius::MD.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                _ => button::Style {
                    background: Some(palette.background.weak.color.into()),
                    text_color: palette.background.base.text,
                    border: Border {
                        radius: radius::MD.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            }
        })
        .on_press(Message::ToggleSection(section));

    let mut section_column = Column::new().spacing(spacing::XS).push(header);

    if is_expanded {
        let content_container = Container::new(content)
            .padding(spacing::MD)
            .width(Length::Fill)
            .style(|theme: &Theme| container::Style {
                background: Some(theme.extended_palette().background.weak.color.into()),
                border: Border {
                    radius: radius::MD.into(),
                    ..Default::default()
                },
                ..Default::default()
            });
        section_column = section_column.push(content_container);
    }

    section_column.into()
}

// ─────────────────────────────────────────────────────────────────────────────
// Section content builders
// ─────────────────────────────────────────────────────────────────────────────

/// Build the viewer section content.
fn build_viewer_content<'a>(ctx: &ViewContext<'a>) -> Element<'a, Message> {
    let role = build_paragraph(ctx.i18n.tr("help-viewer-role"));

    let tools_title = build_subsection_title(ctx.i18n.tr("help-tools-title"));
    let tools_content = Column::new()
        .spacing(spacing::XS)
        .push(build_tool_item(
            ctx.i18n.tr("help-viewer-tool-navigation"),
            ctx.i18n.tr("help-viewer-tool-navigation-desc"),
        ))
        .push(build_tool_item_with_icon(
            action_icons::viewer::zoom_in(),
            ctx.i18n.tr("help-viewer-tool-zoom"),
            ctx.i18n.tr("help-viewer-tool-zoom-desc"),
        ))
        .push(build_tool_item(
            ctx.i18n.tr("help-viewer-tool-pan"),
            ctx.i18n.tr("help-viewer-tool-pan-desc"),
        ))
        .push(build_tool_item_with_icon(
            action_icons::viewer::fit_to_window(),
            ctx.i18n.tr("help-viewer-tool-fit"),
            ctx.i18n.tr("help-viewer-tool-fit-desc"),
        ))
        .push(build_tool_item_with_icon(
            action_icons::viewer::fullscreen(),
            ctx.i18n.tr("help-viewer-tool-fullscreen"),
            ctx.i18n.tr("help-viewer-tool-fullscreen-desc"),
        ))
        .push(build_tool_item_with_icon(
            action_icons::viewer::delete(),
            ctx.i18n.tr("help-viewer-tool-delete"),
            ctx.i18n.tr("help-viewer-tool-delete-desc"),
        ));

    let shortcuts_title = build_subsection_title(ctx.i18n.tr("help-shortcuts-title"));
    let shortcuts_content = Column::new()
        .spacing(spacing::XXS)
        .push(build_shortcut_row(
            "← / →",
            ctx.i18n.tr("help-viewer-key-navigate"),
        ))
        .push(build_shortcut_row("E", ctx.i18n.tr("help-viewer-key-edit")))
        .push(build_shortcut_row("I", ctx.i18n.tr("help-viewer-key-info")))
        .push(build_shortcut_row(
            "F11",
            ctx.i18n.tr("help-viewer-key-fullscreen"),
        ))
        .push(build_shortcut_row(
            "Esc",
            ctx.i18n.tr("help-viewer-key-exit-fullscreen"),
        ));

    let mouse_title = build_subsection_title(ctx.i18n.tr("help-mouse-title"));
    let mouse_content = Column::new()
        .spacing(spacing::XXS)
        .push(build_mouse_row(
            ctx.i18n.tr("viewer-double-click"),
            ctx.i18n.tr("help-viewer-mouse-doubleclick"),
        ))
        .push(build_mouse_row(
            ctx.i18n.tr("viewer-scroll-wheel"),
            ctx.i18n.tr("help-viewer-mouse-wheel"),
        ))
        .push(build_mouse_row(
            ctx.i18n.tr("viewer-click-drag"),
            ctx.i18n.tr("help-viewer-mouse-drag"),
        ));

    Column::new()
        .spacing(spacing::SM)
        .push(role)
        .push(tools_title)
        .push(tools_content)
        .push(shortcuts_title)
        .push(shortcuts_content)
        .push(mouse_title)
        .push(mouse_content)
        .into()
}

/// Build the video playback section content.
fn build_video_content<'a>(ctx: &ViewContext<'a>) -> Element<'a, Message> {
    let role = build_paragraph(ctx.i18n.tr("help-video-role"));

    let tools_title = build_subsection_title(ctx.i18n.tr("help-tools-title"));
    let tools_content = Column::new()
        .spacing(spacing::XS)
        .push(build_tool_item_with_icon(
            action_icons::video::play(),
            ctx.i18n.tr("help-video-tool-playback"),
            ctx.i18n.tr("help-video-tool-playback-desc"),
        ))
        .push(build_tool_item(
            ctx.i18n.tr("help-video-tool-timeline"),
            ctx.i18n.tr("help-video-tool-timeline-desc"),
        ))
        .push(build_tool_item_with_icon(
            action_icons::video::volume(),
            ctx.i18n.tr("help-video-tool-volume"),
            ctx.i18n.tr("help-video-tool-volume-desc"),
        ))
        .push(build_tool_item_with_icon(
            action_icons::video::toggle_loop(),
            ctx.i18n.tr("help-video-tool-loop"),
            ctx.i18n.tr("help-video-tool-loop-desc"),
        ))
        .push(build_tool_item_with_icon(
            action_icons::video::step_forward(),
            ctx.i18n.tr("help-video-tool-stepping"),
            ctx.i18n.tr("help-video-tool-stepping-desc"),
        ))
        .push(build_tool_item_with_icon(
            action_icons::video::capture_frame(),
            ctx.i18n.tr("help-video-tool-capture"),
            ctx.i18n.tr("help-video-tool-capture-desc"),
        ));

    let shortcuts_title = build_subsection_title(ctx.i18n.tr("help-shortcuts-title"));
    let shortcuts_content = Column::new()
        .spacing(spacing::XXS)
        .push(build_shortcut_row(
            "Space",
            ctx.i18n.tr("help-video-key-playpause"),
        ))
        .push(build_shortcut_row("M", ctx.i18n.tr("help-video-key-mute")))
        .push(build_shortcut_row(
            "← / →",
            ctx.i18n.tr("help-video-key-seek"),
        ))
        .push(build_shortcut_row(
            "↑ / ↓",
            ctx.i18n.tr("help-video-key-volume"),
        ))
        .push(build_shortcut_row(
            ",",
            ctx.i18n.tr("help-video-key-step-back"),
        ))
        .push(build_shortcut_row(
            ".",
            ctx.i18n.tr("help-video-key-step-forward"),
        ));

    Column::new()
        .spacing(spacing::SM)
        .push(role)
        .push(tools_title)
        .push(tools_content)
        .push(shortcuts_title)
        .push(shortcuts_content)
        .into()
}

/// Build the frame capture section content.
fn build_capture_content<'a>(ctx: &ViewContext<'a>) -> Element<'a, Message> {
    let role = build_paragraph(ctx.i18n.tr("help-capture-role"));

    let usage_title = build_subsection_title(ctx.i18n.tr("help-usage-title"));
    let usage_content = Column::new()
        .spacing(spacing::XXS)
        .push(build_numbered_step("1", ctx.i18n.tr("help-capture-step1")))
        .push(build_numbered_step("2", ctx.i18n.tr("help-capture-step2")))
        .push(build_numbered_step("3", ctx.i18n.tr("help-capture-step3")))
        .push(build_numbered_step("4", ctx.i18n.tr("help-capture-step4")));

    let formats = build_paragraph(ctx.i18n.tr("help-capture-formats"));

    Column::new()
        .spacing(spacing::SM)
        .push(role)
        .push(usage_title)
        .push(usage_content)
        .push(formats)
        .into()
}

/// Build the image editor section content.
fn build_editor_content<'a>(ctx: &ViewContext<'a>) -> Element<'a, Message> {
    let role = build_paragraph(ctx.i18n.tr("help-editor-role"));
    let workflow = build_paragraph(ctx.i18n.tr("help-editor-workflow"));

    // Rotate tool
    let rotate_title = build_tool_title(ctx.i18n.tr("help-editor-rotate-title"));
    let rotate_content = Column::new()
        .spacing(spacing::XXS)
        .push(build_paragraph(ctx.i18n.tr("help-editor-rotate-desc")))
        .push(build_bullet_with_icon(
            action_icons::editor::rotate_left(),
            ctx.i18n.tr("help-editor-rotate-left"),
        ))
        .push(build_bullet_with_icon(
            action_icons::editor::rotate_right(),
            ctx.i18n.tr("help-editor-rotate-right"),
        ))
        .push(build_bullet_with_icon(
            action_icons::editor::flip_horizontal(),
            ctx.i18n.tr("help-editor-flip-h"),
        ))
        .push(build_bullet_with_icon(
            action_icons::editor::flip_vertical(),
            ctx.i18n.tr("help-editor-flip-v"),
        ));

    // Crop tool
    let crop_title = build_tool_title(ctx.i18n.tr("help-editor-crop-title"));
    let crop_content = Column::new()
        .spacing(spacing::XXS)
        .push(build_paragraph(ctx.i18n.tr("help-editor-crop-desc")))
        .push(build_paragraph(ctx.i18n.tr("help-editor-crop-ratios")))
        .push(build_paragraph(ctx.i18n.tr("help-editor-crop-usage")));

    // Resize tool
    let resize_title = build_tool_title(ctx.i18n.tr("help-editor-resize-title"));
    let resize_content = Column::new()
        .spacing(spacing::XXS)
        .push(build_paragraph(ctx.i18n.tr("help-editor-resize-desc")))
        .push(build_bullet(ctx.i18n.tr("help-editor-resize-scale")))
        .push(build_bullet(ctx.i18n.tr("help-editor-resize-dimensions")))
        .push(build_bullet(ctx.i18n.tr("help-editor-resize-lock")))
        .push(build_bullet(ctx.i18n.tr("help-editor-resize-presets")));

    // Light tool (brightness/contrast)
    let light_title = build_tool_title(ctx.i18n.tr("help-editor-light-title"));
    let light_content = Column::new()
        .spacing(spacing::XXS)
        .push(build_paragraph(ctx.i18n.tr("help-editor-light-desc")))
        .push(build_bullet(ctx.i18n.tr("help-editor-light-brightness")))
        .push(build_bullet(ctx.i18n.tr("help-editor-light-contrast")))
        .push(build_bullet(ctx.i18n.tr("help-editor-light-preview")));

    // Save options
    let save_title = build_tool_title(ctx.i18n.tr("help-editor-save-title"));
    let save_content = Column::new()
        .spacing(spacing::XXS)
        .push(build_bullet(ctx.i18n.tr("help-editor-save-overwrite")))
        .push(build_bullet(ctx.i18n.tr("help-editor-save-as")));

    // Shortcuts
    let shortcuts_title = build_subsection_title(ctx.i18n.tr("help-shortcuts-title"));
    let shortcuts_content = Column::new()
        .spacing(spacing::XXS)
        .push(build_shortcut_row(
            "Ctrl+S",
            ctx.i18n.tr("help-editor-key-save"),
        ))
        .push(build_shortcut_row(
            "Ctrl+Z",
            ctx.i18n.tr("help-editor-key-undo"),
        ))
        .push(build_shortcut_row(
            "Ctrl+Y",
            ctx.i18n.tr("help-editor-key-redo"),
        ))
        .push(build_shortcut_row(
            "Esc",
            ctx.i18n.tr("help-editor-key-cancel"),
        ));

    Column::new()
        .spacing(spacing::SM)
        .push(role)
        .push(workflow)
        .push(rotate_title)
        .push(rotate_content)
        .push(crop_title)
        .push(crop_content)
        .push(resize_title)
        .push(resize_content)
        .push(light_title)
        .push(light_content)
        .push(save_title)
        .push(save_content)
        .push(shortcuts_title)
        .push(shortcuts_content)
        .into()
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper functions for building UI elements
// ─────────────────────────────────────────────────────────────────────────────

/// Build a paragraph of text.
fn build_paragraph<'a>(content: String) -> Element<'a, Message> {
    Text::new(content).size(typography::BODY).into()
}

/// Build a subsection title (e.g., "Available Tools", "Keyboard Shortcuts").
fn build_subsection_title<'a>(title: String) -> Element<'a, Message> {
    Text::new(title)
        .size(typography::BODY)
        .style(|theme: &Theme| text::Style {
            color: Some(theme.extended_palette().background.strong.text),
        })
        .into()
}

/// Build a tool title (e.g., "Rotation", "Crop").
fn build_tool_title<'a>(title: String) -> Element<'a, Message> {
    Text::new(title)
        .size(typography::BODY)
        .style(|theme: &Theme| text::Style {
            color: Some(theme.extended_palette().primary.strong.color),
        })
        .into()
}

/// Build a tool item with name and description.
fn build_tool_item<'a>(name: String, description: String) -> Element<'a, Message> {
    Row::new()
        .spacing(spacing::SM)
        .push(
            Text::new(format!("• {}:", name))
                .size(typography::BODY)
                .font(Font {
                    weight: Weight::Bold,
                    ..Font::default()
                }),
        )
        .push(Text::new(description).size(typography::BODY))
        .into()
}

/// Size for inline icons in help text.
const HELP_ICON_SIZE: f32 = 18.0;

/// Build a tool item with an icon, name, and description.
fn build_tool_item_with_icon<'a>(
    icon: iced::widget::Svg<'a>,
    name: String,
    description: String,
) -> Element<'a, Message> {
    let icon_widget = action_icons::sized(icon, HELP_ICON_SIZE).style(styles::tinted_svg);

    Row::new()
        .spacing(spacing::SM)
        .align_y(Vertical::Center)
        .push(Text::new("•").size(typography::BODY))
        .push(icon_widget)
        .push(
            Text::new(format!("{}:", name))
                .size(typography::BODY)
                .font(Font {
                    weight: Weight::Bold,
                    ..Font::default()
                }),
        )
        .push(Text::new(description).size(typography::BODY))
        .into()
}

/// Build a bullet point with an icon.
fn build_bullet_with_icon<'a>(
    icon: iced::widget::Svg<'a>,
    content: String,
) -> Element<'a, Message> {
    let icon_widget = action_icons::sized(icon, HELP_ICON_SIZE).style(styles::tinted_svg);

    Row::new()
        .spacing(spacing::SM)
        .align_y(Vertical::Center)
        .push(Text::new("  •").size(typography::BODY))
        .push(icon_widget)
        .push(Text::new(content).size(typography::BODY))
        .into()
}

/// Build a bullet point.
fn build_bullet<'a>(content: String) -> Element<'a, Message> {
    Text::new(format!("  • {}", content))
        .size(typography::BODY)
        .into()
}

/// Build a numbered step (for instructions).
fn build_numbered_step<'a>(number: &'a str, content: String) -> Element<'a, Message> {
    let badge = Container::new(Text::new(number).size(typography::CAPTION))
        .padding([spacing::XXS, spacing::XS])
        .style(|theme: &Theme| container::Style {
            background: Some(theme.extended_palette().primary.base.color.into()),
            border: Border {
                radius: radius::SM.into(),
                ..Default::default()
            },
            text_color: Some(theme.extended_palette().primary.base.text),
            ..Default::default()
        });

    Row::new()
        .spacing(spacing::SM)
        .align_y(Vertical::Center)
        .push(badge)
        .push(Text::new(content).size(typography::BODY))
        .into()
}

/// Build a single shortcut row with key badge and description.
fn build_shortcut_row<'a>(key: &'a str, description: String) -> Element<'a, Message> {
    let key_badge = Container::new(Text::new(key).size(typography::CAPTION))
        .padding([spacing::XXS, spacing::XS])
        .style(|theme: &Theme| container::Style {
            background: Some(theme.extended_palette().background.strong.color.into()),
            border: Border {
                radius: radius::SM.into(),
                ..Default::default()
            },
            ..Default::default()
        });

    Row::new()
        .spacing(spacing::SM)
        .align_y(Vertical::Center)
        .push(Container::new(key_badge).width(Length::Fixed(70.0)))
        .push(Text::new(description).size(typography::BODY))
        .into()
}

/// Build a single mouse interaction row with action badge and description.
fn build_mouse_row<'a>(action: String, description: String) -> Element<'a, Message> {
    let action_badge = Container::new(Text::new(action).size(typography::CAPTION))
        .padding([spacing::XXS, spacing::XS])
        .style(|theme: &Theme| container::Style {
            background: Some(theme.extended_palette().background.strong.color.into()),
            border: Border {
                radius: radius::SM.into(),
                ..Default::default()
            },
            ..Default::default()
        });

    Row::new()
        .spacing(spacing::SM)
        .align_y(Vertical::Center)
        .push(Container::new(action_badge).width(Length::Fixed(120.0)))
        .push(Text::new(description).size(typography::BODY))
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::fluent::I18n;

    #[test]
    fn help_view_renders() {
        let i18n = I18n::default();
        let state = State::new();
        let ctx = ViewContext {
            i18n: &i18n,
            state: &state,
        };
        let _element = view(ctx);
    }

    #[test]
    fn back_to_viewer_emits_event() {
        let mut state = State::new();
        let event = update(&mut state, Message::BackToViewer);
        assert!(matches!(event, Event::BackToViewer));
    }

    #[test]
    fn toggle_section_expands_and_collapses() {
        let mut state = State::new();
        assert!(!state.is_expanded(HelpSection::Viewer));

        update(&mut state, Message::ToggleSection(HelpSection::Viewer));
        assert!(state.is_expanded(HelpSection::Viewer));

        update(&mut state, Message::ToggleSection(HelpSection::Viewer));
        assert!(!state.is_expanded(HelpSection::Viewer));
    }

    #[test]
    fn multiple_sections_can_be_expanded() {
        let mut state = State::new();

        update(&mut state, Message::ToggleSection(HelpSection::Viewer));
        update(&mut state, Message::ToggleSection(HelpSection::Editor));

        assert!(state.is_expanded(HelpSection::Viewer));
        assert!(state.is_expanded(HelpSection::Editor));
        assert!(!state.is_expanded(HelpSection::Video));
        assert!(!state.is_expanded(HelpSection::Capture));
    }
}
