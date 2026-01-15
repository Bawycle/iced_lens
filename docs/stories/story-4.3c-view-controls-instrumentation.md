# Story 4.3c: View Controls Instrumentation

**Epic:** 4 - Diagnostics Collection Completeness
**Status:** Ready
**Priority:** P1
**Estimate:** 2-3 hours
**Depends On:** Story 4.0

---

## Story

**As a** developer analyzing view preferences,
**I want** all view control actions logged,
**So that** I can understand how users interact with view controls.

---

## Acceptance Criteria

1. `ZoomIn` action logged with resulting zoom level
2. `ZoomOut` action logged with resulting zoom level
3. `ResetZoom` action logged
4. `ToggleFitToWindow` action logged with resulting state
5. `RotateClockwise` action logged with resulting angle
6. `RotateCounterClockwise` action logged with resulting angle
7. `ToggleFullscreen` action logged
8. `ExitFullscreen` action logged
9. All collection points at handler level
10. Integration tests verify each action is captured

---

## Tasks

- [ ] **Task 1:** Verify existing UserAction variants (AC: 1-8)
  - [ ] Check `events.rs` for existing view control action variants
  - [ ] Add any missing variants with appropriate fields

- [ ] **Task 2:** Add logging for `ZoomIn` (AC: 1, 9)
  - [ ] Locate zoom in handler in `update.rs`
  - [ ] Add `log_action(UserAction::ZoomIn { resulting_zoom_percent })`
  - [ ] Zoom range: 10-800% (ZoomPercent newtype)

- [ ] **Task 3:** Add logging for `ZoomOut` (AC: 2, 9)
  - [ ] Locate zoom out handler
  - [ ] Add `log_action(UserAction::ZoomOut { resulting_zoom_percent })`

- [ ] **Task 4:** Add logging for `ResetZoom` (AC: 3, 9)
  - [ ] Locate reset zoom handler
  - [ ] Add `log_action(UserAction::ResetZoom)`

- [ ] **Task 5:** Add logging for `ToggleFitToWindow` (AC: 4, 9)
  - [ ] Locate fit toggle handler
  - [ ] Add `log_action(UserAction::ToggleFitToWindow { is_fit })`

- [ ] **Task 6:** Add logging for `RotateClockwise` (AC: 5, 9)
  - [ ] Locate rotate CW handler
  - [ ] Add `log_action(UserAction::RotateClockwise { resulting_angle })`
  - [ ] Angle range: 0-359 (RotationAngle newtype)

- [ ] **Task 7:** Add logging for `RotateCounterClockwise` (AC: 6, 9)
  - [ ] Locate rotate CCW handler
  - [ ] Add `log_action(UserAction::RotateCounterClockwise { resulting_angle })`

- [ ] **Task 8:** Add logging for `ToggleFullscreen` (AC: 7, 9)
  - [ ] Locate fullscreen toggle handler
  - [ ] Add `log_action(UserAction::ToggleFullscreen)`

- [ ] **Task 9:** Add logging for `ExitFullscreen` (AC: 8, 9)
  - [ ] Locate exit fullscreen handler
  - [ ] Add `log_action(UserAction::ExitFullscreen)`
  - [ ] Note: May be same handler as ToggleFullscreen

- [ ] **Task 10:** Add integration tests (AC: 10)
  - [ ] Test each action is captured when performed
  - [ ] Verify context data is accurate

- [ ] **Task 11:** Run validation
  - [ ] `cargo fmt --all`
  - [ ] `cargo clippy --all --all-targets -- -D warnings`
  - [ ] `cargo test`

---

## Dev Notes

### Source Tree

```
src/
├── diagnostics/
│   └── events.rs           # MODIFY: Add fields to view control UserAction variants (lines 299-320)
├── app/
│   └── update.rs           # MODIFY: Add logging calls in view control handlers
├── ui/
│   └── state/
│       ├── zoom.rs         # REFERENCE: ZoomPercent, ZoomStep newtypes
│       └── rotation.rs     # REFERENCE: RotationAngle newtype
```

**Note:** Tests are in-file using `#[cfg(test)]` modules per coding standards.

### Current UserAction Variants (from events.rs:299-320)

**IMPORTANT:** All these variants currently exist as UNIT variants (no fields). This story requires MODIFYING some of them to struct variants.

**Unit variants to MODIFY (add context fields):**
- `ZoomIn` (line 299) → add `{ resulting_zoom_percent: u16 }`
- `ZoomOut` (line 302) → add `{ resulting_zoom_percent: u16 }`
- `ToggleFitToWindow` (line 308) → add `{ is_fit: bool }`
- `RotateClockwise` (line 317) → add `{ resulting_angle: u16 }`
- `RotateCounterClockwise` (line 320) → add `{ resulting_angle: u16 }`

**Unit variants to keep as-is:**
- `ResetZoom` (line 305)
- `ToggleFullscreen` (line 311)
- `ExitFullscreen` (line 314)

### Newtype References

```rust
// From src/ui/state/zoom.rs
pub struct ZoomPercent(u16);  // Range: 10-800

// From src/ui/state/rotation.rs
pub struct RotationAngle(u16);  // Range: 0-359
```

### Proposed Event Context

```rust
pub enum UserAction {
    ZoomIn {
        resulting_zoom_percent: u16,
    },
    ZoomOut {
        resulting_zoom_percent: u16,
    },
    ResetZoom,
    ToggleFitToWindow {
        is_fit: bool,  // Resulting state
    },
    RotateClockwise {
        resulting_angle: u16,  // 0-359
    },
    RotateCounterClockwise {
        resulting_angle: u16,
    },
    ToggleFullscreen,
    ExitFullscreen,
    // ... other variants
}
```

### Handler Patterns

```rust
// In handle_viewer_message or similar:
ViewerMessage::ZoomIn => {
    self.zoom = self.zoom.step_up(self.zoom_step);
    log_action(UserAction::ZoomIn {
        resulting_zoom_percent: self.zoom.value(),
    });
    // ...
}

ViewerMessage::RotateClockwise => {
    self.rotation = self.rotation.rotate_clockwise();
    log_action(UserAction::RotateClockwise {
        resulting_angle: self.rotation.degrees(),
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
    fn zoom_in_action_logged() {
        // Setup: Load image
        // Action: Zoom in
        // Assert: ZoomIn action captured with zoom level
    }

    #[test]
    fn rotate_clockwise_action_logged() {
        // Setup: Load image
        // Action: Rotate clockwise
        // Assert: RotateClockwise action captured with angle
    }

    #[test]
    fn fullscreen_actions_logged() {
        // Setup: Normal view
        // Action: Toggle fullscreen, then exit
        // Assert: ToggleFullscreen and ExitFullscreen captured
    }
}
```

---

## Change Log

| Date | Version | Description | Author |
|------|---------|-------------|--------|
| 2025-01-15 | 1.0 | Story created from audit findings | Sarah (PO) |
| 2025-01-15 | 1.1 | PO Validation: Fixed Source Tree (tests in-file), clarified which unit variants need MODIFICATION to struct variants, added exact line numbers | Sarah (PO) |

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
