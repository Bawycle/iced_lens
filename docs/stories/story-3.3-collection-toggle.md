# Story 3.3: Collection Toggle Control

**Epic:** 3 - UI Integration
**Status:** Ready for Review
**Priority:** High
**Estimate:** 3-4 hours
**Depends On:** Story 3.1, Story 3.2

---

## Story

**As a** developer,
**I want** to enable/disable diagnostic collection from the UI,
**So that** I can control when resource metrics are being collected.

---

## Acceptance Criteria

1. Toggle switch component for enabling/disabling resource collection
2. Toggle follows existing IcedLens toggle style (see `filter_dropdown.rs`)
3. Toggling starts/stops the `ResourceCollector` (CPU/RAM/disk metrics)
4. UI reflects state change immediately via `CollectionStatus`
5. Toggle state persists across screen navigation (but not app restart for MVP)
6. Clear label indicates toggle purpose ("Enable Resource Collection")
7. Keyboard accessible (Space to toggle when focused - native toggler behavior)

**Note:** Event collection (user actions, errors, warnings) remains always active. The toggle only controls resource metrics collection.

---

## Tasks

- [x] **Task 1:** Integrate `ResourceCollector` into `DiagnosticsCollector` (AC: 3)
  - [x] Add `resource_collector: Option<ResourceCollector>` field
  - [x] Add `resource_metrics_rx: Option<Receiver<ResourceMetrics>>` for receiving metrics
  - [x] Modify `process_pending()` to also drain metrics channel
  - [x] Store received metrics as `DiagnosticEventKind::ResourceSnapshot`

- [x] **Task 2:** Add enable/disable methods to `DiagnosticsCollector` (AC: 3, 4)
  - [x] `enable_resource_collection(&mut self)` - starts ResourceCollector
  - [x] `disable_resource_collection(&mut self)` - stops ResourceCollector
  - [x] Update `get_status()` to return `Enabled` when ResourceCollector is running

- [x] **Task 3:** Implement toggle widget in `diagnostics_screen.rs` (AC: 1, 2, 6)
  - [x] Use `iced::widget::toggler`
  - [x] Label: "Enable Resource Collection" (i18n)
  - [x] Match existing style: `.size(20.0)`
  - [x] Pass current status from ViewContext

- [x] **Task 4:** Add toggle messages (AC: 4)
  - [x] Add `Message::ToggleResourceCollection` to `diagnostics_screen`
  - [x] Add `Event::ToggleResourceCollection(bool)` for parent
  - [x] Handle in `update()` - emit event to App

- [x] **Task 5:** Handle toggle in App (AC: 3, 4, 5)
  - [x] Handle `diagnostics_screen::Event::ToggleResourceCollection`
  - [x] Call `diagnostics.enable_resource_collection()` or `disable_resource_collection()`
  - [x] State persists in `DiagnosticsCollector` during session

- [x] **Task 6:** Update `CollectionStatus` logic (AC: 4)
  - [x] `get_status()` returns `Enabled` when `resource_collector.is_some()` and running
  - [x] `get_status()` returns `Disabled` when stopped or None
  - [x] Include `started_at` from ResourceCollector creation time

- [x] **Task 7:** Add i18n keys (AC: 6)
  - [x] English and French keys for toggle label

- [x] **Task 8:** Write unit tests (AC: 3, 4)
  - [x] Test enable/disable methods
  - [x] Test status transitions
  - [x] Test metrics flow to buffer

- [x] **Task 9:** Run validation
  - [x] `cargo fmt --all`
  - [x] `cargo clippy --all --all-targets -- -D warnings`
  - [x] `cargo test`

- [x] **Task 10:** Commit changes
  - [x] Stage all changes
  - [x] Commit: `feat(diagnostics): add resource collection toggle [Story 3.3]`

---

## Dev Notes

### Source Tree

```
src/
├── diagnostics/
│   ├── mod.rs              # MODIFY: Export new methods
│   ├── collector.rs        # MODIFY: Add ResourceCollector integration
│   └── resource_collector.rs  # REFERENCE: ResourceCollector API (DO NOT MODIFY)
├── app/
│   ├── mod.rs              # MODIFY: Handle toggle event
│   └── view.rs             # REFERENCE: Pass status to ViewContext
├── ui/
│   └── diagnostics_screen.rs  # MODIFY: Add toggle widget, messages
└── assets/i18n/
    ├── en/main.ftl         # MODIFY: Add toggle label
    └── fr/main.ftl         # MODIFY: Add toggle label
```

### Architecture: ResourceCollector Integration

```
┌─────────────────────────────────────────────────────────────┐
│                    DiagnosticsCollector                      │
├─────────────────────────────────────────────────────────────┤
│  buffer: CircularBuffer<DiagnosticEvent>                    │
│  event_rx: Receiver<DiagnosticEvent>     ← DiagnosticsHandle│
│  resource_collector: Option<ResourceCollector>  ← NEW       │
│  resource_metrics_rx: Option<Receiver<ResourceMetrics>> NEW │
├─────────────────────────────────────────────────────────────┤
│  process_pending():                                         │
│    1. Drain event_rx → buffer (existing)                    │
│    2. Drain resource_metrics_rx → buffer as ResourceSnapshot│
└─────────────────────────────────────────────────────────────┘
```

### DiagnosticsCollector Modifications

```rust
// In src/diagnostics/collector.rs

use super::resource_collector::{ResourceCollector, ResourceMetrics, SamplingInterval};
use crossbeam_channel::Receiver;

pub struct DiagnosticsCollector {
    // ... existing fields ...

    /// Optional resource collector for system metrics.
    resource_collector: Option<ResourceCollector>,
    /// Channel to receive metrics from ResourceCollector.
    resource_metrics_rx: Option<Receiver<ResourceMetrics>>,
}

impl DiagnosticsCollector {
    /// Enables resource collection (CPU/RAM/disk metrics).
    ///
    /// Starts a background thread that samples system metrics at the
    /// default interval and sends them to the event buffer.
    pub fn enable_resource_collection(&mut self) {
        if self.resource_collector.is_some() {
            return; // Already enabled
        }

        let (metrics_tx, metrics_rx) = crossbeam_channel::bounded(100);
        let collector = ResourceCollector::start(
            SamplingInterval::default(),
            metrics_tx,
        );

        self.resource_collector = Some(collector);
        self.resource_metrics_rx = Some(metrics_rx);
    }

    /// Disables resource collection.
    ///
    /// Stops the background thread. Existing metrics in buffer are preserved.
    pub fn disable_resource_collection(&mut self) {
        if let Some(mut collector) = self.resource_collector.take() {
            collector.stop();
        }
        self.resource_metrics_rx = None;
    }

    /// Returns true if resource collection is currently enabled.
    #[must_use]
    pub fn is_resource_collection_enabled(&self) -> bool {
        self.resource_collector
            .as_ref()
            .map_or(false, ResourceCollector::is_running)
    }

    /// Returns the current collection status.
    #[must_use]
    pub fn get_status(&self) -> CollectionStatus {
        if self.is_resource_collection_enabled() {
            CollectionStatus::Enabled {
                started_at: self.collection_started_at,
            }
        } else {
            CollectionStatus::Disabled
        }
    }

    /// Processes all pending events from channels.
    ///
    /// Call this periodically (e.g., on each UI tick) to drain the
    /// event channels and store events in the buffer.
    pub fn process_pending(&mut self) {
        // Process diagnostic events (existing)
        while let Ok(event) = self.event_rx.try_recv() {
            self.buffer.push(event);
        }

        // Process resource metrics (NEW)
        if let Some(ref rx) = self.resource_metrics_rx {
            while let Ok(metrics) = rx.try_recv() {
                let event = DiagnosticEvent::new(
                    DiagnosticEventKind::ResourceSnapshot { metrics }
                );
                self.buffer.push(event);
            }
        }
    }
}
```

### Toggle Widget Pattern

```rust
// In src/ui/diagnostics_screen.rs

use iced::widget::toggler;

/// Messages emitted by the diagnostics screen.
#[derive(Debug, Clone)]
pub enum Message {
    BackToViewer,
    RefreshStatus,
    ToggleResourceCollection(bool),  // NEW
}

/// Events propagated to the parent application.
#[derive(Debug, Clone)]
pub enum Event {
    None,
    BackToViewer,
    ToggleResourceCollection(bool),  // NEW
}

/// Process a message and return the corresponding event.
pub fn update(message: &Message) -> Event {
    match message {
        Message::BackToViewer => Event::BackToViewer,
        Message::RefreshStatus => Event::None,
        Message::ToggleResourceCollection(enabled) => {
            Event::ToggleResourceCollection(*enabled)
        }
    }
}

fn build_toggle_section<'a>(ctx: &ViewContext<'a>) -> Element<'a, Message> {
    let is_enabled = matches!(ctx.status, CollectionStatus::Enabled { .. });

    let label = Text::new(ctx.i18n.tr("diagnostics-toggle-label"))
        .size(typography::BODY);

    let toggle = toggler(is_enabled)
        .on_toggle(Message::ToggleResourceCollection)
        .size(20.0);  // Match existing IcedLens style

    Row::new()
        .spacing(spacing::SM)
        .align_y(Vertical::Center)
        .push(label)
        .push(iced::widget::Space::new().width(Length::Fill))
        .push(toggle)
        .into()
}
```

### App Event Handling

```rust
// In src/app/mod.rs (or update.rs)

// Handle diagnostics screen events
diagnostics_screen::Event::ToggleResourceCollection(enabled) => {
    if enabled {
        self.diagnostics.enable_resource_collection();
    } else {
        self.diagnostics.disable_resource_collection();
    }
    Task::none()
}
```

### I18n Keys

```ftl
# English (assets/i18n/en/main.ftl)
diagnostics-toggle-label = Enable Resource Collection

# French (assets/i18n/fr/main.ftl)
diagnostics-toggle-label = Activer la collecte de ressources
```

### Design Decision: Buffer Behavior on Disable

When resource collection is disabled:
- **Existing events in buffer are preserved** (not cleared)
- User can still export the report with collected data
- Only new resource snapshots stop being added

Rationale: Users may want to disable collection but export what was already captured.

---

## Testing

### Unit Tests (in collector.rs)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn enable_resource_collection_starts_collector() {
        let mut collector = DiagnosticsCollector::default();
        assert!(!collector.is_resource_collection_enabled());

        collector.enable_resource_collection();
        assert!(collector.is_resource_collection_enabled());

        collector.disable_resource_collection();
        assert!(!collector.is_resource_collection_enabled());
    }

    #[test]
    fn enable_twice_is_idempotent() {
        let mut collector = DiagnosticsCollector::default();
        collector.enable_resource_collection();
        collector.enable_resource_collection(); // Should not panic or create duplicate
        assert!(collector.is_resource_collection_enabled());
    }

    #[test]
    fn get_status_reflects_collection_state() {
        let mut collector = DiagnosticsCollector::default();
        assert!(matches!(collector.get_status(), CollectionStatus::Disabled));

        collector.enable_resource_collection();
        assert!(matches!(collector.get_status(), CollectionStatus::Enabled { .. }));

        collector.disable_resource_collection();
        assert!(matches!(collector.get_status(), CollectionStatus::Disabled));
    }

    #[test]
    fn process_pending_drains_resource_metrics() {
        let mut collector = DiagnosticsCollector::default();
        collector.enable_resource_collection();

        // Wait for at least one metric
        std::thread::sleep(Duration::from_millis(1500));
        collector.process_pending();

        // Should have at least one ResourceSnapshot event
        let has_resource_snapshot = collector.iter().any(|e| {
            matches!(e.kind, DiagnosticEventKind::ResourceSnapshot { .. })
        });
        assert!(has_resource_snapshot, "Should have captured resource metrics");

        collector.disable_resource_collection();
    }

    #[test]
    fn disable_preserves_existing_events() {
        let mut collector = DiagnosticsCollector::default();
        collector.log_action(UserAction::NavigateNext);
        collector.enable_resource_collection();
        std::thread::sleep(Duration::from_millis(200));
        collector.process_pending();

        let count_before = collector.len();
        collector.disable_resource_collection();

        assert_eq!(collector.len(), count_before, "Events should be preserved");
    }
}
```

### Manual Tests

| Test | Steps | Expected Result |
|------|-------|-----------------|
| Toggle visual | Click toggle on Diagnostics screen | Toggle switches, status changes |
| Enable collection | Enable toggle, wait 5s | Status shows "Enabled", buffer count increases |
| Disable collection | Disable toggle | Status shows "Disabled", buffer stops growing |
| State persistence | Enable, go to Viewer, return to Diagnostics | Toggle still enabled |
| Keyboard toggle | Tab to toggle, press Space | Toggle switches state |
| Buffer preserved | Enable, collect events, disable, check count | Event count unchanged after disable |

---

## Dev Agent Record

### Agent Model Used
Claude Opus 4.5 (claude-opus-4-5-20251101)

### Completion Notes
- Integrated ResourceCollector into DiagnosticsCollector with enable/disable methods
- Added toggler widget following IcedLens style (size 20.0)
- Toggle event handled directly in App::update() since UpdateContext only has DiagnosticsHandle
- Added i18n keys for all 5 languages (en-US, fr, de, es, it)
- All 6 new unit tests pass, 895 total library tests pass
- Full validation: fmt, clippy (no warnings), all tests pass

### Change Log
| Date | Change | Author |
|------|--------|--------|
| 2026-01-14 | Story created | PM |
| 2026-01-14 | PO Validation: Added ResourceCollector integration, Source Tree, code examples, architecture diagram, aligned with CollectionStatus | Sarah (PO) |
| 2026-01-14 | Implementation complete, all tasks done | James (Dev Agent) |

### File List
- `src/diagnostics/collector.rs` - Added ResourceCollector integration, enable/disable methods, updated get_status(), 5 new tests
- `src/ui/diagnostics_screen.rs` - Added toggle widget, ToggleResourceCollection message/event, 1 new test
- `src/app/mod.rs` - Added diagnostics_screen import, toggle event handling
- `src/app/update.rs` - Added ToggleResourceCollection event case
- `assets/i18n/en-US.ftl` - Added diagnostics-toggle-label key
- `assets/i18n/fr.ftl` - Added diagnostics-toggle-label key
- `assets/i18n/de.ftl` - Added diagnostics-toggle-label key
- `assets/i18n/es.ftl` - Added diagnostics-toggle-label key
- `assets/i18n/it.ftl` - Added diagnostics-toggle-label key

---

## QA Results

<!-- QA agent adds results here after review -->

---
