# Story 1.7: Diagnostic Event Instrumentation

**Epic:** 1 - Diagnostics Core & Data Collection
**Status:** Ready
**Priority:** High
**Estimate:** 4-5 hours
**Depends On:** Story 1.3, Story 1.4, Story 1.5

---

## Story

**As a** developer,
**I want** to instrument application handlers to emit diagnostic events (user actions, state changes, operations),
**So that** the diagnostic system captures real application behavior.

---

## Acceptance Criteria

### User Action Instrumentation
1. At least 5 key user actions instrumented as proof of concept:
   - Navigation (NavigateNext, NavigatePrevious)
   - Media loading (LoadMedia with source context)
   - Playback controls (TogglePlayback, SeekVideo)
   - Screen navigation (OpenSettings, EnterEditor)
2. `DiagnosticsHandle` passed to relevant components

### State Transition Instrumentation
3. At least 3 key state transitions instrumented:
   - Video playback state changes (play/pause/seek/buffering/error)
   - Media loading lifecycle (started/loaded/failed)
   - Editor session (opened/closed)

### Operation Instrumentation
4. At least 3 operations instrumented with duration tracking:
   - AI deblur processing
   - AI upscale processing
   - Video seek operation

### Warning/Error Capture Integration
5. Integration with notification system to capture user-visible warnings/errors
6. Warnings and errors from notification toasts logged as diagnostic events

### Integration Quality
7. Instrumentation integrated at appropriate handler locations
8. All instrumentation is non-blocking (uses existing channel mechanism)
9. Duration tracking uses `Instant` for accurate measurements
10. Integration tests verify events are captured during real operations
11. No performance regression from instrumentation (< 1ms overhead per event)

---

## Tasks

### Task 1: Initialize DiagnosticsHandle in App
- [ ] Create `DiagnosticsCollector` instance in app initialization
- [ ] Store `DiagnosticsHandle` in app state or context
- [ ] Ensure handle is accessible from message handlers

### Task 2: Instrument User Actions in update.rs
- [ ] In `src/app/update.rs`
- [ ] Add `log_action()` calls for:
  - [ ] `NavigateNext` in `handle_navigate_next`
  - [ ] `NavigatePrevious` in `handle_navigate_previous`
  - [ ] `LoadMedia` in `handle_open_file_dialog_result` (source: file_dialog)
  - [ ] `LoadMedia` in `handle_file_dropped` (source: drag_drop)
  - [ ] `TogglePlayback` in playback toggle handler
  - [ ] `SeekVideo` in seek handler
  - [ ] `OpenSettings` in `handle_navbar_message`
  - [ ] `EnterEditor` in editor entry handler

### Task 3: Instrument Video Playback States
- [ ] In `src/video_player/` (state.rs or relevant handler)
- [ ] Add `log_state()` calls for:
  - [ ] `VideoPlaying` when playback starts
  - [ ] `VideoPaused` when playback pauses
  - [ ] `VideoSeeking` when seek initiated
  - [ ] `VideoBuffering` when buffering detected
  - [ ] `VideoError` on playback errors
  - [ ] `VideoAtEndOfStream` when video ends
- [ ] Pass `DiagnosticsHandle` to video player component

### Task 4: Instrument Media Loading Lifecycle
- [ ] In `src/app/update.rs` or media loading handler
- [ ] Add `log_state()` calls for:
  - [ ] `MediaLoadingStarted` with media_type and size_category
  - [ ] `MediaLoaded` on successful load
  - [ ] `MediaFailed` on load failure
- [ ] Calculate `SizeCategory` from file metadata

### Task 5: Instrument Editor Session
- [ ] In `src/ui/image_editor/` handlers
- [ ] Add `log_state()` calls for:
  - [ ] `EditorOpened` when editor screen activated
  - [ ] `EditorClosed` with `had_unsaved_changes` flag
- [ ] Track unsaved changes state for closure event

### Task 6: Instrument AI Deblur Operation
- [ ] In `src/ui/image_editor/state/deblur.rs` or handler
- [ ] Capture start time with `Instant::now()`
- [ ] Add `log_state()` for `EditorDeblurStarted`
- [ ] On completion, calculate duration and call `log_operation()` with:
  - [ ] `AIDeblurProcess { duration_ms, size_category, success }`
- [ ] Add `log_state()` for `EditorDeblurCompleted` or `EditorDeblurCancelled`

### Task 7: Instrument AI Upscale Operation
- [ ] In resize handler where AI upscale is triggered
- [ ] Capture start time with `Instant::now()`
- [ ] On completion, calculate duration and call `log_operation()` with:
  - [ ] `AIUpscaleProcess { duration_ms, scale_factor, size_category, success }`

### Task 8: Instrument Video Seek Operation
- [ ] In video player seek handler
- [ ] Capture start time and initial position
- [ ] On seek completion, calculate duration and call `log_operation()` with:
  - [ ] `VideoSeek { duration_ms, seek_distance_secs }`

### Task 9: Instrument Warning/Error Capture
- [ ] In `src/ui/notifications/manager.rs`
- [ ] Modify `Manager::push()` to accept optional `DiagnosticsHandle`
- [ ] When `Notification` with `Severity::Warning` is pushed:
  - [ ] Call `log_warning()` with `WarningEvent`
  - [ ] Map `notification.message_key()` to `WarningType`
- [ ] When `Notification` with `Severity::Error` is pushed:
  - [ ] Call `log_error()` with `ErrorEvent`
  - [ ] Map `notification.message_key()` to `ErrorType`
- [ ] Update all call sites of `Manager::push()` to pass diagnostics handle
- [ ] Alternative: Store `DiagnosticsHandle` in `Manager` struct at construction

### Task 10: Write Integration Tests
- [ ] Test user action events are captured (navigate, load, playback)
- [ ] Test video state events are captured during playback simulation
- [ ] Test media loading events are captured
- [ ] Test editor session events are captured
- [ ] Test operation duration is reasonable (> 0ms, < timeout)
- [ ] Test warning/error events are captured from notifications

### Task 11: Run Validation
- [ ] `cargo fmt --all`
- [ ] `cargo clippy --all --all-targets -- -D warnings`
- [ ] `cargo test`

### Task 12: Commit Changes
- [ ] Stage all changes
- [ ] Commit with message: `feat(diagnostics): instrument diagnostic events [Story 1.7]`

---

## Dev Notes

### Integration Points from Story 1.3 Analysis

The dev agent identified these integration points during Story 1.3:

| Handler | User Actions to Log |
|---------|---------------------|
| `handle_viewer_message` | NavigateNext/Previous, OpenSettings, EnterEditor, ToggleFullscreen |
| `handle_navigate_next/previous` | Navigation actions |
| `handle_delete_current_media` | DeleteMedia |
| `handle_settings_message` | Settings changes |
| `handle_editor_message` | Editor actions (Save, Deblur, Upscale) |
| `handle_navbar_message` | OpenSettings, OpenHelp, OpenAbout, EnterEditor |
| `handle_open_file_dialog_result` | LoadMedia (file_dialog) |
| `handle_file_dropped` | LoadMedia (drag_drop) |
| `handle_capture_frame` | CaptureFrame |

### State/Operation Integration Points

| Component | File | Events to Emit |
|-----------|------|----------------|
| Video Player | `src/video_player/state.rs` | VideoPlaying, VideoPaused, VideoSeeking, VideoSeek operation |
| Media Loading | `src/app/update.rs` | MediaLoadingStarted, MediaLoaded, MediaFailed |
| Editor Session | `src/ui/image_editor/component.rs` or `messages.rs` | EditorOpened, EditorClosed |
| Deblur | `src/ui/image_editor/state/deblur.rs` | DeblurStarted/Completed, AIDeblurProcess |
| Upscale | `src/ui/image_editor/state/resize.rs` | AIUpscaleProcess |
| Notifications | `src/ui/notifications/manager.rs` | Warning, Error events |

### Warning/Error Capture Pattern

The cleanest approach is to intercept in `Manager::push()`:

```rust
impl Manager {
    pub fn push(&mut self, notification: Notification) {
        // Log to diagnostics based on severity
        if let Some(handle) = &self.diagnostics {
            match notification.severity() {
                Severity::Warning => {
                    let event = WarningEvent::from_notification(&notification);
                    handle.log_warning(event);
                }
                Severity::Error => {
                    let event = ErrorEvent::from_notification(&notification);
                    handle.log_error(event);
                }
                _ => {}
            }
        }
        // ... existing push logic
    }
}
```

This requires adding `diagnostics: Option<DiagnosticsHandle>` to `Manager` and initializing it when the App creates the notification manager.

### Duration Tracking Pattern

```rust
let start = Instant::now();
// ... perform operation ...
let duration_ms = start.elapsed().as_millis() as u64;
diagnostics.log_operation(AppOperation::AIDeblurProcess {
    duration_ms,
    size_category: SizeCategory::from_bytes(file_size),
    success: result.is_ok(),
});
```

### DiagnosticsHandle Access Pattern

The `DiagnosticsHandle` should be passed through the component hierarchy or stored in app state. Recommended approach:

```rust
// In App struct
pub struct App {
    diagnostics: DiagnosticsHandle,
    // ...
}

// In handlers
fn handle_navigate_next(&mut self) {
    self.diagnostics.log_action(UserAction::NavigateNext);
    // ... rest of handler
}
```

---

## Testing

### Integration Tests
| Test | Verification |
|------|--------------|
| User action capture | log_action events in buffer after simulated actions |
| Video playback simulation | State events captured in buffer |
| Media load trigger | Loading lifecycle events present |
| Editor open/close cycle | Session events with correct flags |
| AI operation mock | Duration > 0, success flag correct |

### Performance Tests
- Verify `log_action()` completes in < 1ms (channel send is non-blocking)
- Verify `log_state()` completes in < 1ms
- Verify no UI stutter during instrumented operations

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
