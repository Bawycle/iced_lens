# Story 1.3: User Action Event Capture

**Epic:** 1 - Diagnostics Core & Data Collection
**Status:** Completed
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

1. `UserAction` enum defined for trackable actions (navigate_next, navigate_prev, load_media, seek_video, next_frame, toggle_play, open_settings, fullscreen_exit, fit_to_window_activation, zoom_in, etc.)
2. Integration points identified in existing message handlers
3. `DiagnosticsCollector::log_action()` method implemented
4. Actions stored as `DiagnosticEvent::UserAction` with timestamp and action details
5. Action logging does not block UI thread (uses channel to send to collector)
6. At least 5 key user actions instrumented as proof of concept
7. Unit tests verify action event creation and storage

---

## Tasks

- [x] **Task 1:** Define `UserAction` enum
  - [x] Create in `src/diagnostics/events.rs`
  - [x] Variants: NavigateNext, NavigatePrev, LoadMedia, SeekVideo, TogglePlay, OpenSettings, OpenHelp, OpenAbout, etc.
  - [x] Add optional context field (e.g., seek position)

- [x] **Task 2:** Expand `DiagnosticEvent::UserAction`
  - [x] Include timestamp
  - [x] Include UserAction variant
  - [x] Include optional details string

- [x] **Task 3:** Create `DiagnosticsCollector` struct
  - [x] In `src/diagnostics/collector.rs`
  - [x] Holds CircularBuffer
  - [x] `log_action()` method
  - [x] Channel receiver for events from UI thread

- [x] **Task 4:** Identify integration points
  - [x] Review `src/app/update.rs` for key user actions
  - [x] Document which handlers need instrumentation
  - [x] Do NOT modify handlers yet (integration in later story)

- [x] **Task 5:** Write unit tests
  - [x] Test UserAction enum serialization
  - [x] Test log_action stores correctly
  - [x] Test timestamp is recorded

- [x] **Task 6:** Run validation
  - [x] `cargo fmt --all`
  - [x] `cargo clippy --all --all-targets -- -D warnings`
  - [x] `cargo test`

- [x] **Task 7:** Commit changes
  - [x] Stage all changes
  - [x] Commit with descriptive message following conventional commits
  - [x] Reference story number in commit message

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

## QA Results

### Review Date: 2026-01-13

### Reviewed By: Quinn (Test Architect)

### Code Quality Assessment

Comprehensive UserAction enum (30+ variants) covering all major user interactions. DiagnosticsHandle provides Clone-able, thread-safe logging. Bounded channel for backpressure protection.

### Refactoring Performed

None required.

### Compliance Check

- Coding Standards: ✓
- Project Structure: ✓
- Testing Strategy: ✓ 40 tests in collector.rs
- All ACs Met: ✓ All 7 acceptance criteria verified

### Gate Status

Gate: PASS → docs/qa/gates/1.3-user-action-capture.yml

### Recommended Status

[✓ Ready for Done]

---

## Dev Agent Record

### Agent Model Used
Claude Opus 4.5 (claude-opus-4-5-20251101)

### Completion Notes
- Implemented comprehensive `UserAction` enum with 30+ action variants covering:
  - Navigation (NavigateNext, NavigatePrevious, LoadMedia, DeleteMedia)
  - Playback (TogglePlayback, SeekVideo, StepForward/Backward, SetPlaybackSpeed, ToggleLoop)
  - Audio (SetVolume, ToggleMute)
  - View (ZoomIn/Out, ResetZoom, ToggleFitToWindow, ToggleFullscreen, Rotate)
  - Screen navigation (OpenSettings, OpenHelp, OpenAbout, EnterEditor, ReturnToViewer)
  - Capture/Export (CaptureFrame, ExportFile)
  - Editor (ApplyCrop, ApplyResize, ApplyDeblur, ApplyUpscale, Undo, Redo, SaveImage)
- `DiagnosticsCollector` provides both synchronous (`log_action`) and async (`DiagnosticsHandle`) logging
- `DiagnosticsHandle` is Clone and uses bounded channel for backpressure protection
- Integration points documented (handlers NOT modified per story scope)
- All 48 diagnostics tests pass

### Integration Points Identified (for later stories)
- `handle_viewer_message`: NavigateNext/Previous, OpenSettings, EnterEditor, ToggleFullscreen
- `handle_navigate_next/previous`: Navigation actions
- `handle_delete_current_media`: DeleteMedia
- `handle_settings_message`: Settings changes
- `handle_editor_message`: Editor actions (Save, Deblur, Upscale)
- `handle_navbar_message`: OpenSettings, OpenHelp, OpenAbout, EnterEditor
- `handle_open_file_dialog_result`: LoadMedia (file_dialog)
- `handle_file_dropped`: LoadMedia (drag_drop)
- `handle_capture_frame`: CaptureFrame

### Change Log
| Date | Change | Author |
|------|--------|--------|
| 2026-01-13 | Initial implementation | Claude Opus 4.5 |

### File List
- `src/diagnostics/events.rs` - Added UserAction enum, updated DiagnosticEventKind::UserAction
- `src/diagnostics/collector.rs` - New file with DiagnosticsCollector and DiagnosticsHandle
- `src/diagnostics/mod.rs` - Added collector module exports

---
