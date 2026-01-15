# Story 4.0: Fix Collection Point Architectural Violations

**Epic:** 4 - Diagnostics Collection Completeness
**Status:** Ready
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

- [ ] **Task 1:** Analyze current collection points (AC: 1-5)
  - [ ] Document exact event emission context in `component.rs`
  - [ ] Identify corresponding handler locations in `update.rs`
  - [ ] Map message flow from UI to App handler

- [ ] **Task 2:** Relocate `MediaLoadingStarted` (AC: 1, 6)
  - [ ] Identify handler that processes media loading initiation
  - [ ] Add `log_state_event(AppStateEvent::MediaLoadingStarted {...})` in handler
  - [ ] Remove emission from `component.rs:652`

- [ ] **Task 3:** Relocate `MediaLoaded` (AC: 2, 6)
  - [ ] Identify handler that receives media load completion
  - [ ] Add `log_state_event(AppStateEvent::MediaLoaded {...})` in handler
  - [ ] Remove emission from `component.rs:866`

- [ ] **Task 4:** Relocate `MediaFailed` (AC: 3, 6)
  - [ ] Identify handler that receives media load failure
  - [ ] Add `log_state_event(AppStateEvent::MediaFailed {...})` in handler
  - [ ] Remove emission from `component.rs:946`

- [ ] **Task 5:** Relocate `TogglePlayback` logging (AC: 4, 6)
  - [ ] Identify handler for playback toggle messages
  - [ ] Add `log_action(UserAction::TogglePlayback)` in handler
  - [ ] Remove emission from `component.rs:1147`

- [ ] **Task 6:** Relocate `SeekVideo` logging (AC: 5, 6)
  - [ ] Identify handler for seek messages
  - [ ] Add `log_action(UserAction::SeekVideo {...})` in handler
  - [ ] Remove emission from `component.rs:1197`

- [ ] **Task 7:** Verify no diagnostics imports remain in UI (AC: 6)
  - [ ] Check `component.rs` has no `use crate::diagnostics::*` related to these events
  - [ ] Ensure UI component is fully decoupled from diagnostics

- [ ] **Task 8:** Add/update tests (AC: 7, 8)
  - [ ] Add tests in `update.rs` `#[cfg(test)]` module (per coding standards)
  - [ ] Test `MediaLoadingStarted` captured from handler
  - [ ] Test `MediaLoaded` captured from handler
  - [ ] Test `MediaFailed` captured from handler
  - [ ] Test `TogglePlayback` captured from handler
  - [ ] Test `SeekVideo` captured from handler

- [ ] **Task 9:** Run validation
  - [ ] `cargo fmt --all`
  - [ ] `cargo clippy --all --all-targets -- -D warnings`
  - [ ] `cargo test`
  - [ ] Manual verification: events still appear in diagnostic reports

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
