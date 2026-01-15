# Story 4.0: Fix Collection Point Architectural Violations

**Epic:** 4 - Diagnostics Collection Completeness
**Status:** Complete
**Priority:** P0 (Prerequisite)
**Estimate:** 2-3 hours
**Depends On:** None

---

## Story

**As a** developer maintaining the diagnostics system,
**I want** all collection points to be at the handler level,
**So that** UI components remain decoupled from diagnostics and instrumentation is consistent.

---

## Acceptance Criteria

1. `MediaLoadingStarted` event moved from `component.rs:652` to `update.rs` handler
2. `MediaLoaded` event moved from `component.rs:866` to `update.rs` handler
3. `MediaFailed` event moved from `component.rs:946` to `update.rs` handler
4. `TogglePlayback` logging moved from `component.rs:1147` to `update.rs` handler
5. `SeekVideo` logging moved from `component.rs:1197` to `update.rs` handler
6. All 5 collection points removed from `ui/viewer/component.rs`
7. No functional regression in event capture
8. Integration tests verify events still captured correctly

---

## Tasks

- [x] **Task 1:** Analyze current collection points (AC: 1-5)
  - [x] Document exact event emission context in `component.rs`
  - [x] Identify corresponding handler locations in `update.rs`
  - [x] Map message flow from UI to App handler

- [x] **Task 2:** Relocate `MediaLoadingStarted` (AC: 1, 6)
  - [x] Identify handler that processes media loading initiation
  - [x] Add `log_state_event(AppStateEvent::MediaLoadingStarted {...})` in handler
  - [x] Remove emission from `component.rs:652`

- [x] **Task 3:** Relocate `MediaLoaded` (AC: 2, 6)
  - [x] Identify handler that receives media load completion
  - [x] Add `log_state_event(AppStateEvent::MediaLoaded {...})` in handler
  - [x] Remove emission from `component.rs:866`

- [x] **Task 4:** Relocate `MediaFailed` (AC: 3, 6)
  - [x] Identify handler that receives media load failure
  - [x] Add `log_state_event(AppStateEvent::MediaFailed {...})` in handler
  - [x] Remove emission from `component.rs:946`

- [x] **Task 5:** Relocate `TogglePlayback` logging (AC: 4, 6)
  - [x] Identify handler for playback toggle messages
  - [x] Add `log_action(UserAction::TogglePlayback)` in handler
  - [x] Remove emission from `component.rs:1147`

- [x] **Task 6:** Relocate `SeekVideo` logging (AC: 5, 6)
  - [x] Identify handler for seek messages
  - [x] Add `log_action(UserAction::SeekVideo {...})` in handler
  - [x] Remove emission from `component.rs:1197`

- [x] **Task 7:** Verify no diagnostics imports remain in UI (AC: 6)
  - [x] Check `component.rs` has no `use crate::diagnostics::*` related to these events
  - [x] Ensure UI component is fully decoupled from diagnostics

- [x] **Task 8:** Add/update tests (AC: 7, 8)
  - [x] Add tests in `update.rs` `#[cfg(test)]` module (per coding standards)
  - [x] Test `MediaLoadingStarted` captured from handler
  - [x] Test `MediaLoaded` captured from handler
  - [x] Test `MediaFailed` captured from handler
  - [x] Test `TogglePlayback` captured from handler
  - [x] Test `SeekVideo` captured from handler

- [x] **Task 9:** Run validation
  - [x] `cargo fmt --all`
  - [x] `cargo clippy --all --all-targets -- -D warnings`
  - [x] `cargo test`
  - [x] Manual verification: events still appear in diagnostic reports

---

## Dev Notes

### Source Tree

```
src/
├── app/
│   └── update.rs           # MODIFY: Add relocated collection points
├── ui/
│   └── viewer/
│       └── component.rs    # MODIFY: Remove 5 collection points
└── diagnostics/
    ├── events.rs           # REFERENCE: Event definitions (tests in-file)
    ├── collector.rs        # REFERENCE: DiagnosticsHandle API (tests in-file)
    └── mod.rs              # REFERENCE: Module exports
```

**Note:** Tests are in-file using `#[cfg(test)]` modules per coding standards, NOT in a separate `tests/` folder.

### Current Violations (from Audit)

| # | File | Line | Event | Target Location |
|---|------|------|-------|-----------------|
| V1 | `component.rs` | 652 | `MediaLoadingStarted` | Handler for media load request |
| V2 | `component.rs` | 866 | `MediaLoaded` | Handler for media load completion |
| V3 | `component.rs` | 946 | `MediaFailed` | Handler for media load error |
| V4 | `component.rs` | 1147 | `TogglePlayback` | Handler for playback toggle message |
| V5 | `component.rs` | 1197 | `SeekVideo` | Handler for seek message |

### Target Handler Locations

The events need to be moved to handlers in `update.rs`. Here's guidance on where to place them:

| Event | Message Flow | Target Handler Strategy |
|-------|--------------|------------------------|
| `MediaLoadingStarted` | Viewer emits `Effect::StartLoadMedia` | Handler receiving this effect, before async load starts |
| `MediaLoaded` | Async task returns success | Handler processing `LoadResult::Success` or equivalent |
| `MediaFailed` | Async task returns error | Handler processing `LoadResult::Error` or equivalent |
| `TogglePlayback` | `Message::Viewer(ViewerMessage::VideoControls(VM::TogglePlayback))` | In `handle_viewer_message()` when processing this message |
| `SeekVideo` | `Message::Viewer(ViewerMessage::VideoControls(VM::SeekCommit))` | In `handle_viewer_message()` when processing seek commit |

**Key insight:** The viewer component currently processes these internally. The relocation requires:
1. Viewer to emit an `Effect` or return a result to App
2. App handler to emit the diagnostic event
3. App handler to then delegate back to viewer if needed

### Diagnostics API Pattern

Use the existing pattern from `update.rs`:

```rust
// For user actions:
ctx.diagnostics.log_action(UserAction::TogglePlayback);

// For state events with fields:
ctx.diagnostics.log_state(AppStateEvent::MediaLoaded {
    media_type,
    file_size_bytes,
    dimensions,
    extension: metadata.extension,
    storage_type: metadata.storage_type,
    path_hash: metadata.path_hash,
});
```

**Existing examples in update.rs:**
- Line 947: `ctx.diagnostics.log_action(UserAction::OpenSettings)`
- Line 360: `ctx.diagnostics.log_state(AppStateEvent::EditorOpened { tool: None })`
- Line 782: `ctx.diagnostics.log_state(AppStateEvent::EditorDeblurStarted)`

### Architectural Principle

**Rule R1:** Collect at handler level, not UI components

UI components should only:
- Render views
- Emit messages/effects to App

UI components should NOT:
- Import diagnostics modules
- Call `log_action()` or `log_state()`
- Have knowledge of the diagnostics system

The App layer (handlers in `update.rs`) is the correct place for:
- Processing messages
- Coordinating state changes
- Emitting diagnostic events

### Message Flow Pattern (Corrected)

```
UI Component                    App Handler                     Diagnostics
     │                              │                               │
     │  Effect::StartLoadMedia      │                               │
     ├─────────────────────────────>│                               │
     │                              │  log_state(MediaLoading...)   │
     │                              ├──────────────────────────────>│
     │                              │                               │
     │                              │  [spawn async load task]      │
     │                              │                               │
     │  [async result received]     │                               │
     │                              │  log_state(MediaLoaded/Failed)│
     │                              ├──────────────────────────────>│
     │                              │                               │
```

### Important Implementation Notes

1. **TogglePlayback/SeekVideo** are currently handled entirely within the viewer component. To relocate:
   - Option A: Viewer emits `Effect::TogglePlayback` → App logs + calls viewer method
   - Option B: App intercepts `VideoControls` message before passing to viewer

2. **MediaLoaded/MediaFailed** come from async task results. The viewer currently receives and logs them. To relocate:
   - App must receive the async result first
   - App logs the event
   - App passes result to viewer for state update

3. **Preserve event data:** The current events include metadata (extension, storage_type, path_hash). Ensure this data is available at the handler level.

---

## Testing

### Testing Standards

Per `docs/architecture/coding-standards.md`:
- Unit tests in same file using `#[cfg(test)]` module
- Use Clippy pedantic (`cargo clippy --all --all-targets -- -D warnings`)
- All tests must pass before commit

### Test Location

Add tests to `src/app/update.rs` in the existing `#[cfg(test)]` module (or create one if absent).

### Test Examples

```rust
// In src/app/update.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_loading_started_emitted_from_handler() {
        // Setup: Initialize app with test diagnostics collector
        // Action: Trigger media load through handler
        // Assert: MediaLoadingStarted event captured
    }

    #[test]
    fn media_loaded_emitted_from_handler() {
        // Setup: Initialize app with test diagnostics collector
        // Action: Simulate successful media load completion
        // Assert: MediaLoaded event captured with correct metadata
    }

    #[test]
    fn toggle_playback_logged_from_handler() {
        // Setup: Initialize with video loaded
        // Action: Process TogglePlayback message through handler
        // Assert: TogglePlayback action captured
    }

    #[test]
    fn seek_video_logged_from_handler() {
        // Setup: Initialize with video loaded
        // Action: Process SeekCommit message through handler
        // Assert: SeekVideo action captured with position
    }
}
```

### Manual Verification

| Test | Steps | Expected Result |
|------|-------|-----------------|
| Events captured | 1. Load media<br>2. Export diagnostics | `MediaLoadingStarted`, `MediaLoaded` in report |
| Playback events | 1. Load video<br>2. Toggle play/pause<br>3. Seek<br>4. Export | `TogglePlayback`, `SeekVideo` in report |
| No regression | Compare report before/after | Same events, possibly different source location |

---

## Change Log

| Date | Version | Description | Author |
|------|---------|-------------|--------|
| 2025-01-15 | 1.0 | Story created from audit findings | Sarah (PO) |
| 2025-01-15 | 1.1 | PO Validation: Fixed Source Tree (tests in-file), added Target Handler Locations, added Diagnostics API Pattern, clarified Testing Standards | Sarah (PO) |
| 2026-01-15 | 1.2 | Implementation complete: All 5 collection points relocated, 5 unit tests added | James (Dev) |
| 2026-01-15 | 1.3 | QA fixes applied: Added 3 tests for TogglePlayback/SeekVideo, updated Dev Agent Record, marked all tasks complete | James (Dev) |

---

## Dev Agent Record

### Agent Model Used
Claude Opus 4.5 (claude-opus-4-5-20251101)

### Debug Log References
N/A - No debug issues encountered

### Completion Notes
- Relocated all 5 diagnostic collection points from UI layer to handler layer
- Created 4 helper functions: `log_media_loading_started()`, `log_media_loaded()`, `log_media_failed()`, `log_viewer_message_diagnostics()`
- Added `seek_preview_position()` getter to viewer component for SeekVideo logging
- Added 8 unit tests covering all relocated events
- QA review identified 2 missing tests (TogglePlayback, SeekVideo) - added 3 tests to address
- All 930 tests pass, clippy clean, fmt clean

### File List
- `src/app/update.rs` - Added 4 helper functions + 8 tests (~220 lines)
- `src/app/mod.rs` - Added 1 logging call in `handle_viewer_message()`
- `src/ui/viewer/component.rs` - Removed 5 logging blocks, added `seek_preview_position()` getter

---

## QA Results

### Review Date: 2026-01-15

### Reviewed By: Quinn (Test Architect)

### Code Quality Assessment

**Overall: GOOD** - Clean implementation following architectural principles.

The implementation correctly relocates all 5 diagnostic collection points from the UI component layer (`component.rs`) to the App handler layer (`update.rs`). The code demonstrates good separation of concerns with well-extracted helper functions:

- `log_media_loading_started()` - Handles MediaLoadingStarted events
- `log_media_loaded()` - Handles MediaLoaded events
- `log_media_failed()` - Handles MediaFailed events
- `log_viewer_message_diagnostics()` - Orchestrates logging for viewer messages including TogglePlayback and SeekVideo

The refactoring is clean and follows the R1 principle: "Collect at handler level, not UI components."

### Refactoring Performed

_No refactoring performed during review - implementation is clean._

### Compliance Check

- Coding Standards: ✓ In-file tests with `#[cfg(test)]`, Clippy pedantic passes
- Project Structure: ✓ Tests in same file, proper module organization
- Testing Strategy: ✓ 8 unit tests added covering all event types (QA fixes applied)
- All ACs Met: ✓ AC1-8 fully met

### Improvements Checklist

- [x] MediaLoadingStarted relocated to handler level
- [x] MediaLoaded relocated to handler level
- [x] MediaFailed relocated to handler level
- [x] TogglePlayback logging relocated to handler level
- [x] SeekVideo logging relocated to handler level
- [x] All 5 collection points removed from component.rs
- [x] No functional regression (930 tests pass)
- [x] Helper functions extracted for maintainability
- [x] Documentation with proper backticks per Clippy
- [x] Add test for TogglePlayback handler-level logging
- [x] Add test for SeekVideo handler-level logging
- [x] Update Dev Agent Record section (model used, completion notes, file list)

### Security Review

No security concerns. The implementation maintains proper path sanitization through `media_metadata()` which hashes file paths instead of storing them directly.

### Performance Considerations

No performance concerns. The logging is non-blocking (channel-based) and the helper functions are lightweight.

### Files Modified During Review

_No files modified during review._

### Implementation Files (for Dev Agent Record update)

Files modified by implementation:
- `src/app/update.rs` - Added 4 helper functions + 5 tests (~180 lines)
- `src/app/mod.rs` - Added 1 logging call
- `src/ui/viewer/component.rs` - Removed 5 logging blocks, added `seek_preview_position()` getter

### Gate Status

Gate: **CONCERNS** → docs/qa/gates/4.0-fix-collection-point-violations.yml

### Recommended Status

✓ **Ready for Final Review** - All QA concerns addressed

**QA fixes applied (2026-01-15):**
1. ✓ Added 3 tests for TogglePlayback and SeekVideo handler-level logging
2. ✓ Updated Dev Agent Record section with model, completion notes, and file list
3. ✓ All 930 tests pass, clippy and fmt clean

---

### Validation Review Date: 2026-01-15

### Reviewed By: Quinn (Test Architect)

### Validation Summary

**Gate: PASS** - All QA concerns from initial review have been addressed.

This is a validation review confirming that the QA fixes have been correctly applied.

### Verification Checklist

| Item | Status | Evidence |
|------|--------|----------|
| TEST-001: TogglePlayback test | ✓ FIXED | `toggle_playback_logged_from_handler()` at update.rs:2123 |
| TEST-001: SeekVideo test | ✓ FIXED | `seek_video_logged_from_handler()` at update.rs:2147 |
| TEST-001: Edge case test | ✓ ADDED | `seek_video_not_logged_without_preview_position()` at update.rs:2172 |
| DOC-001: Dev Agent Record | ✓ FIXED | Model, completion notes, and file list populated |
| No UI-level diagnostics | ✓ VERIFIED | grep confirms no `log_action`/`log_state` in component.rs |
| All tests pass | ✓ VERIFIED | 930 tests pass |
| Clippy clean | ✓ VERIFIED | No warnings |

### Test Quality Assessment

The added tests are well-structured:
- **toggle_playback_logged_from_handler**: Verifies TogglePlayback action is captured at handler level
- **seek_video_logged_from_handler**: Verifies SeekVideo action with position data is captured
- **seek_video_not_logged_without_preview_position**: Important edge case - no spurious events when seek position unavailable

### Final Gate Status

Gate: **PASS** → docs/qa/gates/4.0-fix-collection-point-violations.yml

### Recommended Status

✓ **Ready for Done** - All acceptance criteria met, all QA concerns resolved.

---
