// SPDX-License-Identifier: MPL-2.0
//! Filter dropdown component for the viewer toolbar.
//!
//! Provides a dropdown menu for filtering media during navigation.
//! Supports filtering by media type (images/videos) and date range.

use crate::i18n::fluent::I18n;
use crate::media::filter::{DateFilterField, MediaFilter, MediaTypeFilter};
use crate::ui::action_icons;
use crate::ui::design_tokens::{radius, spacing, typography};
use crate::ui::icons;
use crate::ui::styles;
use crate::ui::viewer::shared_styles;
use iced::widget::{button, container, pick_list, text, text_input, toggler, Column, Row, Text};
use iced::{alignment::Vertical, Border, Element, Length, Padding, Theme};
use std::time::SystemTime;

// =============================================================================
// Messages
// =============================================================================

/// Which date is being edited (start or end).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateTarget {
    Start,
    End,
}

/// Which segment of the date is being edited.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateSegment {
    Day,
    Month,
    Year,
}

/// Messages emitted by the filter dropdown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    /// Toggle the dropdown visibility.
    ToggleDropdown,
    /// Close the dropdown (e.g., when clicking outside).
    CloseDropdown,
    /// No-op message to consume clicks on the panel without closing it.
    ConsumeClick,
    /// Media type filter changed.
    MediaTypeChanged(MediaTypeFilter),
    /// Toggle date filter on/off.
    ToggleDateFilter(bool),
    /// Date filter field changed (created/modified).
    DateFieldChanged(DateFilterField),
    /// Date segment input changed (with auto-advance logic).
    DateSegmentChanged {
        target: DateTarget,
        segment: DateSegment,
        value: String,
    },
    /// Date fully entered and validated - submit to filter.
    DateSubmit(DateTarget),
    /// Clear a date (start or end).
    ClearDate(DateTarget),
    /// Reset all filters to default.
    ResetFilters,
}


// =============================================================================
// State
// =============================================================================

/// State for a single segmented date input (day, month, year).
#[derive(Debug, Clone, Default)]
pub struct SegmentedDateState {
    /// Day value (1-31).
    pub day: String,
    /// Month value (1-12).
    pub month: String,
    /// Year value (1970-2100).
    pub year: String,
}

impl SegmentedDateState {
    /// Check if all segments are empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.day.is_empty() && self.month.is_empty() && self.year.is_empty()
    }

    /// Clear all segments.
    pub fn clear(&mut self) {
        self.day.clear();
        self.month.clear();
        self.year.clear();
    }

    /// Try to parse the segments into a valid date.
    /// Returns `Some(SystemTime)` if valid, `None` otherwise.
    #[must_use]
    pub fn to_system_time(&self) -> Option<SystemTime> {
        if self.day.is_empty() || self.month.is_empty() || self.year.is_empty() {
            return None;
        }

        let day: u32 = self.day.parse().ok()?;
        let month: u32 = self.month.parse().ok()?;
        let year: u32 = self.year.parse().ok()?;

        // Basic validation
        if !(1970..=2100).contains(&year) || !(1..=12).contains(&month) || !(1..=31).contains(&day)
        {
            return None;
        }

        // Convert to days since epoch
        let days = ymd_to_days(year, month, day)?;
        let secs = u64::from(days) * SECS_PER_DAY;

        Some(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(secs))
    }

    /// Set segments from a `SystemTime`.
    pub fn set_from_system_time(&mut self, time: SystemTime) {
        let formatted = format_system_time(time);
        // Format is YYYY-MM-DD
        let parts: Vec<&str> = formatted.split('-').collect();
        if parts.len() == 3 {
            self.year = parts[0].to_string();
            self.month = parts[1].to_string();
            self.day = parts[2].to_string();
        }
    }

    /// Validate the day segment (1-31).
    #[must_use]
    pub fn is_day_valid(&self) -> bool {
        if self.day.is_empty() {
            return true; // Empty is valid (not yet entered)
        }
        self.day
            .parse::<u32>()
            .is_ok_and(|d| (1..=31).contains(&d))
    }

    /// Validate the month segment (1-12).
    #[must_use]
    pub fn is_month_valid(&self) -> bool {
        if self.month.is_empty() {
            return true;
        }
        self.month
            .parse::<u32>()
            .is_ok_and(|m| (1..=12).contains(&m))
    }

    /// Validate the year segment (1970-2100).
    #[must_use]
    pub fn is_year_valid(&self) -> bool {
        if self.year.is_empty() {
            return true;
        }
        self.year
            .parse::<u32>()
            .is_ok_and(|y| (1970..=2100).contains(&y))
    }

    /// Check if the complete date is valid (all segments filled and valid).
    #[must_use]
    pub fn is_complete_and_valid(&self) -> bool {
        !self.day.is_empty()
            && !self.month.is_empty()
            && !self.year.is_empty()
            && self.to_system_time().is_some()
    }
}

/// State for the filter dropdown component.
#[derive(Debug, Clone, Default)]
pub struct FilterDropdownState {
    /// Whether the dropdown is currently open.
    pub is_open: bool,
    /// Start date segments.
    pub start_date: SegmentedDateState,
    /// End date segments.
    pub end_date: SegmentedDateState,
}

impl FilterDropdownState {
    /// Create a new filter dropdown state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Toggle the dropdown open/closed.
    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }

    /// Close the dropdown.
    pub fn close(&mut self) {
        self.is_open = false;
    }

    /// Get mutable reference to the date state for a target.
    pub fn date_state_mut(&mut self, target: DateTarget) -> &mut SegmentedDateState {
        match target {
            DateTarget::Start => &mut self.start_date,
            DateTarget::End => &mut self.end_date,
        }
    }

    /// Get reference to the date state for a target.
    #[must_use]
    pub fn date_state(&self, target: DateTarget) -> &SegmentedDateState {
        match target {
            DateTarget::Start => &self.start_date,
            DateTarget::End => &self.end_date,
        }
    }

    /// Update a specific segment of a date.
    /// Returns `true` if auto-advance should happen (segment is complete).
    pub fn set_segment(&mut self, target: DateTarget, segment: DateSegment, value: String) -> bool {
        let date_state = self.date_state_mut(target);

        // Filter to only digits
        let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();

        // Apply max length and store
        let (max_len, should_advance) = match segment {
            DateSegment::Day => {
                let truncated = if digits.len() > 2 {
                    digits[..2].to_string()
                } else {
                    digits.clone()
                };
                date_state.day = truncated;
                (2, digits.len() >= 2)
            }
            DateSegment::Month => {
                let truncated = if digits.len() > 2 {
                    digits[..2].to_string()
                } else {
                    digits.clone()
                };
                date_state.month = truncated;
                (2, digits.len() >= 2)
            }
            DateSegment::Year => {
                let truncated = if digits.len() > 4 {
                    digits[..4].to_string()
                } else {
                    digits.clone()
                };
                date_state.year = truncated;
                (4, digits.len() >= 4)
            }
        };

        // Only advance if we've reached the max length
        should_advance && digits.len() >= max_len
    }

    /// Clear a date (start or end).
    pub fn clear_date(&mut self, target: DateTarget) {
        self.date_state_mut(target).clear();
    }

    /// Sync input values from the current filter state.
    /// Call this when the filter is loaded from settings.
    pub fn sync_from_filter(&mut self, filter: &MediaFilter) {
        if let Some(ref date_range) = filter.date_range {
            if let Some(start) = date_range.start {
                self.start_date.set_from_system_time(start);
            } else {
                self.start_date.clear();
            }
            if let Some(end) = date_range.end {
                self.end_date.set_from_system_time(end);
            } else {
                self.end_date.clear();
            }
        } else {
            self.start_date.clear();
            self.end_date.clear();
        }
    }
}

// =============================================================================
// View Context
// =============================================================================

/// Context for rendering the filter dropdown.
#[derive(Clone)]
pub struct ViewContext<'a> {
    pub i18n: &'a I18n,
    pub filter: &'a MediaFilter,
    pub state: &'a FilterDropdownState,
    /// Total number of media files in the directory.
    pub total_count: usize,
    /// Number of media files matching the current filter.
    pub filtered_count: usize,
}

// =============================================================================
// View
// =============================================================================

/// Render just the filter button (for toolbar).
/// The panel is rendered separately as an overlay via `view_panel`.
pub fn view_button<'a>(ctx: &ViewContext<'a>) -> Element<'a, Message> {
    let filter_active = ctx.filter.is_active();

    // Filter button with indicator
    let filter_icon = if filter_active {
        icons::fill(action_icons::viewer::toolbar::filter_active())
    } else {
        icons::fill(action_icons::viewer::toolbar::filter())
    };

    let filter_button = button(filter_icon)
        .on_press(Message::ToggleDropdown)
        .padding(spacing::XXS)
        .width(Length::Fixed(shared_styles::ICON_SIZE))
        .height(Length::Fixed(shared_styles::ICON_SIZE));

    // Apply selected style if filter is active or dropdown is open
    let filter_button_styled: Element<'_, Message> = if filter_active || ctx.state.is_open {
        filter_button.style(styles::button::selected).into()
    } else {
        filter_button.into()
    };

    // Build tooltip text - show active filters if any
    let tooltip_text = build_tooltip_text(ctx);

    styles::tooltip::styled(
        filter_button_styled,
        tooltip_text,
        iced::widget::tooltip::Position::Bottom,
    )
    .into()
}

/// Build the tooltip text based on active filters.
fn build_tooltip_text(ctx: &ViewContext<'_>) -> String {
    use crate::media::filter::MediaTypeFilter;

    if !ctx.filter.is_active() {
        return ctx.i18n.tr("filter-dropdown-tooltip");
    }

    let mut parts = Vec::new();

    // Add media type filter description
    match ctx.filter.media_type {
        MediaTypeFilter::ImagesOnly => {
            parts.push(ctx.i18n.tr("filter-media-type-images"));
        }
        MediaTypeFilter::VideosOnly => {
            parts.push(ctx.i18n.tr("filter-media-type-videos"));
        }
        MediaTypeFilter::All => {}
    }

    // Add date filter description
    if let Some(ref date_range) = ctx.filter.date_range {
        let date_desc = match (date_range.start, date_range.end) {
            (Some(start), Some(end)) => {
                let start_str = format_system_time(start);
                let end_str = format_system_time(end);
                ctx.i18n
                    .tr_with_args("filter-tooltip-date-range", &[("start", &start_str), ("end", &end_str)])
            }
            (Some(start), None) => {
                let date_str = format_system_time(start);
                ctx.i18n.tr_with_args("filter-tooltip-date-from", &[("date", &date_str)])
            }
            (None, Some(end)) => {
                let date_str = format_system_time(end);
                ctx.i18n.tr_with_args("filter-tooltip-date-to", &[("date", &date_str)])
            }
            (None, None) => String::new(),
        };
        if !date_desc.is_empty() {
            parts.push(date_desc);
        }
    }

    if parts.is_empty() {
        ctx.i18n.tr("filter-dropdown-tooltip")
    } else {
        let filters = parts.join(", ");
        ctx.i18n.tr_with_args("filter-dropdown-tooltip-active", &[("filters", &filters)])
    }
}

/// Render the filter panel as an overlay element.
/// Returns None if the panel is closed.
#[must_use]
pub fn view_panel(ctx: ViewContext<'_>) -> Option<Element<'_, Message>> {
    if !ctx.state.is_open {
        return None;
    }
    Some(build_dropdown_panel(ctx))
}

/// Render the filter dropdown button with optional open panel.
/// Legacy function that combines button and panel in a column.
#[deprecated(note = "Use view_button + view_panel for proper overlay rendering")]
#[must_use]
pub fn view(ctx: ViewContext<'_>) -> Element<'_, Message> {
    let button = view_button(&ctx);

    if !ctx.state.is_open {
        return button;
    }

    let panel = build_dropdown_panel(ctx);
    Column::new()
        .push(button)
        .push(panel)
        .spacing(spacing::XXS)
        .into()
}

/// Build the dropdown panel with filter options.
#[allow(clippy::needless_pass_by_value)] // ViewContext is small and all fields are references
fn build_dropdown_panel(ctx: ViewContext<'_>) -> Element<'_, Message> {
    let filter_active = ctx.filter.is_active();

    // Header with title and count
    let header = build_header(&ctx);

    // Media type filter section
    let media_type_section = build_media_type_section(&ctx);

    // Date filter section
    let date_section = build_date_section(&ctx);

    // Reset button (only shown when filter is active)
    let footer: Option<Element<'_, Message>> = if filter_active {
        let reset_btn: Element<'_, Message> =
            button(text(ctx.i18n.tr("filter-reset-button")).size(typography::BODY))
                .on_press(Message::ResetFilters)
                .padding([spacing::XXS, spacing::SM])
                .style(styles::button::unselected)
                .into();

        Some(
            container(reset_btn)
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Right)
                .into(),
        )
    } else {
        None
    };

    // Assemble panel
    let mut content = Column::new()
        .spacing(spacing::SM)
        .push(header)
        .push(media_type_section)
        .push(date_section);

    if let Some(footer_elem) = footer {
        content = content.push(footer_elem);
    }

    container(content)
        .padding(Padding::new(spacing::MD))
        .width(Length::Fixed(280.0))
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();
            container::Style {
                background: Some(palette.background.base.color.into()),
                border: Border {
                    color: palette.background.strong.color,
                    width: 1.0,
                    radius: radius::MD.into(),
                },
                ..Default::default()
            }
        })
        .into()
}

/// Build the header row with title and filter count.
fn build_header<'a>(ctx: &ViewContext<'a>) -> Element<'a, Message> {
    let title = Text::new(ctx.i18n.tr("filter-panel-title"))
        .size(typography::BODY_LG)
        .style(|theme: &Theme| text::Style {
            color: Some(theme.extended_palette().primary.strong.color),
        });

    let count_text = if ctx.filter.is_active() {
        format!("{} / {}", ctx.filtered_count, ctx.total_count)
    } else {
        format!("{}", ctx.total_count)
    };

    let count_label = Text::new(count_text).size(typography::BODY);

    Row::new()
        .push(title)
        .push(iced::widget::Space::new().width(Length::Fill))
        .push(count_label)
        .align_y(Vertical::Center)
        .into()
}

/// Build the media type filter section.
fn build_media_type_section<'a>(ctx: &ViewContext<'a>) -> Element<'a, Message> {
    let label = Text::new(ctx.i18n.tr("filter-media-type-label")).size(typography::BODY);

    let options = vec![
        MediaTypeOption {
            filter: MediaTypeFilter::All,
            label: ctx.i18n.tr("filter-media-type-all"),
        },
        MediaTypeOption {
            filter: MediaTypeFilter::ImagesOnly,
            label: ctx.i18n.tr("filter-media-type-images"),
        },
        MediaTypeOption {
            filter: MediaTypeFilter::VideosOnly,
            label: ctx.i18n.tr("filter-media-type-videos"),
        },
    ];

    let selected = options
        .iter()
        .find(|opt| opt.filter == ctx.filter.media_type)
        .cloned();

    let picker = pick_list(options, selected, |opt| {
        Message::MediaTypeChanged(opt.filter)
    })
    .placeholder(ctx.i18n.tr("filter-media-type-placeholder"))
    .padding(spacing::XS)
    .width(Length::Fill);

    Column::new()
        .spacing(spacing::XXS)
        .push(label)
        .push(picker)
        .into()
}

/// Build the date filter section.
fn build_date_section<'a>(ctx: &ViewContext<'a>) -> Element<'a, Message> {
    let date_filter_enabled = ctx.filter.date_range.is_some();

    let label = Text::new(ctx.i18n.tr("filter-date-label")).size(typography::BODY);

    let toggle = toggler(date_filter_enabled)
        .on_toggle(Message::ToggleDateFilter)
        .size(20.0);

    let header_row = Row::new()
        .push(label)
        .push(iced::widget::Space::new().width(Length::Fill))
        .push(toggle)
        .align_y(Vertical::Center);

    let mut section = Column::new().spacing(spacing::XXS).push(header_row);

    // Show date inputs when date filter is enabled
    if date_filter_enabled {
        let field_options = vec![
            DateFieldOption {
                field: DateFilterField::Modified,
                label: ctx.i18n.tr("filter-date-field-modified"),
            },
            DateFieldOption {
                field: DateFilterField::Created,
                label: ctx.i18n.tr("filter-date-field-created"),
            },
        ];

        let current_field = ctx
            .filter
            .date_range
            .as_ref()
            .map_or(DateFilterField::Modified, |d| d.field);

        let selected = field_options
            .iter()
            .find(|opt| opt.field == current_field)
            .cloned();

        let field_picker = pick_list(field_options, selected, |opt| {
            Message::DateFieldChanged(opt.field)
        })
        .padding(spacing::XS)
        .width(Length::Fill);

        let field_label =
            Text::new(ctx.i18n.tr("filter-date-field-label")).size(typography::BODY_SM);

        section = section.push(
            Column::new()
                .spacing(spacing::XXS)
                .push(field_label)
                .push(field_picker),
        );

        // Get current date range values for display
        let date_range = ctx.filter.date_range.as_ref();
        let has_start = date_range.and_then(|dr| dr.start).is_some();
        let has_end = date_range.and_then(|dr| dr.end).is_some();

        // Start date input (segmented)
        let start_label =
            Text::new(ctx.i18n.tr("filter-date-start-label")).size(typography::BODY_SM);

        let start_row = build_segmented_date_input(ctx, DateTarget::Start, has_start);

        section = section.push(
            Column::new()
                .spacing(spacing::XXS)
                .push(start_label)
                .push(start_row),
        );

        // End date input (segmented)
        let end_label = Text::new(ctx.i18n.tr("filter-date-end-label")).size(typography::BODY_SM);

        let end_row = build_segmented_date_input(ctx, DateTarget::End, has_end);

        section = section.push(
            Column::new()
                .spacing(spacing::XXS)
                .push(end_label)
                .push(end_row),
        );
    }

    section.into()
}

/// Width for day/month input fields.
const SEGMENT_WIDTH_SHORT: f32 = 36.0;
/// Width for year input field.
const SEGMENT_WIDTH_LONG: f32 = 52.0;

/// Build a segmented date input row (DD / MM / YYYY) with clear button.
fn build_segmented_date_input<'a>(
    ctx: &ViewContext<'a>,
    target: DateTarget,
    has_value: bool,
) -> Element<'a, Message> {
    let date_state = ctx.state.date_state(target);

    // Create styled text inputs for each segment
    let day_input = build_segment_input(
        ctx,
        target,
        DateSegment::Day,
        &date_state.day,
        ctx.i18n.tr("filter-date-day-placeholder"),
        SEGMENT_WIDTH_SHORT,
        date_state.is_day_valid(),
    );

    let month_input = build_segment_input(
        ctx,
        target,
        DateSegment::Month,
        &date_state.month,
        ctx.i18n.tr("filter-date-month-placeholder"),
        SEGMENT_WIDTH_SHORT,
        date_state.is_month_valid(),
    );

    let year_input = build_segment_input(
        ctx,
        target,
        DateSegment::Year,
        &date_state.year,
        ctx.i18n.tr("filter-date-year-placeholder"),
        SEGMENT_WIDTH_LONG,
        date_state.is_year_valid(),
    );

    // Separator text
    let separator_style = |theme: &Theme| text::Style {
        color: Some(theme.extended_palette().background.strong.text),
    };
    let sep1 = Text::new("/").size(typography::BODY).style(separator_style);
    let sep2 = Text::new("/").size(typography::BODY).style(separator_style);

    // Build the row: DD / MM / YYYY
    let mut row = Row::new()
        .spacing(spacing::XXS)
        .align_y(Vertical::Center)
        .push(day_input)
        .push(sep1)
        .push(month_input)
        .push(sep2)
        .push(year_input);

    // Add clear button if a date is set
    if has_value {
        let clear_button = button(icons::fill(action_icons::common::clear()))
            .on_press(Message::ClearDate(target))
            .padding(spacing::XXS)
            .width(Length::Fixed(shared_styles::ICON_SIZE))
            .height(Length::Fixed(shared_styles::ICON_SIZE));

        let clear_tooltip = styles::tooltip::styled(
            clear_button,
            ctx.i18n.tr("filter-date-clear"),
            iced::widget::tooltip::Position::Bottom,
        );

        row = row.push(clear_tooltip);
    }

    row.into()
}

/// Build a single segment input (day, month, or year).
fn build_segment_input<'a>(
    _ctx: &ViewContext<'a>,
    target: DateTarget,
    segment: DateSegment,
    value: &str,
    placeholder: String,
    width: f32,
    is_valid: bool,
) -> Element<'a, Message> {
    let input = text_input(&placeholder, value)
        .on_input(move |v| Message::DateSegmentChanged {
            target,
            segment,
            value: v,
        })
        .on_submit(Message::DateSubmit(target))
        .padding(spacing::XXS)
        .width(Length::Fixed(width))
        .style(move |theme: &Theme, status| {
            use iced::widget::text_input::{Status, Style};

            let palette = theme.extended_palette();

            // Base style based on status
            let mut style = match status {
                Status::Active | Status::Hovered => Style {
                    background: palette.background.base.color.into(),
                    border: Border {
                        color: palette.background.strong.color,
                        width: 1.0,
                        radius: radius::SM.into(),
                    },
                    icon: palette.background.weak.text,
                    placeholder: palette.background.strong.text,
                    value: palette.background.base.text,
                    selection: palette.primary.weak.color,
                },
                Status::Focused { .. } => Style {
                    background: palette.background.base.color.into(),
                    border: Border {
                        color: palette.primary.strong.color,
                        width: 1.0,
                        radius: radius::SM.into(),
                    },
                    icon: palette.background.weak.text,
                    placeholder: palette.background.strong.text,
                    value: palette.background.base.text,
                    selection: palette.primary.weak.color,
                },
                Status::Disabled => Style {
                    background: palette.background.weak.color.into(),
                    border: Border {
                        color: palette.background.strong.color,
                        width: 1.0,
                        radius: radius::SM.into(),
                    },
                    icon: palette.background.strong.text,
                    placeholder: palette.background.strong.text,
                    value: palette.background.strong.text,
                    selection: palette.background.weak.color,
                },
            };

            // Show red border if invalid (and not empty)
            if !is_valid {
                style.border.color = palette.danger.base.color;
            }

            style
        });

    input.into()
}

// =============================================================================
// Option Types for Pick Lists
// =============================================================================

/// Media type option for the pick list.
#[derive(Debug, Clone, PartialEq, Eq)]
struct MediaTypeOption {
    filter: MediaTypeFilter,
    label: String,
}

impl std::fmt::Display for MediaTypeOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label)
    }
}

/// Date field option for the pick list.
#[derive(Debug, Clone, PartialEq, Eq)]
struct DateFieldOption {
    field: DateFilterField,
    label: String,
}

impl std::fmt::Display for DateFieldOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label)
    }
}

// =============================================================================
// Date Utilities
// =============================================================================

/// Seconds per day (24 * 60 * 60).
const SECS_PER_DAY: u64 = 86400;

/// Parse a date string (YYYY-MM-DD format) to `SystemTime`.
/// Returns `None` if the string is empty or invalid.
#[must_use]
pub fn parse_date_string(input: &str) -> Option<SystemTime> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }

    // Try parsing YYYY-MM-DD format
    let parts: Vec<&str> = input.split('-').collect();
    if parts.len() != 3 {
        return None;
    }

    let year: u32 = parts[0].parse().ok()?;
    let month: u32 = parts[1].parse().ok()?;
    let day: u32 = parts[2].parse().ok()?;

    // Basic validation
    if !(1970..=2100).contains(&year) || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }

    // Convert to days since epoch
    let days = ymd_to_days(year, month, day)?;
    let secs = u64::from(days) * SECS_PER_DAY;

    Some(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(secs))
}

/// Format a `SystemTime` as YYYY-MM-DD string (for display).
#[must_use]
pub fn format_system_time(time: SystemTime) -> String {
    use std::time::UNIX_EPOCH;

    let duration = time.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();
    let days_since_epoch = secs / SECS_PER_DAY;
    let (year, month, day) = days_to_ymd(days_since_epoch);

    format!("{year:04}-{month:02}-{day:02}")
}

/// Convert days since epoch to (year, month, day).
fn days_to_ymd(days: u64) -> (u32, u32, u32) {
    // Algorithm based on Howard Hinnant's date algorithms
    // http://howardhinnant.github.io/date_algorithms.html
    let z = days + 719_468;
    let era = z / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    #[allow(clippy::cast_possible_truncation)]
    (y as u32, m as u32, d as u32)
}

/// Convert (year, month, day) to days since epoch.
fn ymd_to_days(year: u32, month: u32, day: u32) -> Option<u32> {
    // Algorithm based on Howard Hinnant's date algorithms
    let y = if month <= 2 {
        year.checked_sub(1)?
    } else {
        year
    };
    let m = if month <= 2 { month + 12 } else { month };
    let era = y / 400;
    let yoe = y - era * 400;
    let doy = (153 * (m - 3) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe;

    // Subtract epoch offset (days from year 0 to 1970-01-01)
    days.checked_sub(719_468)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_dropdown_state_default() {
        let state = FilterDropdownState::new();
        assert!(!state.is_open);
        assert!(state.start_date.is_empty());
        assert!(state.end_date.is_empty());
    }

    #[test]
    fn filter_dropdown_state_toggle() {
        let mut state = FilterDropdownState::new();
        assert!(!state.is_open);

        state.toggle();
        assert!(state.is_open);

        state.toggle();
        assert!(!state.is_open);
    }

    #[test]
    fn filter_dropdown_state_close() {
        let mut state = FilterDropdownState::new();
        state.is_open = true;

        state.close();
        assert!(!state.is_open);
    }

    #[test]
    fn segmented_date_state_default() {
        let state = SegmentedDateState::default();
        assert!(state.is_empty());
        assert!(state.day.is_empty());
        assert!(state.month.is_empty());
        assert!(state.year.is_empty());
    }

    #[test]
    fn segmented_date_state_validation() {
        let mut state = SegmentedDateState::default();

        // Valid day
        state.day = "15".to_string();
        assert!(state.is_day_valid());

        // Invalid day
        state.day = "32".to_string();
        assert!(!state.is_day_valid());

        // Valid month
        state.month = "06".to_string();
        assert!(state.is_month_valid());

        // Invalid month
        state.month = "13".to_string();
        assert!(!state.is_month_valid());

        // Valid year
        state.year = "2024".to_string();
        assert!(state.is_year_valid());

        // Invalid year
        state.year = "1969".to_string();
        assert!(!state.is_year_valid());
    }

    #[test]
    fn segmented_date_state_to_system_time() {
        let mut state = SegmentedDateState::default();
        state.day = "15".to_string();
        state.month = "06".to_string();
        state.year = "2024".to_string();

        let time = state.to_system_time();
        assert!(time.is_some());

        // Verify roundtrip
        let formatted = format_system_time(time.unwrap());
        assert_eq!(formatted, "2024-06-15");
    }

    #[test]
    fn segmented_date_state_incomplete() {
        let mut state = SegmentedDateState::default();
        state.day = "15".to_string();
        // Missing month and year
        assert!(state.to_system_time().is_none());
        assert!(!state.is_complete_and_valid());
    }

    #[test]
    fn filter_dropdown_set_segment() {
        let mut state = FilterDropdownState::new();

        // Set day segment
        let should_advance = state.set_segment(DateTarget::Start, DateSegment::Day, "15".to_string());
        assert!(should_advance); // 2 digits entered
        assert_eq!(state.start_date.day, "15");

        // Set month segment
        let should_advance = state.set_segment(DateTarget::Start, DateSegment::Month, "6".to_string());
        assert!(!should_advance); // Only 1 digit
        assert_eq!(state.start_date.month, "6");

        // Set year segment
        let should_advance = state.set_segment(DateTarget::Start, DateSegment::Year, "2024".to_string());
        assert!(should_advance); // 4 digits entered
        assert_eq!(state.start_date.year, "2024");
    }

    #[test]
    fn filter_dropdown_set_segment_filters_non_digits() {
        let mut state = FilterDropdownState::new();

        state.set_segment(DateTarget::Start, DateSegment::Day, "1a5".to_string());
        assert_eq!(state.start_date.day, "15");

        state.set_segment(DateTarget::Start, DateSegment::Month, "0x6".to_string());
        assert_eq!(state.start_date.month, "06");
    }

    #[test]
    fn filter_dropdown_set_segment_truncates() {
        let mut state = FilterDropdownState::new();

        // Day should be max 2 digits
        state.set_segment(DateTarget::Start, DateSegment::Day, "123".to_string());
        assert_eq!(state.start_date.day, "12");

        // Year should be max 4 digits
        state.set_segment(DateTarget::Start, DateSegment::Year, "20241".to_string());
        assert_eq!(state.start_date.year, "2024");
    }

    #[test]
    fn filter_dropdown_clear_date() {
        let mut state = FilterDropdownState::new();
        state.start_date.day = "15".to_string();
        state.start_date.month = "06".to_string();
        state.start_date.year = "2024".to_string();

        state.clear_date(DateTarget::Start);
        assert!(state.start_date.is_empty());
    }

    #[test]
    fn parse_date_string_valid() {
        let time = parse_date_string("2024-01-15");
        assert!(time.is_some());
    }

    #[test]
    fn parse_date_string_empty() {
        assert!(parse_date_string("").is_none());
        assert!(parse_date_string("  ").is_none());
    }

    #[test]
    fn parse_date_string_invalid_format() {
        assert!(parse_date_string("2024/01/15").is_none());
        assert!(parse_date_string("15-01-2024").is_none());
        assert!(parse_date_string("not a date").is_none());
    }

    #[test]
    fn parse_date_string_invalid_year() {
        assert!(parse_date_string("1969-01-15").is_none());
        assert!(parse_date_string("2101-01-15").is_none());
    }

    #[test]
    fn parse_date_string_invalid_month() {
        assert!(parse_date_string("2024-13-01").is_none());
        assert!(parse_date_string("2024-00-01").is_none());
    }

    #[test]
    fn date_string_roundtrip() {
        let original = "2024-06-15";
        let time = parse_date_string(original).unwrap();
        let formatted = format_system_time(time);
        assert_eq!(formatted, original);
    }

    #[test]
    fn format_system_time_epoch() {
        let formatted = format_system_time(SystemTime::UNIX_EPOCH);
        assert_eq!(formatted, "1970-01-01");
    }
}
