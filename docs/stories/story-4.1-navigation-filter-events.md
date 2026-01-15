# Story 4.1: Navigation Filter Diagnostic Events

**Epic:** 4 - Diagnostics Collection Completeness
**Status:** Ready
**Priority:** P1
**Estimate:** 3-4 hours
**Depends On:** Story 4.0

---

## Story

**As a** developer analyzing user behavior,
**I want** filter activation/deactivation events captured,
**So that** I can understand how users use navigation filters.

---

## Acceptance Criteria

1. `FilterChangeType` enum defined in `events.rs` with variants: `MediaType`, `DateRangeEnabled`, `DateRangeDisabled`, `DateFieldChanged`, `DateBoundSet`, `DateBoundCleared`
2. `FilterChanged` AppStateEvent defined with: `filter_type`, `previous_active`, `new_active`, `filtered_count`, `total_count`
3. `FilterCleared` AppStateEvent defined with: `had_media_type_filter`, `had_date_filter`
4. Collection point in filter change handler at handler level
5. Events emitted for all filter operations: set, modify, clear
6. Integration tests verify event capture for each filter operation
7. No performance regression (< 1ms overhead per event)

---

## Tasks

- [ ] **Task 1:** Define `FilterChangeType` enum (AC: 1)
  - [ ] Add to `src/diagnostics/events.rs`
  - [ ] Variants: `MediaType { from: String, to: String }` (serialize MediaTypeFilter)
  - [ ] Variants: `DateRangeEnabled`, `DateRangeDisabled`
  - [ ] Variants: `DateFieldChanged { field: String }` (serialize DateFilterField)
  - [ ] Variants: `DateBoundSet { target: String }`, `DateBoundCleared { target: String }`
  - [ ] Derive `Debug, Clone, Serialize`
  - [ ] Note: Use String serialization to avoid coupling diagnostics to filter module

- [ ] **Task 2:** Define `FilterChanged` event (AC: 2)
  - [ ] Add variant to `AppStateEvent` enum
  - [ ] Fields: `filter_type: FilterChangeType`
  - [ ] Fields: `previous_active: bool`, `new_active: bool`
  - [ ] Fields: `filtered_count: usize`, `total_count: usize`

- [ ] **Task 3:** Define `FilterCleared` event (AC: 3)
  - [ ] Add variant to `AppStateEvent` enum
  - [ ] Fields: `had_media_type_filter: bool`, `had_date_filter: bool`

- [ ] **Task 4:** Add collection points in `handle_filter_changed()` (AC: 4, 5)
  - [ ] Location: `src/app/update.rs:1596`
  - [ ] Capture `previous_active` and counts BEFORE applying filter change
  - [ ] Emit event AFTER `ctx.media_navigator.set_filter(filter)` call
  - [ ] Handle each `filter_dropdown::Message` variant appropriately

- [ ] **Task 5:** Add FilterChanged for MediaTypeFilter (AC: 4, 5)
  - [ ] In `MediaTypeChanged(media_type)` branch
  - [ ] Emit `FilterChanged { filter_type: FilterChangeType::MediaType { from, to }, ... }`

- [ ] **Task 6:** Add FilterChanged for DateRangeFilter (AC: 4, 5)
  - [ ] In `ToggleDateFilter(enabled)` branch: emit `DateRangeEnabled` or `DateRangeDisabled`
  - [ ] In `DateFieldChanged(field)` branch: emit `DateFieldChanged`
  - [ ] In `DateSubmit(target)` branch: emit `DateBoundSet`
  - [ ] In `ClearDate(target)` branch: emit `DateBoundCleared`

- [ ] **Task 7:** Add FilterCleared for reset (AC: 4, 5)
  - [ ] In `ResetFilters` branch
  - [ ] Capture which filters were active before reset
  - [ ] Emit `FilterCleared { had_media_type_filter, had_date_filter }`

- [ ] **Task 8:** Add tests in `update.rs` `#[cfg(test)]` module (AC: 6)
  - [ ] Test `FilterChanged` for MediaType filter
  - [ ] Test `FilterChanged` for DateRange enable/disable
  - [ ] Test `FilterChanged` for date bound changes
  - [ ] Test `FilterCleared` on reset

- [ ] **Task 9:** Performance verification (AC: 7)
  - [ ] Verify event emission uses non-blocking channel
  - [ ] Ensure < 1ms per event (channel send is ~nanoseconds)

- [ ] **Task 10:** Run validation
  - [ ] `cargo fmt --all`
  - [ ] `cargo clippy --all --all-targets -- -D warnings`
  - [ ] `cargo test`

---

## Dev Notes

### Source Tree

```
src/
├── diagnostics/
│   └── events.rs           # MODIFY: Add FilterChangeType, FilterChanged, FilterCleared
├── media/
│   └── filter.rs           # REFERENCE: MediaTypeFilter, DateRangeFilter, DateFilterField
├── app/
│   └── update.rs           # MODIFY: Add collection points in handle_filter_changed() (line 1596)
└── ui/
    └── viewer/
        └── filter_dropdown.rs  # REFERENCE: Message enum, DateTarget enum
```

**Note:** Tests are in-file using `#[cfg(test)]` modules per coding standards.

### Existing Filter Types (Reference)

From `src/media/filter.rs`:

```rust
// MediaTypeFilter - line 46
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum MediaTypeFilter {
    #[default]
    All,
    ImagesOnly,
    VideosOnly,
}

// DateFilterField - line 81 (ALREADY EXISTS - do not recreate)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum DateFilterField {
    #[default]
    Modified,
    Created,
}

// DateRangeFilter - line 96
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DateRangeFilter {
    pub field: DateFilterField,
    pub start: Option<SystemTime>,
    pub end: Option<SystemTime>,
}

// MediaFilter - line 166 (composite)
pub struct MediaFilter {
    pub media_type: MediaTypeFilter,
    pub date_range: Option<DateRangeFilter>,
}
```

From `src/ui/viewer/filter_dropdown.rs`:

```rust
// DateTarget - ALREADY EXISTS in filter_dropdown module
pub enum DateTarget {
    Start,
    End,
}
```

### Proposed Event Structure

```rust
// New enum for filter change types (in events.rs)
#[derive(Debug, Clone, Serialize)]
pub enum FilterChangeType {
    MediaType {
        from: String,  // "all", "images-only", "videos-only"
        to: String,
    },
    DateRangeEnabled,
    DateRangeDisabled,
    DateFieldChanged { field: String },  // "modified" or "created"
    DateBoundSet { target: String },     // "start" or "end"
    DateBoundCleared { target: String },
}

// New AppStateEvent variants
AppStateEvent::FilterChanged {
    filter_type: FilterChangeType,
    previous_active: bool,
    new_active: bool,
    filtered_count: usize,
    total_count: usize,
}

AppStateEvent::FilterCleared {
    had_media_type_filter: bool,
    had_date_filter: bool,
}
```

### Target Handler: `handle_filter_changed()` (update.rs:1596)

The handler already exists and processes `filter_dropdown::Message`. Add logging before the final `set_filter()` call:

```rust
fn handle_filter_changed(
    ctx: &mut UpdateContext<'_>,
    msg: filter_dropdown::Message,
) -> Task<Message> {
    use crate::media::filter::{DateRangeFilter, MediaFilter};
    use filter_dropdown::DateTarget;

    // === ADD: Capture state BEFORE change ===
    let previous_active = ctx.media_navigator.filter().is_active();
    let previous_media_type = ctx.media_navigator.filter().media_type;
    let previous_date_active = ctx.media_navigator.filter().date_range.is_some();

    // Clone current filter to modify
    let mut filter = ctx.media_navigator.filter().clone();

    // Determine filter_type for logging
    let filter_type: Option<FilterChangeType> = match msg {
        filter_dropdown::Message::MediaTypeChanged(media_type) => {
            let from = format!("{:?}", previous_media_type).to_lowercase();
            let to = format!("{:?}", media_type).to_lowercase();
            filter.media_type = media_type;
            Some(FilterChangeType::MediaType { from, to })
        }
        filter_dropdown::Message::ToggleDateFilter(enabled) => {
            if enabled {
                filter.date_range = Some(DateRangeFilter::default());
                Some(FilterChangeType::DateRangeEnabled)
            } else {
                filter.date_range = None;
                Some(FilterChangeType::DateRangeDisabled)
            }
        }
        // ... other cases
        filter_dropdown::Message::ResetFilters => {
            // Handle separately as FilterCleared
            let had_media_type = previous_media_type.is_active();
            let had_date = previous_date_active;
            filter = MediaFilter::default();

            ctx.diagnostics.log_state(AppStateEvent::FilterCleared {
                had_media_type_filter: had_media_type,
                had_date_filter: had_date,
            });
            None // Don't emit FilterChanged
        }
        _ => None,
    };

    // Update the navigator's filter
    ctx.media_navigator.set_filter(filter);

    // === ADD: Emit diagnostic event ===
    if let Some(ft) = filter_type {
        ctx.diagnostics.log_state(AppStateEvent::FilterChanged {
            filter_type: ft,
            previous_active,
            new_active: ctx.media_navigator.filter().is_active(),
            filtered_count: ctx.media_navigator.filtered_count(),
            total_count: ctx.media_navigator.total_count(),
        });
    }

    // ... rest of handler
}
```

### Diagnostics API Pattern

Use the existing pattern from `update.rs`:

```rust
ctx.diagnostics.log_state(AppStateEvent::FilterChanged { ... });
```

---

## Testing

### Testing Standards

Per `docs/architecture/coding-standards.md`:
- Unit tests in same file using `#[cfg(test)]` module
- Tests go in `src/app/update.rs` or `src/diagnostics/events.rs`
- Use Clippy pedantic

### Test Location

- Event serialization tests: `src/diagnostics/events.rs` `#[cfg(test)]` module
- Handler integration tests: `src/app/update.rs` `#[cfg(test)]` module (if exists)

### Test Examples

```rust
// In src/diagnostics/events.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_change_type_serializes_correctly() {
        let change = FilterChangeType::MediaType {
            from: "all".to_string(),
            to: "images-only".to_string(),
        };
        let json = serde_json::to_string(&change).unwrap();
        assert!(json.contains("images-only"));
    }

    #[test]
    fn filter_changed_event_serializes() {
        let event = AppStateEvent::FilterChanged {
            filter_type: FilterChangeType::DateRangeEnabled,
            previous_active: false,
            new_active: true,
            filtered_count: 10,
            total_count: 50,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("DateRangeEnabled"));
    }

    #[test]
    fn filter_cleared_event_serializes() {
        let event = AppStateEvent::FilterCleared {
            had_media_type_filter: true,
            had_date_filter: false,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("had_media_type_filter"));
    }
}
```

### Manual Verification

| Test | Steps | Expected Result |
|------|-------|-----------------|
| MediaType filter | 1. Change filter to "Images Only"<br>2. Export diagnostics | `FilterChanged` with `MediaType { from: "all", to: "images-only" }` |
| Date filter enable | 1. Enable date filter<br>2. Export diagnostics | `FilterChanged` with `DateRangeEnabled` |
| Reset filters | 1. Set filters active<br>2. Click "Reset"<br>3. Export | `FilterCleared` with correct flags |

---

## Change Log

| Date | Version | Description | Author |
|------|---------|-------------|--------|
| 2025-01-15 | 1.0 | Story created from architecture review | Sarah (PO) |
| 2025-01-15 | 1.1 | PO Validation: Fixed source location (filter.rs not navigator.rs), removed duplicate Task 2 (DateFilterField/DateTarget already exist), fixed Source Tree, added exact handler location (line 1596), added complete implementation example | Sarah (PO) |

---

## Dev Agent Record

### Agent Model Used
_To be filled by Dev Agent_

### Debug Log References
_To be filled by Dev Agent_

### Completion Notes
_To be filled by Dev Agent_

### File List
_To be filled by Dev Agent_

---

## QA Results

_To be filled by QA Agent_

---
