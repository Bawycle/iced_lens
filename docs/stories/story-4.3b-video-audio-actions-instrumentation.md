# Story 4.3b: Video/Audio Actions Instrumentation

**Epic:** 4 - Diagnostics Collection Completeness
**Status:** Ready
**Priority:** P1
**Estimate:** 3-4 hours
**Depends On:** Story 4.0

---

## Story

**As a** developer analyzing playback behavior,
**I want** all video and audio control actions logged,
**So that** I can understand playback interaction patterns.

---

## Acceptance Criteria

1. `SetVolume` action logged with volume level
2. `ToggleMute` action logged with resulting mute state
3. `DeleteMedia` action logged
4. `CaptureFrame` action logged with capture timestamp
5. `ExportFile` action logged with export format
6. `StepForward` action logged
7. `StepBackward` action logged
8. `SetPlaybackSpeed` action logged with speed value
9. `ToggleLoop` action logged with resulting loop state
10. `TogglePlayback` and `SeekVideo` already at handler level (from Story 4.0)
11. All collection points at handler level
12. Integration tests verify each action is captured

---

## Tasks

- [ ] **Task 1:** Verify existing UserAction variants (AC: 1-9)
  - [ ] Check `events.rs` for existing video/audio action variants
  - [ ] Add any missing variants with appropriate fields

- [ ] **Task 2:** Add logging for `SetVolume` (AC: 1, 11)
  - [ ] Locate volume change handler in `update.rs`
  - [ ] Add `log_action(UserAction::SetVolume { level })`
  - [ ] Level should be 0.0-1.5 (Volume newtype range)

- [ ] **Task 3:** Add logging for `ToggleMute` (AC: 2, 11)
  - [ ] Locate mute toggle handler
  - [ ] Add `log_action(UserAction::ToggleMute { is_muted })`
  - [ ] Capture resulting state, not previous

- [ ] **Task 4:** Add logging for `DeleteMedia` (AC: 3, 11)
  - [ ] Locate delete handler
  - [ ] Add `log_action(UserAction::DeleteMedia)`
  - [ ] Consider: include media type (image/video)?

- [ ] **Task 5:** Add logging for `CaptureFrame` (AC: 4, 11)
  - [ ] Locate frame capture handler
  - [ ] Add `log_action(UserAction::CaptureFrame { timestamp_ms })`
  - [ ] Timestamp relative to video position

- [ ] **Task 6:** Add logging for `ExportFile` (AC: 5, 11)
  - [ ] Locate export handler
  - [ ] Add `log_action(UserAction::ExportFile { format })`
  - [ ] Format: "png", "jpg", etc.

- [ ] **Task 7:** Add logging for `StepForward` (AC: 6, 11)
  - [ ] Locate step forward handler
  - [ ] Add `log_action(UserAction::StepForward)`

- [ ] **Task 8:** Add logging for `StepBackward` (AC: 7, 11)
  - [ ] Locate step backward handler
  - [ ] Add `log_action(UserAction::StepBackward)`

- [ ] **Task 9:** Add logging for `SetPlaybackSpeed` (AC: 8, 11)
  - [ ] Locate speed change handler
  - [ ] Add `log_action(UserAction::SetPlaybackSpeed { speed })`
  - [ ] Speed range: 0.1-8.0 (PlaybackSpeed newtype)

- [ ] **Task 10:** Add logging for `ToggleLoop` (AC: 9, 11)
  - [ ] Locate loop toggle handler
  - [ ] Add `log_action(UserAction::ToggleLoop { is_looping })`

- [ ] **Task 11:** Verify Story 4.0 relocations complete (AC: 10)
  - [ ] Confirm `TogglePlayback` logging at handler level
  - [ ] Confirm `SeekVideo` logging at handler level

- [ ] **Task 12:** Add integration tests (AC: 12)
  - [ ] Test each action is captured when performed
  - [ ] Verify context data is accurate

- [ ] **Task 13:** Run validation
  - [ ] `cargo fmt --all`
  - [ ] `cargo clippy --all --all-targets -- -D warnings`
  - [ ] `cargo test`

---

## Dev Notes

### Source Tree

```
src/
├── diagnostics/
│   └── events.rs           # MODIFY: Add fields to some UserAction variants
├── app/
│   └── update.rs           # MODIFY: Add logging calls in video/audio handlers
├── video_player/
│   ├── state.rs            # REFERENCE: Player state
│   ├── volume.rs           # REFERENCE: Volume newtype (0.0-1.5)
│   └── playback_speed.rs   # REFERENCE: PlaybackSpeed newtype (0.1-8.0)
```

**Note:** Tests are in-file using `#[cfg(test)]` modules per coding standards.

### Current UserAction Variants (from events.rs)

**Already struct variants (just add logging):**
- `SetVolume { volume: f32 }` (line 287) - **Note: field is `volume`, not `level`**
- `SeekVideo { position_secs: f64 }` (line 263) - **Note: seconds, not milliseconds**
- `SetPlaybackSpeed { speed: f64 }` (line 275)

**Unit variants that need MODIFICATION to add context:**
- `ToggleMute` (line 293) → add `{ is_muted: bool }`
- `CaptureFrame` (line 347) → add `{ timestamp_secs: f64 }` (to match SeekVideo convention)
- `ExportFile` (line 350) → add `{ format: String }`
- `ToggleLoop` (line 281) → add `{ is_looping: bool }`

**Unit variants (keep as-is):**
- `DeleteMedia` (line 254)
- `StepForward` (line 269)
- `StepBackward` (line 272)
- `TogglePlayback` (line 260)

**Already logged but need relocation (Story 4.0):**
- `TogglePlayback` (currently in UI layer at component.rs:1147)
- `SeekVideo` (currently in UI layer at component.rs:1197)

### Existing and Proposed Event Structure

**Existing variants (correct field names - just add logging):**
```rust
SetVolume { volume: f32 },           // Field is `volume`, range 0.0-1.5
SeekVideo { position_secs: f64 },    // Field is `position_secs` (seconds!)
SetPlaybackSpeed { speed: f64 },     // Range 0.1-8.0
```

**Variants to MODIFY (add fields):**
```rust
// Current → Proposed
ToggleMute         → ToggleMute { is_muted: bool }
CaptureFrame       → CaptureFrame { timestamp_secs: f64 }  // Match SeekVideo convention
ExportFile         → ExportFile { format: String }
ToggleLoop         → ToggleLoop { is_looping: bool }
```

**Unit variants (keep as-is):**
```rust
TogglePlayback,
DeleteMedia,
StepForward,
StepBackward,
```

### Handler Patterns

```rust
// In handle_video_message or similar:
VideoMessage::SetVolume(vol) => {
    ctx.diagnostics.log_action(UserAction::SetVolume {
        volume: vol.value(),  // Note: field is `volume`, not `level`
    });
    self.video_player.set_volume(vol);
    // ...
}

VideoMessage::ToggleLoop => {
    self.video_player.toggle_loop();
    ctx.diagnostics.log_action(UserAction::ToggleLoop {
        is_looping: self.video_player.is_looping(),
    });
    // ...
}
```

---

## Testing

### Integration Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn set_volume_action_logged() {
        // Setup: Load video
        // Action: Change volume
        // Assert: SetVolume action captured with level
    }

    #[test]
    fn capture_frame_action_logged() {
        // Setup: Load video, seek to position
        // Action: Capture frame
        // Assert: CaptureFrame action captured with timestamp
    }

    #[test]
    fn playback_speed_action_logged() {
        // Setup: Load video
        // Action: Change playback speed
        // Assert: SetPlaybackSpeed action captured
    }
}
```

---

## Change Log

| Date | Version | Description | Author |
|------|---------|-------------|--------|
| 2025-01-15 | 1.0 | Story created from audit findings | Sarah (PO) |
| 2025-01-15 | 1.1 | PO Validation: Fixed Source Tree (tests in-file), corrected field names (`volume` not `level`, `position_secs` not `position_ms`), clarified which variants need modification vs just logging | Sarah (PO) |

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
