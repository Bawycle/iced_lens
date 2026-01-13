# Story 1.3: User Action Event Capture

**Epic:** 1 - Diagnostics Core & Data Collection
**Status:** Draft
**Priority:** High
**Estimate:** 2-3 hours
**Depends On:** Story 1.1

---

## Story

**As a** developer,
**I want** to capture user actions as diagnostic events,
**So that** I can understand what the user was doing when issues occurred.

---

## Acceptance Criteria

1. `UserAction` enum defined for trackable actions (navigate_next, navigate_prev, load_media, seek_video, toggle_play, open_settings, etc.)
2. Integration points identified in existing message handlers
3. `DiagnosticsCollector::log_action()` method implemented
4. Actions stored as `DiagnosticEvent::UserAction` with timestamp and action details
5. Action logging does not block UI thread (uses channel to send to collector)
6. At least 5 key user actions instrumented as proof of concept
7. Unit tests verify action event creation and storage

---

## Tasks

- [ ] **Task 1:** Define `UserAction` enum
  - [ ] Create in `src/diagnostics/events.rs`
  - [ ] Variants: NavigateNext, NavigatePrev, LoadMedia, SeekVideo, TogglePlay, OpenSettings, OpenHelp, OpenAbout, etc.
  - [ ] Add optional context field (e.g., seek position)

- [ ] **Task 2:** Expand `DiagnosticEvent::UserAction`
  - [ ] Include timestamp
  - [ ] Include UserAction variant
  - [ ] Include optional details string

- [ ] **Task 3:** Create `DiagnosticsCollector` struct
  - [ ] In `src/diagnostics/collector.rs`
  - [ ] Holds CircularBuffer
  - [ ] `log_action()` method
  - [ ] Channel receiver for events from UI thread

- [ ] **Task 4:** Identify integration points
  - [ ] Review `src/app/update.rs` for key user actions
  - [ ] Document which handlers need instrumentation
  - [ ] Do NOT modify handlers yet (integration in later story)

- [ ] **Task 5:** Write unit tests
  - [ ] Test UserAction enum serialization
  - [ ] Test log_action stores correctly
  - [ ] Test timestamp is recorded

- [ ] **Task 6:** Run validation
  - [ ] `cargo fmt --all`
  - [ ] `cargo clippy --all --all-targets -- -D warnings`
  - [ ] `cargo test`

- [ ] **Task 7:** Commit changes
  - [ ] Stage all changes
  - [ ] Commit with descriptive message following conventional commits
  - [ ] Reference story number in commit message

---

## Dev Notes

- Actual handler integration deferred to avoid scope creep
- Channel-based design allows non-blocking logging
- UserAction should be `Clone + Debug + Serialize`

---

## Testing

### Unit Tests
- `events.rs`: UserAction creation
- `collector.rs`: log_action behavior

### Integration Tests
- None (handler integration in later story)

---

## Dev Agent Record

### Agent Model Used
<!-- Record which AI model completed this story -->

### Completion Notes
<!-- Dev agent adds notes here during implementation -->

### Change Log
| Date | Change | Author |
|------|--------|--------|

### File List
<!-- Files created or modified -->

---
