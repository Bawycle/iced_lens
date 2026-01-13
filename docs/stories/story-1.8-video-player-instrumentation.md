# Story 1.8: Video Player Instrumentation

**Epic:** 1 - Diagnostics Core & Data Collection
**Status:** Done
**Priority:** Medium
**Estimate:** 2 hours
**Depends On:** Story 1.7

---

## Story

**As a** developer,
**I want** to instrument video playback states and seek operations,
**So that** the diagnostic system captures video player behavior for debugging playback issues.

---

## Acceptance Criteria

### DiagnosticsHandle Integration
1. `DiagnosticsHandle` passed to video player component
2. Handle stored in video player state and accessible from event handlers

### User Action Events (deferred from Story 1.7)
3. User actions instrumented:
   - `TogglePlayback` when play/pause is toggled
   - `SeekVideo` when user seeks to a position

### Video Playback State Events
4. State events emitted for:
   - `VideoPlaying` when playback starts
   - `VideoPaused` when playback pauses
   - `VideoSeeking` when seek initiated
   - `VideoBuffering` when buffering detected
   - `VideoError` on playback errors
   - `VideoAtEndOfStream` when video ends

### Video Seek Operation
5. `VideoSeek` operation logged with:
   - `duration_ms` - time taken to complete seek
   - `seek_distance_secs` - distance seeked in seconds

### Quality
6. All instrumentation is non-blocking
7. No performance regression (< 1ms overhead per event)

---

## Tasks

### Task 1: Pass DiagnosticsHandle to Video Player (AC: 1, 2)
- [x] Add `diagnostics: Option<DiagnosticsHandle>` field to `VideoPlayer` struct in `state.rs`
- [x] Add `pub fn set_diagnostics(&mut self, handle: DiagnosticsHandle)` method
- [x] Call `set_diagnostics()` when VideoPlayer is created in `component.rs`
- [x] Verify handle is accessible in `play()`, `pause()`, `seek()` methods

### Task 2: Instrument User Actions (AC: 3)
- [x] In `src/ui/viewer/component.rs`, `match video_msg` block (~line 984):
  - [x] Add `log_action(UserAction::TogglePlayback)` in `VM::TogglePlayback` handler
  - [x] Add `log_action(UserAction::SeekVideo { position_secs })` in `VM::SeekCommit` handler

### Task 3: Instrument Video Playback States (AC: 4)
- [x] In `src/video_player/state.rs`:
  - [x] Add `log_state(VideoPlaying)` at end of `play()` method (line 299)
  - [x] Add `log_state(VideoPaused)` at end of `pause()` method (line 352)
  - [x] Add `log_state(VideoSeeking)` at start of `seek()` method (line 415)
  - [x] Add `log_state(VideoBuffering)` in `set_buffering()` method (line 531)
  - [x] Add `log_state(VideoError)` in `set_error()` method (line 536)
  - [x] Add `log_state(VideoAtEndOfStream)` in `set_at_end_of_stream()` method (line 277)

### Task 4: Instrument Video Seek Operation (AC: 5)
- [x] Reuse existing `seeking_started_at: Option<Instant>` field (line 176)
- [x] Store initial position when seek starts
- [x] On seek completion (in `complete_seek` or equivalent):
  - [x] Calculate `duration_ms = seeking_started_at.elapsed().as_millis()`
  - [x] Calculate `seek_distance_secs = (final_pos - initial_pos).abs()`
  - [x] Call `log_operation(AppOperation::VideoSeek { duration_ms, seek_distance_secs })`

### Task 5: Add Tests (AC: 6, 7)
- [x] Add test `instrumentation_overhead_is_minimal` in `collector.rs`
- [x] Verify non-blocking behavior (channel-based logging)

### Task 6: Run Validation
- [x] `cargo fmt --all`
- [x] `cargo clippy --all --all-targets -- -D warnings`
- [x] `cargo test`

### Task 7: Commit Changes
- [x] Stage all changes
- [x] Commit with message: `feat(diagnostics): instrument video player [Story 1.8]`

---

## Dev Notes

### Source Tree (Video Player Module)

```
src/video_player/
├── state.rs            # VideoPlayer struct + PlaybackState enum (TARGET)
├── subscription.rs     # Iced subscription for decoder communication
├── decoder.rs          # FFmpeg frame decoding
├── sync.rs             # Audio/video synchronization
└── mod.rs              # Module exports

src/ui/viewer/
├── component.rs        # Viewer component - handles video messages (TARGET)
└── video_controls.rs   # UI controls - emits TogglePlayback, SeekCommit, etc.
```

### Key Structures

**`src/video_player/state.rs`:**
```rust
pub enum PlaybackState {
    Playing { position_secs: f64 },
    Paused { position_secs: f64 },
    Seeking { target_secs: f64, resume_playing: bool },
    Buffering { position_secs: f64 },
    Error { message: String },
    Stopped,
}

pub struct VideoPlayer {
    state: PlaybackState,
    // ... other fields
}

impl VideoPlayer {
    pub fn play(&mut self)           // Line 299
    pub fn pause(&mut self)          // Line 352
    pub fn seek(&mut self, target_secs: f64)  // Line 415
    pub fn state(&self) -> &PlaybackState
}
```

### Handler Locations

**User Actions (TogglePlayback, SeekVideo):**
- File: `src/ui/viewer/component.rs`
- Location: `match video_msg` block (~line 984)
- `VM::TogglePlayback` calls `player.play()` or `player.pause()`
- `VM::SeekCommit` calls `player.seek(target_secs)`

**State Transitions:**
- File: `src/video_player/state.rs`
- Methods: `play()`, `pause()`, `seek()`, `set_buffering()`, `set_error()`

### Required Imports

```rust
// In video_player/state.rs
use crate::diagnostics::{DiagnosticsHandle, AppStateEvent, AppOperation};

// In ui/viewer/component.rs
use crate::diagnostics::UserAction;
```

### Handle Passing Strategy

**Option B recommended (setter method - less invasive):**
```rust
impl VideoPlayer {
    pub fn set_diagnostics(&mut self, handle: DiagnosticsHandle) {
        self.diagnostics = Some(handle);
    }
}
```

Call from `App::new()` or when creating VideoPlayer in component.rs.

### Instrumentation Patterns

**User Action - TogglePlayback (in component.rs):**
```rust
VM::TogglePlayback => {
    ctx.diagnostics.log_action(UserAction::TogglePlayback);
    // ... existing logic
}
```

**User Action - SeekVideo (in component.rs):**
```rust
VM::SeekCommit => {
    if let Some(target_secs) = self.seek_preview_position {
        ctx.diagnostics.log_action(UserAction::SeekVideo { position_secs: target_secs });
        // ... existing logic
    }
}
```

**State Transition (in state.rs):**
```rust
pub fn play(&mut self) {
    // ... existing logic that sets state
    if let Some(ref handle) = self.diagnostics {
        handle.log_state(AppStateEvent::VideoPlaying {
            position_secs: self.state.position().unwrap_or(0.0),
        });
    }
}
```

**Operation with Timing (in state.rs):**
```rust
pub fn seek(&mut self, target_secs: f64) {
    let initial_position = self.state.position().unwrap_or(0.0);
    self.seeking_started_at = Some(Instant::now());  // Already exists!
    // ... existing seek logic
}

pub fn complete_seek(&mut self, position_secs: f64) {
    if let (Some(ref handle), Some(start)) = (&self.diagnostics, self.seeking_started_at.take()) {
        let duration_ms = start.elapsed().as_millis() as u64;
        handle.log_operation(AppOperation::VideoSeek {
            duration_ms,
            seek_distance_secs: (position_secs - initial_position).abs(),
        });
    }
    // ... existing logic
}
```

**Note:** `seeking_started_at` field already exists (line 176) - reuse it for timing!

---

## Testing

### Existing Test Coverage

`src/video_player/state.rs` contains 40+ unit tests covering:
- State transitions (play, pause, seek, stop)
- Position tracking
- Seek behavior
- A/V sync

### New Tests Required

| Test | File | Verification |
|------|------|--------------|
| `log_action_toggle_playback` | `collector.rs` | UserAction::TogglePlayback captured |
| `log_action_seek_video` | `collector.rs` | UserAction::SeekVideo with position |
| `log_state_video_playing` | `collector.rs` | AppStateEvent::VideoPlaying captured |
| `log_state_video_paused` | `collector.rs` | AppStateEvent::VideoPaused captured |
| `log_operation_video_seek` | `collector.rs` | AppOperation::VideoSeek with duration |

### Test Pattern

```rust
#[test]
fn video_player_logs_play_state() {
    let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
    let handle = collector.handle();

    // Simulate play action
    handle.log_state(AppStateEvent::VideoPlaying { position_secs: 0.0 });
    collector.process_pending();

    assert_eq!(collector.len(), 1);
    // Verify event content
}
```

### Performance Verification

Run benchmark to ensure < 1ms overhead:
```rust
#[test]
fn instrumentation_overhead_is_minimal() {
    let start = Instant::now();
    for _ in 0..1000 {
        handle.log_action(UserAction::TogglePlayback);
    }
    let elapsed = start.elapsed();
    assert!(elapsed.as_micros() < 1000, "Should complete 1000 logs in < 1ms");
}
```

---

## Dev Agent Record

### File List
| File | Action | Description |
|------|--------|-------------|
| `src/video_player/state.rs` | Modified | Added DiagnosticsHandle, instrumented play/pause/seek/buffering/error/end-of-stream |
| `src/ui/viewer/component.rs` | Modified | Pass DiagnosticsHandle to VideoPlayer, instrument TogglePlayback/SeekVideo |
| `src/app/update.rs` | Modified | Pass diagnostics to handle_message, extract handle_successful_media_load helper |
| `src/app/mod.rs` | Modified | Add test_diagnostics() helper for tests |
| `src/diagnostics/collector.rs` | Modified | Add instrumentation_overhead_is_minimal test |

### Change Log
| Date | Change | Author |
|------|--------|--------|
| 2026-01-13 | Story created from Story 1.8 split | Claude Opus 4.5 |
| 2026-01-13 | Added TogglePlayback/SeekVideo user actions (deferred from 1.7) | PO Review |
| 2026-01-13 | Added comprehensive Dev Notes (source tree, handler locations, imports, patterns), Testing section, Task-AC mappings | PO Validation |
| 2026-01-13 | Implementation complete: all AC implemented, tests passing | Dev (James) |

---
