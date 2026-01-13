# Story 1.4: Application State and Operation Capture

**Epic:** 1 - Diagnostics Core & Data Collection
**Status:** Draft
**Priority:** High
**Estimate:** 2-3 hours
**Depends On:** Story 1.1, 1.3

---

## Story

**As a** developer,
**I want** to capture application state changes and internal operations,
**So that** I can understand what the application was doing during issues.

---

## Acceptance Criteria

1. `AppStateEvent` enum defined for key states (media_loading_started, media_loaded, media_failed, video_playing, video_paused, video_seeking, etc.)
2. `AppOperation` enum defined for internal operations (decode_frame, resize_image, apply_filter, etc.)
3. Integration with existing state management to capture transitions
4. Events stored with timestamp and relevant context (e.g., media type, file size category)
5. At least 3 key state transitions and 3 operations instrumented
6. Does not capture sensitive data (paths anonymized or excluded at this stage)
7. Unit tests verify state event creation

---

## Tasks

- [ ] **Task 1:** Define `AppStateEvent` enum
  - [ ] In `src/diagnostics/events.rs`
  - [ ] Variants: MediaLoadingStarted, MediaLoaded, MediaFailed, VideoPlaying, VideoPaused, VideoSeeking, EditorOpened, EditorClosed, etc.
  - [ ] Include optional context (media_type, size_category)

- [ ] **Task 2:** Define `AppOperation` enum
  - [ ] Variants: DecodeFrame, ResizeImage, ApplyFilter, ExportFrame, LoadMetadata, etc.
  - [ ] Include duration_ms for timing info

- [ ] **Task 3:** Expand `DiagnosticEvent`
  - [ ] Add `AppState(AppStateEvent)` variant
  - [ ] Add `Operation(AppOperation)` variant
  - [ ] Both include timestamp

- [ ] **Task 4:** Add `log_state()` and `log_operation()` to DiagnosticsCollector
  - [ ] Similar pattern to `log_action()`
  - [ ] Non-blocking via channel

- [ ] **Task 5:** Define size categories for context
  - [ ] Small (<1MB), Medium (1-10MB), Large (10-100MB), VeryLarge (>100MB)
  - [ ] Avoids leaking exact file sizes

- [ ] **Task 6:** Write unit tests
  - [ ] Test AppStateEvent creation
  - [ ] Test AppOperation with duration
  - [ ] Test size category helper

- [ ] **Task 7:** Run validation
  - [ ] `cargo fmt --all`
  - [ ] `cargo clippy --all --all-targets -- -D warnings`
  - [ ] `cargo test`

- [ ] **Task 8:** Commit changes
  - [ ] Stage all changes
  - [ ] Commit with descriptive message following conventional commits
  - [ ] Reference story number in commit message

---

## Dev Notes

- Actual handler integration deferred
- Size categories preserve privacy while giving diagnostic context
- Duration tracking useful for identifying slow operations

---

## Testing

### Unit Tests
- `events.rs`: AppStateEvent, AppOperation creation
- `collector.rs`: log_state, log_operation

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
