# Story 3.2: Collection Status Display

**Epic:** 3 - UI Integration
**Status:** Ready for Review
**Priority:** High
**Estimate:** 2-3 hours
**Depends On:** Story 3.1

---

## Story

**As a** developer,
**I want** to see the current collection status on the Diagnostics screen,
**So that** I know whether diagnostics are active and collecting data.

---

## Acceptance Criteria

1. Status indicator shows: Disabled, Enabled (Collecting), or Error state
2. Visual indicator uses appropriate colors (following design tokens)
3. Status text describes current state clearly
4. Shows collection duration since app start (e.g., "Running for 5m 32s")
5. Shows buffer fill level (e.g., "Buffer: 847 events")
6. Status updates in real-time (subscription refreshes every 1s)
7. Accessible: status text is descriptive and visible (no hidden state)

**Note:** The toggle to change status is implemented in Story 3.3. This story only displays the current state.

---

## Tasks

- [x] **Task 1:** Add `CollectionStatus` to diagnostics module (AC: 1)
  - [x] Create enum in `src/diagnostics/mod.rs`:
    ```rust
    pub enum CollectionStatus {
        Disabled,
        Enabled { started_at: Instant },
        Error { message: String },
    }
    ```
  - [x] Export from module

- [x] **Task 2:** Add status query methods to `DiagnosticsCollector` (AC: 1, 4, 5)
  - [x] Add `resource_collector: Option<ResourceCollector>` field (prepared for Story 3.3)
  - [x] Add `get_status(&self) -> CollectionStatus` method
  - [x] Add `get_event_count(&self) -> usize` (already exists as `len()`)
  - [x] Add `get_collection_duration(&self) -> Duration` method

- [x] **Task 3:** Add status section to `diagnostics_screen.rs` (AC: 1, 2, 3)
  - [x] Create `build_status_section()` function
  - [x] Status indicator dot with color:
    - Enabled: `palette::SUCCESS_500` (green)
    - Disabled: `palette::GRAY_400` (gray)
    - Error: `palette::ERROR_500` (red)
  - [x] Status text: "Collection: Enabled" / "Disabled" / "Error: {message}"

- [x] **Task 4:** Implement duration display (AC: 4)
  - [x] Format duration as "Xh Ym Zs" or "Xm Zs" (if < 1 hour)
  - [x] Create `format_duration(duration: Duration) -> String` helper
  - [x] Use i18n keys for labels

- [x] **Task 5:** Implement buffer count display (AC: 5)
  - [x] Display "Buffer: X events"
  - [x] Use i18n key: `diagnostics-buffer-count`

- [x] **Task 6:** Add subscription for real-time updates (AC: 6)
  - [x] Add `Message::RefreshStatus` variant to `diagnostics_screen`
  - [x] Add subscription in `App::subscription()` when on Diagnostics screen
  - [x] Poll every 1 second using `iced::time::every()`
  - [x] Update ViewContext with fresh status on each tick

- [x] **Task 7:** Update ViewContext and view integration (AC: 1-7)
  - [x] Add status fields to `diagnostics_screen::ViewContext`:
    - `status: CollectionStatus`
    - `event_count: usize`
    - `collection_duration: Duration`
  - [x] Pass data from App to ViewContext in `view.rs`

- [x] **Task 8:** Add i18n keys (AC: 3, 4, 5)
  - [x] English keys in `assets/i18n/en-US.ftl`
  - [x] French keys in `assets/i18n/fr.ftl`
  - [x] German keys in `assets/i18n/de.ftl`
  - [x] Spanish keys in `assets/i18n/es.ftl`
  - [x] Italian keys in `assets/i18n/it.ftl`

- [x] **Task 9:** Write unit tests (AC: 1, 4)
  - [x] Test `CollectionStatus` enum creation
  - [x] Test `format_duration()` helper
  - [x] Test status display logic

- [x] **Task 10:** Run validation
  - [x] `cargo fmt --all`
  - [x] `cargo clippy --all --all-targets -- -D warnings`
  - [x] `cargo test`

- [x] **Task 11:** Commit changes
  - [x] Stage all changes
  - [x] Commit: `feat(ui): add collection status display [Story 3.2]`

---

## Dev Notes

### Source Tree

```
src/
├── diagnostics/
│   ├── mod.rs              # MODIFY: Add CollectionStatus enum, export
│   └── collector.rs        # MODIFY: Add get_status(), get_collection_duration()
├── app/
│   ├── mod.rs              # MODIFY: Add subscription for Diagnostics screen
│   └── view.rs             # MODIFY: Pass status to diagnostics_screen::ViewContext
├── ui/
│   ├── diagnostics_screen.rs  # MODIFY: Add status section, Message::RefreshStatus
│   └── design_tokens.rs    # REFERENCE: palette colors (DO NOT MODIFY)
└── assets/i18n/
    ├── en/main.ftl         # MODIFY: Add i18n keys
    └── fr/main.ftl         # MODIFY: Add i18n keys
```

### CollectionStatus Enum

```rust
// In src/diagnostics/mod.rs (or collector.rs)

use std::time::Instant;

/// Current status of diagnostic data collection.
#[derive(Debug, Clone)]
pub enum CollectionStatus {
    /// Resource collection is disabled (ResourceCollector not running).
    Disabled,
    /// Resource collection is active.
    Enabled {
        /// When collection started (monotonic).
        started_at: Instant,
    },
    /// Resource collection encountered an error.
    Error {
        /// Error description.
        message: String,
    },
}
```

### DiagnosticsCollector Additions

```rust
// In src/diagnostics/collector.rs

impl DiagnosticsCollector {
    /// Returns the current collection status.
    ///
    /// Note: Until Story 3.3 integrates ResourceCollector, this always
    /// returns Disabled for resource collection. Event collection is
    /// always active.
    #[must_use]
    pub fn get_status(&self) -> CollectionStatus {
        // Placeholder until ResourceCollector is integrated in Story 3.3
        // For now, report as Disabled since no resource metrics are being collected
        CollectionStatus::Disabled
    }

    /// Returns how long the collector has been running.
    #[must_use]
    pub fn get_collection_duration(&self) -> std::time::Duration {
        self.collection_started_at.elapsed()
    }
}
```

### Status Section UI

```rust
// In src/ui/diagnostics_screen.rs

fn build_status_section<'a>(ctx: &ViewContext<'a>) -> Element<'a, Message> {
    use crate::ui::design_tokens::{palette, spacing, typography};

    // Status indicator color
    let status_color = match &ctx.status {
        CollectionStatus::Enabled { .. } => palette::SUCCESS_500,
        CollectionStatus::Disabled => palette::GRAY_400,
        CollectionStatus::Error { .. } => palette::ERROR_500,
    };

    // Status text
    let status_text = match &ctx.status {
        CollectionStatus::Enabled { .. } => ctx.i18n.tr("diagnostics-status-enabled"),
        CollectionStatus::Disabled => ctx.i18n.tr("diagnostics-status-disabled"),
        CollectionStatus::Error { message } => format!("{}: {}",
            ctx.i18n.tr("diagnostics-status-error"), message),
    };

    // Duration text
    let duration_text = format_duration(ctx.collection_duration);

    // Buffer count
    let buffer_text = ctx.i18n.tr_with_args(
        "diagnostics-buffer-count",
        &[("count", ctx.event_count.into())],
    );

    // Build the section...
    Column::new()
        .spacing(spacing::SM)
        .push(/* status indicator + text */)
        .push(Text::new(duration_text))
        .push(Text::new(buffer_text))
        .into()
}
```

### Duration Formatting

```rust
/// Formats a duration for display.
///
/// - Under 1 hour: "Xm Ys"
/// - Over 1 hour: "Xh Ym Zs"
fn format_duration(duration: std::time::Duration) -> String {
    let total_secs = duration.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else {
        format!("{}m {}s", minutes, seconds)
    }
}
```

### Subscription Pattern

```rust
// In src/app/mod.rs, within App::subscription()

fn subscription(&self) -> Subscription<Message> {
    // ... existing subscriptions ...

    // Diagnostics screen refresh subscription
    let diagnostics_sub = if self.screen == Screen::Diagnostics {
        time::every(Duration::from_secs(1))
            .map(|_| Message::Diagnostics(diagnostics_screen::Message::RefreshStatus))
    } else {
        Subscription::none()
    };

    Subscription::batch([
        event_sub,
        tick_sub,
        video_sub,
        editor_sub,
        diagnostics_sub,  // NEW
    ])
}
```

### I18n Keys

```ftl
# English (assets/i18n/en/main.ftl)
diagnostics-status-enabled = Collection: Enabled
diagnostics-status-disabled = Collection: Disabled
diagnostics-status-error = Collection: Error
diagnostics-running-for = Running for { $duration }
diagnostics-buffer-count = Buffer: { $count } events

# French (assets/i18n/fr/main.ftl)
diagnostics-status-enabled = Collecte : Activée
diagnostics-status-disabled = Collecte : Désactivée
diagnostics-status-error = Collecte : Erreur
diagnostics-running-for = En cours depuis { $duration }
diagnostics-buffer-count = Buffer : { $count } événements
```

### Color Tokens Reference

| State | Token | Color |
|-------|-------|-------|
| Enabled | `palette::SUCCESS_500` | Green (#43B367) |
| Disabled | `palette::GRAY_400` | Gray (#666666) |
| Error | `palette::ERROR_500` | Red (#E53935) |

### Note on Initial State

Until Story 3.3 integrates `ResourceCollector`:
- `get_status()` will return `CollectionStatus::Disabled`
- Event count and duration will still display correctly
- The UI is ready for when the toggle enables resource collection

---

## Testing

### Unit Tests (in collector.rs and diagnostics_screen.rs)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn format_duration_under_one_hour() {
        let duration = Duration::from_secs(5 * 60 + 32); // 5m 32s
        assert_eq!(format_duration(duration), "5m 32s");
    }

    #[test]
    fn format_duration_over_one_hour() {
        let duration = Duration::from_secs(2 * 3600 + 15 * 60 + 45); // 2h 15m 45s
        assert_eq!(format_duration(duration), "2h 15m 45s");
    }

    #[test]
    fn format_duration_zero() {
        let duration = Duration::from_secs(0);
        assert_eq!(format_duration(duration), "0m 0s");
    }

    #[test]
    fn collection_status_disabled_default() {
        let collector = DiagnosticsCollector::default();
        assert!(matches!(collector.get_status(), CollectionStatus::Disabled));
    }

    #[test]
    fn collection_duration_increases() {
        let collector = DiagnosticsCollector::default();
        std::thread::sleep(Duration::from_millis(100));
        let duration = collector.get_collection_duration();
        assert!(duration.as_millis() >= 100);
    }
}
```

### Manual Tests

| Test | Steps | Expected Result |
|------|-------|-----------------|
| Status display | Open Diagnostics screen | Shows "Collection: Disabled" with gray indicator |
| Duration updates | Wait 5 seconds on screen | Duration increments each second |
| Buffer count | Perform actions in app, return to Diagnostics | Event count increases |
| Color accuracy | Compare indicator colors | Green/Gray/Red match design tokens |

---

## Dev Agent Record

### Agent Model Used
Claude Opus 4.5 (claude-opus-4-5-20251101)

### Completion Notes
- Implemented `CollectionStatus` enum in `diagnostics/mod.rs` with Disabled, Enabled, and Error variants
- Added `get_status()` and `get_collection_duration()` methods to `DiagnosticsCollector`
- Built status section UI with colored indicator dot using design tokens
- Added `format_duration()` helper function with unit tests
- Created 1-second polling subscription for real-time status updates
- Updated `ViewContext` and view.rs to pass diagnostics data to the screen
- Added i18n keys to all 5 language files (en-US, fr, de, es, it)
- All tests pass (891 unit tests + 37 integration tests)

### Change Log
| Date | Change | Author |
|------|--------|--------|
| 2026-01-14 | Story created | PM |
| 2026-01-14 | PO Validation: Added Source Tree, code examples, fixed accessibility (removed ARIA), clarified ResourceCollector integration deferred to 3.3 | Sarah (PO) |
| 2026-01-14 | Implementation complete | James (Dev Agent - Claude Opus 4.5) |

### File List
- `src/diagnostics/mod.rs` - Added `CollectionStatus` enum
- `src/diagnostics/collector.rs` - Added `get_status()`, `get_collection_duration()` methods and tests
- `src/ui/diagnostics_screen.rs` - Added status section UI, `format_duration()`, `Message::RefreshStatus`
- `src/app/mod.rs` - Added diagnostics subscription to batch, pass status data to view
- `src/app/view.rs` - Added diagnostics fields to `ViewContext`, updated `view_diagnostics()`
- `src/app/subscription.rs` - Added `create_diagnostics_subscription()` function
- `assets/i18n/en-US.ftl` - Added diagnostics status i18n keys
- `assets/i18n/fr.ftl` - Added diagnostics status i18n keys (French)
- `assets/i18n/de.ftl` - Added diagnostics status i18n keys (German)
- `assets/i18n/es.ftl` - Added diagnostics status i18n keys (Spanish)
- `assets/i18n/it.ftl` - Added diagnostics status i18n keys (Italian)

---

## QA Results

<!-- QA agent adds results here after review -->

---
