# Story 4.4: Missing State Events Instrumentation

**Epic:** 4 - Diagnostics Collection Completeness
**Status:** Ready
**Priority:** P2
**Estimate:** 3-4 hours
**Depends On:** Stories 4.0, 4.1, 4.2, 4.3a-c

---

## Story

**As a** developer analyzing state transitions,
**I want** all state events logged,
**So that** I can track complete state machine behavior.

---

## Acceptance Criteria

1. `VideoLoopToggled` event emitted with `enabled: bool`
2. `VideoSpeedChanged` event emitted with `speed: f64`
3. `EditorDeblurProgress` event emitted with `percent: f32`
4. `EditorDeblurCancelled` event emitted when deblur cancelled
5. `ModelDownloadStarted` event emitted with `model: AIModel`
6. `ModelDownloadCompleted` event emitted with `model: AIModel`
7. `ModelDownloadFailed` event emitted with `model: AIModel`, `reason: String`
8. Video events emitted from `state.rs` (Domain layer - acceptable)
9. AI model events emitted from AI engine code at lifecycle points
10. Integration tests verify each state event is captured

---

## Tasks

- [ ] **Task 1:** Verify existing AppStateEvent variants (AC: 1-7)
  - [ ] Confirm all 7 variants exist in `events.rs` (lines 488-560) - ALREADY VERIFIED
  - [ ] Confirm fields match requirements - ALREADY VERIFIED
  - [ ] **Note:** No new variants needed - only logging calls

- [ ] **Task 2:** Add logging for `VideoLoopToggled` (AC: 1, 8)
  - [ ] Locate loop toggle in `video_player/state.rs`
  - [ ] Add `log_state_event(AppStateEvent::VideoLoopToggled { enabled })`
  - [ ] Emit when loop state changes

- [ ] **Task 3:** Add logging for `VideoSpeedChanged` (AC: 2, 8)
  - [ ] Locate speed change in `video_player/state.rs`
  - [ ] Add `log_state_event(AppStateEvent::VideoSpeedChanged { speed })`
  - [ ] Emit when playback speed changes

- [ ] **Task 4:** Add logging for `EditorDeblurProgress` (AC: 3)
  - [ ] Locate deblur progress updates in handler
  - [ ] Add `log_state_event(AppStateEvent::EditorDeblurProgress { percent })`
  - [ ] Consider throttling (e.g., every 10% or 5 seconds)

- [ ] **Task 5:** Add logging for `EditorDeblurCancelled` (AC: 4)
  - [ ] Locate deblur cancellation handler
  - [ ] Add `log_state_event(AppStateEvent::EditorDeblurCancelled)`

- [ ] **Task 6:** Add logging for `ModelDownloadStarted` (AC: 5, 9)
  - [ ] Locate AI model download initiation
  - [ ] Add `log_state_event(AppStateEvent::ModelDownloadStarted { model })`
  - [ ] Model could be "NAFNet", "RealESRGAN", etc.

- [ ] **Task 7:** Add logging for `ModelDownloadCompleted` (AC: 6, 9)
  - [ ] Locate AI model download completion
  - [ ] Add `log_state_event(AppStateEvent::ModelDownloadCompleted { model })`

- [ ] **Task 8:** Add logging for `ModelDownloadFailed` (AC: 7, 9)
  - [ ] Locate AI model download error handling
  - [ ] Add `log_state_event(AppStateEvent::ModelDownloadFailed { model, reason })`
  - [ ] Capture error reason string

- [ ] **Task 9:** Add integration tests (AC: 10)
  - [ ] Test video state events captured
  - [ ] Test editor state events captured
  - [ ] Test AI model lifecycle events captured (if testable)

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
│   └── events.rs           # REFERENCE: All state event variants ALREADY EXIST (lines 488-560)
├── video_player/
│   └── state.rs            # MODIFY: Add logging for VideoLoopToggled, VideoSpeedChanged
├── app/
│   └── update.rs           # MODIFY: Add logging for EditorDeblur* events
├── ai/
│   └── engine.rs           # MODIFY: Add logging for ModelDownload* events (or similar)
```

**Note:** Tests are in-file using `#[cfg(test)]` modules per coding standards.

### Current AppStateEvent Variants (from events.rs)

**IMPORTANT:** All state event variants ALREADY EXIST in events.rs. This story only adds LOGGING calls.

**Logged (14):**
- `MediaLoadingStarted`, `MediaLoaded`, `MediaFailed`
- `VideoPlaying`, `VideoPaused`, `VideoSeeking`, `VideoBuffering`, `VideoError`, `VideoAtEndOfStream`
- `EditorOpened`, `EditorClosed`, `EditorDeblurStarted`, `EditorDeblurCompleted`

**Variants that EXIST but are NOT logged (7):**
- `VideoLoopToggled` (line 488) - Exists with `{ enabled: bool }`
- `VideoSpeedChanged` (line 494) - Exists with `{ speed: f64 }`
- `EditorDeblurProgress` (line 519) - Exists with `{ percent: f32 }`
- `EditorDeblurCancelled` (line 528) - Exists as unit variant
- `ModelDownloadStarted` (line 543) - Exists with `{ model: AIModel }`
- `ModelDownloadCompleted` (line 549) - Exists with `{ model: AIModel }`
- `ModelDownloadFailed` (line 555) - Exists with `{ model: AIModel, reason: String }`

### Existing Event Structure (NO CHANGES NEEDED)

**These variants ALREADY EXIST in events.rs - just add logging calls:**

```rust
// Video state (emit from state.rs - Domain layer OK)
VideoLoopToggled { enabled: bool },          // line 488
VideoSpeedChanged { speed: f64 },            // line 494 - range 0.1-8.0

// Editor state (emit from update.rs handlers)
EditorDeblurProgress { percent: f32 },       // line 519 - range 0.0-100.0
EditorDeblurCancelled,                       // line 528

// AI Model lifecycle (emit from AI engine code)
// Note: model field is AIModel enum, not String
ModelDownloadStarted { model: AIModel },     // line 543
ModelDownloadCompleted { model: AIModel },   // line 549
ModelDownloadFailed { model: AIModel, reason: String },  // line 555
```

### Collection Point Locations

| Event | Location | Layer | Notes |
|-------|----------|-------|-------|
| `VideoLoopToggled` | `state.rs` | Domain | With other video state events |
| `VideoSpeedChanged` | `state.rs` | Domain | With other video state events |
| `EditorDeblurProgress` | `update.rs` | Handler | Throttle to avoid flood |
| `EditorDeblurCancelled` | `update.rs` | Handler | On cancellation |
| `ModelDownload*` | AI engine | Handler | During download lifecycle |

### Progress Throttling

For `EditorDeblurProgress`, avoid flooding the buffer:

```rust
// Only emit at significant milestones
if percent >= last_reported_percent + 10.0 {
    log_state_event(AppStateEvent::EditorDeblurProgress { percent });
    last_reported_percent = percent;
}
```

Or emit at fixed intervals (e.g., every 25%: 0, 25, 50, 75, 100).

---

## Testing

### Integration Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn video_loop_toggled_event_emitted() {
        // Setup: Load video
        // Action: Toggle loop
        // Assert: VideoLoopToggled event captured
    }

    #[test]
    fn video_speed_changed_event_emitted() {
        // Setup: Load video
        // Action: Change speed
        // Assert: VideoSpeedChanged event captured with speed value
    }

    #[test]
    fn editor_deblur_cancelled_event_emitted() {
        // Setup: Start deblur operation
        // Action: Cancel deblur
        // Assert: EditorDeblurCancelled event captured
    }

    // Note: AI model download tests may require mocking
}
```

---

## Change Log

| Date | Version | Description | Author |
|------|---------|-------------|--------|
| 2025-01-15 | 1.0 | Story created from audit findings | Sarah (PO) |
| 2025-01-15 | 1.1 | PO Validation: Fixed Source Tree (tests in-file), confirmed all 7 variants ALREADY EXIST with correct fields, clarified this story only adds logging (no event creation needed) | Sarah (PO) |

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
