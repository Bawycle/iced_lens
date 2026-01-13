# Story 1.10: AI Operation Timing Instrumentation

**Epic:** 1 - Diagnostics Core & Data Collection
**Status:** Done
**Priority:** Medium
**Estimate:** 1.5 hours
**Depends On:** Story 1.7

---

## Story

**As a** developer,
**I want** to instrument AI operations (deblur and upscale) with duration tracking,
**So that** the diagnostic system captures AI processing performance for optimization.

---

## Acceptance Criteria

### AI Deblur Instrumentation
1. `EditorDeblurStarted` state event when deblur begins
2. `AIDeblurProcess` operation logged on completion with:
   - `duration_ms` - processing time
   - `size_category` - image size category
   - `success` - whether operation succeeded
3. `EditorDeblurCompleted` or `EditorDeblurCancelled` state event

### AI Upscale Instrumentation
4. `AIUpscaleProcess` operation logged on completion with:
   - `duration_ms` - processing time
   - `scale_factor` - upscale multiplier (e.g., 2x, 4x)
   - `size_category` - image size category
   - `success` - whether operation succeeded

### Quality
5. Timing uses `Instant` for accurate measurements
6. Duration spans full async operation
7. No blocking of AI operations

---

## Tasks

### Task 1: Add Timing Fields to App State (AC: 5)
- [x] In `src/app/mod.rs`, add timing fields to `App` struct:
  - [x] `deblur_started_at: Option<Instant>`
  - [x] `upscale_started_at: Option<Instant>`
  - [x] `upscale_scale_factor: Option<f32>`
- [x] Initialize to `None` in `App::new()`

### Task 2: Instrument AI Deblur Start (AC: 1, 5)
- [x] In `src/app/update.rs`, `handle_deblur_request()`:
  - [x] Store `Instant::now()` in `ctx.deblur_started_at`
  - [x] Call `ctx.diagnostics.log_state(AppStateEvent::EditorDeblurStarted)`

### Task 3: Instrument AI Deblur Completion (AC: 2, 3, 6)
- [x] In `src/app/mod.rs`, `handle_deblur_apply_completed()`:
  - [x] Calculate `duration_ms` from `deblur_started_at`
  - [x] Calculate `size_category` from image dimensions
  - [x] On success: `log_operation(AIDeblurProcess { duration_ms, size_category, success: true })`
  - [x] On success: `log_state(EditorDeblurCompleted)`
  - [x] On error: `log_operation(AIDeblurProcess { ..., success: false })`

### Task 4: Instrument AI Upscale Start (AC: 5)
- [x] In `src/app/update.rs`, `handle_upscale_resize_request()`:
  - [x] Store `Instant::now()` in `ctx.upscale_started_at`
  - [x] Calculate and store `scale_factor` before async task

### Task 5: Instrument AI Upscale Completion (AC: 4, 6)
- [x] In `src/app/mod.rs`, `handle_upscale_resize_completed()`:
  - [x] Calculate `duration_ms` from `upscale_started_at`
  - [x] Calculate `size_category` from original image dimensions
  - [x] Retrieve stored `scale_factor`
  - [x] On success: `log_operation(AIUpscaleProcess { duration_ms, scale_factor, size_category, success: true })`
  - [x] On error: `log_operation(AIUpscaleProcess { ..., success: false })`

### Task 6: Run Validation (AC: 7)
- [x] `cargo fmt --all`
- [x] `cargo clippy --all --all-targets -- -D warnings`
- [x] `cargo test`

### Task 7: Commit Changes
- [x] Stage all changes
- [x] Commit with message: `feat(diagnostics): instrument AI operations [Story 1.10]`

---

## Dev Notes

### Source Tree

```
src/app/
├── mod.rs              # App struct, handle_deblur_apply_completed, handle_upscale_resize_completed
├── update.rs           # handle_editor_event, handle_deblur_request, handle_upscale_resize_request
└── message.rs          # DeblurApplyCompleted, UpscaleResizeCompleted messages

src/ui/image_editor/
├── state/
│   ├── deblur.rs       # DeblurState (UI state only, not timing)
│   └── resize.rs       # ResizeState with is_upscale_processing flag
└── messages.rs         # ImageEditorEvent::ApplyDeblur, UpscaleResizeRequested

src/diagnostics/
├── events.rs           # EditorDeblurStarted, AIDeblurProcess, AIUpscaleProcess
└── collector.rs        # DiagnosticsHandle
```

### Handler Locations

| Event | File | Function | Line |
|-------|------|----------|------|
| Deblur Request | `update.rs` | `handle_editor_event` → `ApplyDeblur` | ~740 |
| Deblur Async | `update.rs` | `handle_deblur_request` | 760 |
| Deblur Completed | `mod.rs` | `handle_deblur_apply_completed` | 759 |
| Upscale Request | `update.rs` | `handle_editor_event` → `UpscaleResizeRequested` | ~745 |
| Upscale Async | `update.rs` | `handle_upscale_resize_request` | 789 |
| Upscale Completed | `mod.rs` | `handle_upscale_resize_completed` | 791 |

### Key Structures

**`src/diagnostics/events.rs`:**
```rust
pub enum AppStateEvent {
    EditorDeblurStarted,
    EditorDeblurCompleted,
    EditorDeblurCancelled,
    // ...
}

pub enum AppOperation {
    AIDeblurProcess {
        duration_ms: u64,
        size_category: SizeCategory,
        success: bool,
    },
    AIUpscaleProcess {
        duration_ms: u64,
        scale_factor: f32,
        size_category: SizeCategory,
        success: bool,
    },
    // ...
}
```

### Required Imports

```rust
// In src/app/mod.rs
use std::time::Instant;
use crate::diagnostics::{AppStateEvent, AppOperation, SizeCategory};
```

### Async Timing Strategy

AI operations are async - timing must span the async boundary via App state:

```rust
// In App struct (src/app/mod.rs)
pub struct App {
    // ... existing fields ...

    /// Timestamp when deblur operation started (for duration tracking)
    deblur_started_at: Option<Instant>,
    /// Timestamp when upscale operation started (for duration tracking)
    upscale_started_at: Option<Instant>,
    /// Scale factor for current upscale operation
    upscale_scale_factor: Option<f32>,
}
```

### Instrumentation Patterns

**Deblur Start (in `handle_editor_event` for `ApplyDeblur`):**
```rust
ImageEditorEvent::ApplyDeblur => {
    // Store start time for duration tracking
    self.deblur_started_at = Some(Instant::now());

    // Log state event
    self.diagnostics_handle.log_state(AppStateEvent::EditorDeblurStarted);

    // Continue with existing logic
    handle_deblur_request(ctx)
}
```

**Deblur Completion (in `handle_deblur_apply_completed`):**
```rust
fn handle_deblur_apply_completed(&mut self, result: Result<...>) -> Task<Message> {
    // Calculate duration
    let duration_ms = self.deblur_started_at
        .take()
        .map(|start| start.elapsed().as_millis() as u64)
        .unwrap_or(0);

    if let Some(editor) = self.image_editor.as_mut() {
        // Calculate size category from image dimensions
        let (w, h) = (editor.current_image.width, editor.current_image.height);
        let size_category = SizeCategory::from_bytes((w * h * 4) as u64);

        match result {
            Ok(deblurred_image) => {
                // Log operation with success
                self.diagnostics_handle.log_operation(AppOperation::AIDeblurProcess {
                    duration_ms,
                    size_category,
                    success: true,
                });
                self.diagnostics_handle.log_state(AppStateEvent::EditorDeblurCompleted);

                editor.apply_deblur_result(*deblurred_image);
                // ... notification ...
            }
            Err(e) => {
                // Log operation with failure
                self.diagnostics_handle.log_operation(AppOperation::AIDeblurProcess {
                    duration_ms,
                    size_category,
                    success: false,
                });

                editor.deblur_failed();
                // ... notification ...
            }
        }
    }
    Task::none()
}
```

**Upscale Start (in `handle_upscale_resize_request`):**
```rust
fn handle_upscale_resize_request(
    ctx: &mut UpdateContext<'_>,
    target_width: u32,
    target_height: u32,
) -> Task<Message> {
    // ... existing checks ...

    if use_ai_upscale {
        let working_image = editor_state.working_image().clone();

        // Store timing info in App (via ctx or self)
        // Calculate scale factor: ratio of output to input pixels
        let original_pixels = (working_image.width() * working_image.height()) as f32;
        let target_pixels = (target_width * target_height) as f32;
        let scale_factor = (target_pixels / original_pixels).sqrt();

        // Store for completion handler
        // Note: Need to pass through ctx or store in App
        ctx.app.upscale_started_at = Some(Instant::now());
        ctx.app.upscale_scale_factor = Some(scale_factor);

        // ... existing async task ...
    }
}
```

**Upscale Completion (in `handle_upscale_resize_completed`):**
```rust
fn handle_upscale_resize_completed(&mut self, result: Result<...>) -> Task<Message> {
    let duration_ms = self.upscale_started_at
        .take()
        .map(|start| start.elapsed().as_millis() as u64)
        .unwrap_or(0);

    let scale_factor = self.upscale_scale_factor.take().unwrap_or(1.0);

    if let Some(editor) = self.image_editor.as_mut() {
        let (w, h) = (editor.current_image.width, editor.current_image.height);
        let size_category = SizeCategory::from_bytes((w * h * 4) as u64);

        match result {
            Ok(upscaled_image) => {
                self.diagnostics_handle.log_operation(AppOperation::AIUpscaleProcess {
                    duration_ms,
                    scale_factor,
                    size_category,
                    success: true,
                });
                editor.apply_upscale_resize_result(*upscaled_image);
            }
            Err(e) => {
                self.diagnostics_handle.log_operation(AppOperation::AIUpscaleProcess {
                    duration_ms,
                    scale_factor,
                    size_category,
                    success: false,
                });
                editor.clear_upscale_processing();
            }
        }
    }
    Task::none()
}
```

### Scale Factor Calculation

The Real-ESRGAN model always upscales by 4x. If a different target size is requested, the result is downscaled with Lanczos. The `scale_factor` should reflect the user's requested scale:

```rust
// From original to target dimensions
let scale_factor = ((target_width * target_height) as f32
    / (original_width * original_height) as f32).sqrt();
```

### Size Category from Image Dimensions

```rust
// RGBA = 4 bytes per pixel
let size_bytes = (image.width() * image.height() * 4) as u64;
let size_category = SizeCategory::from_bytes(size_bytes);
```

---

## Testing

### Unit Tests

| Test | File | Verification |
|------|------|--------------|
| `log_operation_ai_deblur` | `collector.rs` | AIDeblurProcess captured with duration |
| `log_operation_ai_upscale` | `collector.rs` | AIUpscaleProcess captured with scale_factor |
| `log_state_deblur_started` | `collector.rs` | EditorDeblurStarted captured |
| `log_state_deblur_completed` | `collector.rs` | EditorDeblurCompleted captured |

### Test Pattern

```rust
#[test]
fn ai_deblur_operation_tracked() {
    let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
    let handle = collector.handle();

    // Simulate deblur start
    handle.log_state(AppStateEvent::EditorDeblurStarted);

    // Simulate deblur completion
    handle.log_operation(AppOperation::AIDeblurProcess {
        duration_ms: 1500,
        size_category: SizeCategory::Medium,
        success: true,
    });
    handle.log_state(AppStateEvent::EditorDeblurCompleted);

    collector.process_pending();
    assert_eq!(collector.len(), 3); // start, operation, completed
}

#[test]
fn ai_upscale_operation_tracked() {
    let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
    let handle = collector.handle();

    handle.log_operation(AppOperation::AIUpscaleProcess {
        duration_ms: 2500,
        scale_factor: 2.0,
        size_category: SizeCategory::Large,
        success: true,
    });

    collector.process_pending();
    assert_eq!(collector.len(), 1);
}
```

### Performance Verification

```rust
#[test]
fn ai_operation_duration_is_positive() {
    // Verify duration_ms > 0 for real operations
    // This catches timing bugs where start time is not captured
}
```

---

## Dev Agent Record

### File List
| File | Action | Description |
|------|--------|-------------|
| `src/app/mod.rs` | Modified | Added timing fields, instrumented deblur/upscale completion handlers |
| `src/app/update.rs` | Modified | Added timing fields to UpdateContext, instrumented deblur/upscale start |

### Change Log
| Date | Change | Author |
|------|--------|--------|
| 2026-01-13 | Story created from Story 1.8 split | Claude Opus 4.5 |
| 2026-01-13 | PO Validation: Added comprehensive Dev Notes, Testing section, Task-AC mappings, async timing strategy | PO Validation |
| 2026-01-13 | Implementation complete: all ACs implemented, tests passing | Dev (Claude Opus 4.5) |

---
