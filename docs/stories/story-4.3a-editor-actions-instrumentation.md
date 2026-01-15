# Story 4.3a: Editor Actions Instrumentation

**Epic:** 4 - Diagnostics Collection Completeness
**Status:** Ready
**Priority:** P1
**Estimate:** 2-3 hours
**Depends On:** Story 4.0

---

## Story

**As a** developer analyzing feature usage,
**I want** all editor actions logged,
**So that** I can understand which editing features are most used.

---

## Acceptance Criteria

1. `ApplyCrop` action logged with crop dimensions
2. `ApplyResize` action logged with resize parameters
3. `ApplyDeblur` action logged
4. `ApplyUpscale` action logged with scale factor
5. `SaveImage` action logged with save format
6. `Undo` action logged with operation type undone
7. `Redo` action logged with operation type redone
8. `ReturnToViewer` action logged
9. All collection points at handler level in `update.rs`
10. Integration tests verify each action is captured

---

## Tasks

- [ ] **Task 1:** Verify existing UserAction variants (AC: 1-8)
  - [ ] Check `events.rs` for existing editor action variants
  - [ ] Add any missing variants with appropriate fields

- [ ] **Task 2:** Add logging for `ApplyCrop` (AC: 1, 9)
  - [ ] Locate crop application handler in `update.rs`
  - [ ] Add `log_action(UserAction::ApplyCrop { width, height, x, y })`
  - [ ] Ensure dimensions captured before crop applied

- [ ] **Task 3:** Add logging for `ApplyResize` (AC: 2, 9)
  - [ ] Locate resize application handler
  - [ ] Add `log_action(UserAction::ApplyResize { scale, new_width, new_height })`

- [ ] **Task 4:** Add logging for `ApplyDeblur` (AC: 3, 9)
  - [ ] Locate deblur application handler
  - [ ] Add `log_action(UserAction::ApplyDeblur)`
  - [ ] Note: State event `EditorDeblurStarted` may already exist

- [ ] **Task 5:** Add logging for `ApplyUpscale` (AC: 4, 9)
  - [ ] Locate upscale application handler
  - [ ] Add `log_action(UserAction::ApplyUpscale { scale_factor })`

- [ ] **Task 6:** Add logging for `SaveImage` (AC: 5, 9)
  - [ ] Locate save handler
  - [ ] Add `log_action(UserAction::SaveImage { format })`
  - [ ] Format could be "png", "jpg", "original", etc.

- [ ] **Task 7:** Add logging for `Undo` (AC: 6, 9)
  - [ ] Locate undo handler
  - [ ] Add `log_action(UserAction::Undo { operation_type })`
  - [ ] Operation type: what was undone (crop, resize, etc.)

- [ ] **Task 8:** Add logging for `Redo` (AC: 7, 9)
  - [ ] Locate redo handler
  - [ ] Add `log_action(UserAction::Redo { operation_type })`

- [ ] **Task 9:** Add logging for `ReturnToViewer` (AC: 8, 9)
  - [ ] Locate return handler
  - [ ] Add `log_action(UserAction::ReturnToViewer)`
  - [ ] May want to include: `had_unsaved_changes: bool`

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
│   └── events.rs           # MODIFY: Add fields to existing UserAction variants (lines 356-374)
├── app/
│   └── update.rs           # MODIFY: Add logging calls in editor handlers (line 683+)
├── ui/
│   └── image_editor/
│       └── state.rs        # REFERENCE: Editor state and operations
```

**Note:** Tests are in-file using `#[cfg(test)]` modules per coding standards.

### Target Handler Locations

| Action | Handler | Line | Notes |
|--------|---------|------|-------|
| ExitEditor | `handle_editor_message` | 693 | Currently logs `EditorClosed` (state event) |
| SaveRequested | `handle_editor_message` | 727 | NO user action logging |
| DeblurRequested | `handle_deblur_request` | 772 | Logs `EditorDeblurStarted` (state event, not UserAction) |
| UpscaleResize | `handle_upscale_resize_request` | 757 | Need to locate |

**Note:** Many edit actions (crop, resize, undo, redo) occur within `ImageEditorState::update()` in `src/ui/image_editor/state.rs`. Dev agent should trace message flow to find exact handlers.

### Current UserAction Variants (from events.rs:356-374)

**IMPORTANT:** These variants currently exist as UNIT variants (no fields). This story requires MODIFYING them to struct variants with context data.

Current structure (to be modified):
```rust
ApplyCrop,           // → ApplyCrop { width, height, x, y }
ApplyResize,         // → ApplyResize { scale_percent, new_width, new_height }
ApplyDeblur,         // → ApplyDeblur (keep as unit)
ApplyUpscale,        // → ApplyUpscale { scale_factor }
SaveImage,           // → SaveImage { format }
Undo,                // → Undo { operation_type } or keep as unit
Redo,                // → Redo { operation_type } or keep as unit
ReturnToViewer,      // → ReturnToViewer { had_unsaved_changes }
```

**Note on UserAction vs AppStateEvent:**
- `UserAction::ApplyDeblur` = User clicked deblur button (intent)
- `AppStateEvent::EditorDeblurStarted` = Deblur operation started (state)
- Both should be logged for complete instrumentation

### Proposed Event Context

```rust
pub enum UserAction {
    ApplyCrop {
        width: u32,
        height: u32,
        x: u32,
        y: u32,
    },
    ApplyResize {
        scale_percent: f32,
        new_width: u32,
        new_height: u32,
    },
    ApplyDeblur,
    ApplyUpscale {
        scale_factor: u32,  // e.g., 4 for 4x
    },
    SaveImage {
        format: String,  // "png", "jpg", "webp"
    },
    Undo {
        operation_type: String,
    },
    Redo {
        operation_type: String,
    },
    ReturnToViewer {
        had_unsaved_changes: bool,
    },
    // ... existing variants
}
```

### Handler Patterns

Editor actions are typically handled in patterns like:

```rust
// In handle_editor_message or similar:
EditorMessage::ApplyCrop(rect) => {
    log_action(UserAction::ApplyCrop {
        width: rect.width,
        height: rect.height,
        x: rect.x,
        y: rect.y,
    });
    self.editor.apply_crop(rect);
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
    fn apply_crop_action_logged() {
        // Setup: Load image into editor
        // Action: Apply crop
        // Assert: ApplyCrop action captured with dimensions
    }

    #[test]
    fn save_image_action_logged() {
        // Setup: Edit image
        // Action: Save
        // Assert: SaveImage action captured with format
    }

    #[test]
    fn undo_redo_actions_logged() {
        // Setup: Apply crop, then undo, then redo
        // Assert: Undo and Redo actions captured
    }
}
```

---

## Change Log

| Date | Version | Description | Author |
|------|---------|-------------|--------|
| 2025-01-15 | 1.0 | Story created from audit findings | Sarah (PO) |
| 2025-01-15 | 1.1 | PO Validation: Fixed Source Tree (tests in-file), added Target Handler Locations, clarified that variants need to be MODIFIED from unit to struct, noted UserAction vs AppStateEvent distinction | Sarah (PO) |

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
