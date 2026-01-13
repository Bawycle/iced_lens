# Story 1.11: Instrumentation Integration Tests

**Epic:** 1 - Diagnostics Core & Data Collection
**Status:** Ready
**Priority:** Medium
**Estimate:** 1 hour
**Depends On:** Story 1.7, Story 1.8, Story 1.9, Story 1.10

---

## Story

**As a** developer,
**I want** integration tests that verify diagnostic events are captured during real operations,
**So that** I can be confident the instrumentation works correctly in practice.

---

## Acceptance Criteria

### Test Coverage
1. Tests verify user action events are captured
2. Tests verify state transition events are captured
3. Tests verify operation events with duration are captured
4. Tests verify warning/error events from notifications are captured

### Test Quality
5. Tests use real (or realistic mock) operations, not just direct API calls
6. Tests verify event content, not just presence
7. Tests verify timing data is reasonable (> 0ms, < timeout)

---

## Tasks

### Task 1: User Action Event Tests (AC: 1, 5, 6)
- [ ] In `src/diagnostics/collector.rs`, add tests after existing tests (~line 800):
  - [ ] `test_navigate_actions_have_correct_structure`
  - [ ] `test_load_media_captures_source`
  - [ ] `test_editor_actions_captured`
- [ ] Verify event content matches expected structure

### Task 2: State Transition Event Tests (AC: 2, 5, 6)
- [ ] Add tests for state events:
  - [ ] `test_editor_opened_closed_lifecycle`
  - [ ] `test_video_state_events_captured` (VideoPlaying, VideoPaused, etc.)
  - [ ] `test_media_loading_lifecycle_events` (Started, Loaded, Failed)
- [ ] Verify state event fields are correct

### Task 3: Operation Event Tests (AC: 3, 5, 6, 7)
- [ ] Add tests for operation events:
  - [ ] `test_ai_deblur_operation_has_valid_duration`
  - [ ] `test_ai_upscale_operation_has_scale_factor`
  - [ ] `test_video_seek_operation_has_distance`
- [ ] Verify duration_ms > 0 for all operations
- [ ] Verify duration_ms < 300_000 (5 min timeout)

### Task 4: Warning/Error Event Tests (AC: 4, 6)
- [ ] Add tests for warning/error events:
  - [ ] `test_warning_event_has_correct_type`
  - [ ] `test_error_event_has_correct_type`
  - [ ] `test_warning_error_messages_sanitized`
- [ ] Verify type mapping matches expected categories

### Task 5: Run Validation
- [ ] `cargo fmt --all`
- [ ] `cargo clippy --all --all-targets -- -D warnings`
- [ ] `cargo test`

### Task 6: Commit Changes
- [ ] Stage all changes
- [ ] Commit with message: `test(diagnostics): add instrumentation integration tests [Story 1.11]`

---

## Dev Notes

### Test File Location

All tests go in `src/diagnostics/collector.rs` in the existing `#[cfg(test)] mod tests` section (~line 309).

There are already **121 tests** in the diagnostics module. Add new tests after the existing ones.

### Existing Test Patterns (Reference)

The codebase uses this pattern consistently:

```rust
#[test]
fn handle_log_action_sends_to_collector() {
    let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
    let handle = collector.handle();

    handle.log_action(UserAction::TogglePlayback);

    // Event is in channel, not yet in buffer
    assert!(collector.is_empty());

    // Process pending events
    collector.process_pending();

    assert_eq!(collector.len(), 1);
}
```

### Correct API Usage

**IMPORTANT:** Use `collector.iter()` and match on `event.kind`:

```rust
#[test]
fn test_navigate_actions_have_correct_structure() {
    let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
    let handle = collector.handle();

    handle.log_action(UserAction::NavigateNext);
    handle.log_action(UserAction::NavigatePrevious);

    collector.process_pending();

    assert_eq!(collector.len(), 2);

    // Iterate and verify structure
    let events: Vec<_> = collector.iter().collect();

    match &events[0].kind {
        DiagnosticEventKind::UserAction { action, details } => {
            assert!(matches!(action, UserAction::NavigateNext));
            assert!(details.is_none());
        }
        _ => panic!("expected UserAction event"),
    }
}
```

### User Action Tests

```rust
#[test]
fn test_load_media_captures_source() {
    let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
    let handle = collector.handle();

    handle.log_action(UserAction::LoadMedia {
        source: Some("file_dialog".to_string()),
    });

    collector.process_pending();

    let event = collector.iter().next().unwrap();
    match &event.kind {
        DiagnosticEventKind::UserAction { action, .. } => {
            match action {
                UserAction::LoadMedia { source } => {
                    assert_eq!(source.as_deref(), Some("file_dialog"));
                }
                _ => panic!("expected LoadMedia action"),
            }
        }
        _ => panic!("expected UserAction event"),
    }
}
```

### State Transition Tests

```rust
#[test]
fn test_editor_opened_closed_lifecycle() {
    let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
    let handle = collector.handle();

    // Simulate editor lifecycle
    handle.log_state(AppStateEvent::EditorOpened { tool: None });
    handle.log_state(AppStateEvent::EditorClosed {
        had_unsaved_changes: false,
    });

    collector.process_pending();

    assert_eq!(collector.len(), 2);

    let events: Vec<_> = collector.iter().collect();

    // Verify order and content
    assert!(matches!(
        &events[0].kind,
        DiagnosticEventKind::AppState {
            state: AppStateEvent::EditorOpened { .. }
        }
    ));
    assert!(matches!(
        &events[1].kind,
        DiagnosticEventKind::AppState {
            state: AppStateEvent::EditorClosed { .. }
        }
    ));
}

#[test]
fn test_video_state_events_captured() {
    let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
    let handle = collector.handle();

    handle.log_state(AppStateEvent::VideoPlaying { position_secs: 0.0 });
    handle.log_state(AppStateEvent::VideoPaused { position_secs: 10.5 });
    handle.log_state(AppStateEvent::VideoSeeking { target_secs: 30.0 });

    collector.process_pending();

    assert_eq!(collector.len(), 3);
}

#[test]
fn test_media_loading_lifecycle_events() {
    let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
    let handle = collector.handle();

    handle.log_state(AppStateEvent::MediaLoadingStarted {
        media_type: MediaType::Image,
        size_category: SizeCategory::Medium,
    });
    handle.log_state(AppStateEvent::MediaLoaded {
        media_type: MediaType::Image,
        size_category: SizeCategory::Medium,
    });

    collector.process_pending();

    assert_eq!(collector.len(), 2);
}
```

### Operation Tests with Duration Validation

```rust
#[test]
fn test_ai_deblur_operation_has_valid_duration() {
    let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
    let handle = collector.handle();

    handle.log_operation(AppOperation::AIDeblurProcess {
        duration_ms: 1500,
        size_category: SizeCategory::Medium,
        success: true,
    });

    collector.process_pending();

    let event = collector.iter().next().unwrap();
    match &event.kind {
        DiagnosticEventKind::Operation { operation } => {
            match operation {
                AppOperation::AIDeblurProcess { duration_ms, .. } => {
                    assert!(*duration_ms > 0, "Duration should be positive");
                    assert!(*duration_ms < 300_000, "Duration should be < 5 minutes");
                }
                _ => panic!("expected AIDeblurProcess"),
            }
        }
        _ => panic!("expected Operation event"),
    }
}

#[test]
fn test_ai_upscale_operation_has_scale_factor() {
    let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
    let handle = collector.handle();

    handle.log_operation(AppOperation::AIUpscaleProcess {
        duration_ms: 2500,
        scale_factor: 2.0,
        size_category: SizeCategory::Large,
        success: true,
    });

    collector.process_pending();

    let event = collector.iter().next().unwrap();
    match &event.kind {
        DiagnosticEventKind::Operation { operation } => {
            match operation {
                AppOperation::AIUpscaleProcess { scale_factor, .. } => {
                    assert!(*scale_factor > 0.0);
                    assert!(*scale_factor <= 4.0); // Real-ESRGAN max
                }
                _ => panic!("expected AIUpscaleProcess"),
            }
        }
        _ => panic!("expected Operation event"),
    }
}

#[test]
fn test_video_seek_operation_has_distance() {
    let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
    let handle = collector.handle();

    handle.log_operation(AppOperation::VideoSeek {
        duration_ms: 150,
        seek_distance_secs: 10.5,
    });

    collector.process_pending();

    let event = collector.iter().next().unwrap();
    match &event.kind {
        DiagnosticEventKind::Operation { operation } => {
            match operation {
                AppOperation::VideoSeek {
                    duration_ms,
                    seek_distance_secs,
                } => {
                    assert!(*duration_ms > 0);
                    assert!(*seek_distance_secs >= 0.0);
                }
                _ => panic!("expected VideoSeek"),
            }
        }
        _ => panic!("expected Operation event"),
    }
}
```

### Warning/Error Tests

```rust
#[test]
fn test_warning_event_has_correct_type() {
    let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
    let handle = collector.handle();

    handle.log_warning(WarningEvent::new(
        WarningType::UnsupportedFormat,
        "Format not supported",
    ));

    collector.process_pending();

    let event = collector.iter().next().unwrap();
    match &event.kind {
        DiagnosticEventKind::Warning { event } => {
            assert_eq!(event.warning_type, WarningType::UnsupportedFormat);
            assert_eq!(event.message, "Format not supported");
        }
        _ => panic!("expected Warning event"),
    }
}

#[test]
fn test_error_event_has_correct_type() {
    let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
    let handle = collector.handle();

    handle.log_error(ErrorEvent::new(
        ErrorType::AIModelError,
        "Model inference failed",
    ));

    collector.process_pending();

    let event = collector.iter().next().unwrap();
    match &event.kind {
        DiagnosticEventKind::Error { event } => {
            assert_eq!(event.error_type, ErrorType::AIModelError);
            assert_eq!(event.message, "Model inference failed");
        }
        _ => panic!("expected Error event"),
    }
}
```

### Required Imports (already present in collector.rs tests)

```rust
use super::*;
use std::time::Duration;
// MediaType, SizeCategory imported via crate::diagnostics::*
```

### Note on "Integration" vs Unit Tests

These tests verify the **API contract** of the instrumentation system:
- Events are captured with correct structure
- Event content matches what was logged
- Duration values are within reasonable bounds

True integration tests (testing App-level handlers) would require mocking the full Iced application and are out of scope. The instrumentation correctness is verified by these API tests combined with the instrumentation code from Stories 1.7-1.10.

---

## Dev Agent Record

### Change Log
| Date | Change | Author |
|------|--------|--------|
| 2026-01-13 | Story created from Story 1.8 split | Claude Opus 4.5 |
| 2026-01-13 | PO Validation: Fixed API usage, added test file location, complete test examples, Task-AC mappings | PO Validation |

---
