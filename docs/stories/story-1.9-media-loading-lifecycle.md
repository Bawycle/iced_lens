# Story 1.9: Media Loading Lifecycle Instrumentation

**Epic:** 1 - Diagnostics Core & Data Collection
**Status:** Ready
**Priority:** Medium
**Estimate:** 1 hour
**Depends On:** Story 1.7

---

## Story

**As a** developer,
**I want** to instrument the media loading lifecycle,
**So that** the diagnostic system captures loading performance and failure patterns.

---

## Acceptance Criteria

### Media Loading Events
1. `MediaLoadingStarted` emitted when media load begins, with:
   - `media_type: MediaType` (Image, Video, or Unknown)
   - `size_category: SizeCategory` (Small, Medium, Large, VeryLarge)
2. `MediaLoaded` emitted on successful load, with:
   - `media_type: MediaType`
   - `size_category: SizeCategory`
3. `MediaFailed` emitted on load failure, with:
   - `media_type: MediaType`
   - `reason: String` (sanitized error message, no file paths)

### Size Category Calculation
4. `SizeCategory` calculated from file metadata using `SizeCategory::from_bytes()`
5. Thresholds: Small (<1MB), Medium (1-10MB), Large (10-100MB), VeryLarge (>100MB)

### Quality
6. Events emitted at appropriate points in loading flow
7. No blocking of media loading operations (async flow preserved)

---

## Tasks

### Task 1: Pass DiagnosticsHandle to Viewer Component (AC: 1, 2, 3)
- [ ] Verify `DiagnosticsHandle` is accessible in `ViewerState` (via `UpdateContext`)
- [ ] If not present, add `diagnostics: Option<DiagnosticsHandle>` field to `ViewerState`
- [ ] Pass handle when creating/updating viewer state

### Task 2: Instrument Loading Started (AC: 1, 4, 5)
- [ ] In `src/ui/viewer/component.rs`, find where `is_loading_media = true` is set (~line 586)
- [ ] Before setting loading state, get file size from path metadata
- [ ] Determine `MediaType` from file extension
- [ ] Call `log_state(AppStateEvent::MediaLoadingStarted { media_type, size_category })`

### Task 3: Instrument Loading Success (AC: 2)
- [ ] In `src/ui/viewer/component.rs`, `Message::MediaLoaded(Ok(...))` handler (~line 738)
- [ ] Determine `MediaType` from loaded `MediaData` variant
- [ ] Call `log_state(AppStateEvent::MediaLoaded { media_type, size_category })`
- [ ] Note: `size_category` should be stored from Task 2 or recalculated

### Task 4: Instrument Loading Failure (AC: 3)
- [ ] In `src/ui/viewer/component.rs`, `Message::MediaLoaded(Err(...))` handler (~line 799)
- [ ] Sanitize error message (remove any file paths)
- [ ] Call `log_state(AppStateEvent::MediaFailed { media_type, reason })`

### Task 5: Run Validation (AC: 6, 7)
- [ ] `cargo fmt --all`
- [ ] `cargo clippy --all --all-targets -- -D warnings`
- [ ] `cargo test`

### Task 6: Commit Changes
- [ ] Stage all changes
- [ ] Commit with message: `feat(diagnostics): instrument media loading lifecycle [Story 1.9]`

---

## Dev Notes

### Source Tree

```
src/ui/viewer/
├── component.rs        # ViewerState + Message handlers (TARGET)
├── controls.rs         # UI controls
└── mod.rs              # Module exports

src/diagnostics/
├── events.rs           # MediaType, SizeCategory, AppStateEvent definitions
├── collector.rs        # DiagnosticsHandle
└── mod.rs              # Public exports
```

### Key Structures

**`src/diagnostics/events.rs`:**
```rust
pub enum MediaType {
    Image,
    Video,
    Unknown,
}

pub enum SizeCategory {
    Small,      // < 1 MB
    Medium,     // 1-10 MB
    Large,      // 10-100 MB
    VeryLarge,  // > 100 MB
}

impl SizeCategory {
    pub fn from_bytes(bytes: u64) -> Self { ... }
}

pub enum AppStateEvent {
    MediaLoadingStarted {
        media_type: MediaType,
        size_category: SizeCategory,
    },
    MediaLoaded {
        media_type: MediaType,
        size_category: SizeCategory,
    },
    MediaFailed {
        media_type: MediaType,
        reason: String,  // NOTE: 'reason' not 'error_type'
    },
    // ...
}
```

**`src/ui/viewer/component.rs`:**
```rust
pub struct ViewerState {
    pub is_loading_media: bool,
    pub loading_started_at: Option<Instant>,  // Already exists (line 206)
    pub current_media_path: Option<PathBuf>,
    // ...
}
```

### Handler Locations

| Event | File | Location | Trigger |
|-------|------|----------|---------|
| Loading Started | `component.rs` | ~line 586 | `is_loading_media = true` |
| Loading Success | `component.rs` | ~line 738 | `Message::MediaLoaded(Ok(...))` |
| Loading Failure | `component.rs` | ~line 799 | `Message::MediaLoaded(Err(...))` |

### Required Imports

```rust
// In src/ui/viewer/component.rs
use crate::diagnostics::{AppStateEvent, DiagnosticsHandle, MediaType, SizeCategory};
```

### Instrumentation Patterns

**Loading Started (in `start_loading` or equivalent):**
```rust
// Get file metadata for size category
let size_category = self.current_media_path
    .as_ref()
    .and_then(|p| std::fs::metadata(p).ok())
    .map(|m| SizeCategory::from_bytes(m.len()))
    .unwrap_or(SizeCategory::Small);

// Determine media type from extension
let media_type = self.current_media_path
    .as_ref()
    .and_then(|p| p.extension())
    .map(|ext| {
        let ext = ext.to_string_lossy().to_lowercase();
        if ["mp4", "webm", "avi", "mkv", "mov"].contains(&ext.as_str()) {
            MediaType::Video
        } else {
            MediaType::Image
        }
    })
    .unwrap_or(MediaType::Unknown);

// Store for later use in success/failure handlers
self.loading_media_type = Some(media_type);
self.loading_size_category = Some(size_category);

if let Some(ref handle) = self.diagnostics {
    handle.log_state(AppStateEvent::MediaLoadingStarted {
        media_type,
        size_category,
    });
}
```

**Loading Success (in `MediaLoaded(Ok(...))` handler):**
```rust
// Use stored values or derive from MediaData
let media_type = match &media {
    MediaData::Image(_) => MediaType::Image,
    MediaData::Video(_) => MediaType::Video,
};

if let Some(ref handle) = self.diagnostics {
    handle.log_state(AppStateEvent::MediaLoaded {
        media_type,
        size_category: self.loading_size_category.take().unwrap_or(SizeCategory::Small),
    });
}
```

**Loading Failure (in `MediaLoaded(Err(...))` handler):**
```rust
// Sanitize error - remove any file paths
let reason = match &error {
    Error::Svg(e) => format!("SVG error: {}", e),
    Error::Video(e) => format!("Video error: {}", e),
    Error::Io(e) => format!("IO error: {}", e.kind()),
    Error::Config(e) => format!("Config error: {}", e),
};

if let Some(ref handle) = self.diagnostics {
    handle.log_state(AppStateEvent::MediaFailed {
        media_type: self.loading_media_type.take().unwrap_or(MediaType::Unknown),
        reason,
    });
}
```

### Design Decisions

1. **Size category stored at load start**: File metadata is available before async load begins
2. **Media type from extension initially**: Confirmed from `MediaData` variant on success
3. **Sanitized error messages**: Never include file paths in `reason` field
4. **New fields needed**: `loading_media_type: Option<MediaType>`, `loading_size_category: Option<SizeCategory>`

---

## Testing

### Unit Tests

| Test | File | Verification |
|------|------|--------------|
| `log_state_media_loading_started` | `collector.rs` | Event captured with correct fields |
| `log_state_media_loaded` | `collector.rs` | Event captured on success |
| `log_state_media_failed` | `collector.rs` | Event captured with sanitized reason |
| `size_category_from_bytes` | `events.rs` | Already exists (lines 847-878) |

### Test Pattern

```rust
#[test]
fn media_loading_lifecycle_events() {
    let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
    let handle = collector.handle();

    // Simulate loading started
    handle.log_state(AppStateEvent::MediaLoadingStarted {
        media_type: MediaType::Image,
        size_category: SizeCategory::Medium,
    });

    // Simulate loading completed
    handle.log_state(AppStateEvent::MediaLoaded {
        media_type: MediaType::Image,
        size_category: SizeCategory::Medium,
    });

    collector.process_pending();
    assert_eq!(collector.len(), 2);
}

#[test]
fn media_failed_reason_is_sanitized() {
    let mut collector = DiagnosticsCollector::new(BufferCapacity::default());
    let handle = collector.handle();

    handle.log_state(AppStateEvent::MediaFailed {
        media_type: MediaType::Image,
        reason: "IO error: NotFound".to_string(), // No path!
    });

    collector.process_pending();
    // Verify reason doesn't contain path separators
}
```

---

## Dev Agent Record

### Change Log
| Date | Change | Author |
|------|--------|--------|
| 2026-01-13 | Story created from Story 1.8 split | Claude Opus 4.5 |
| 2026-01-13 | PO Validation: Added comprehensive Dev Notes, Testing section, Task-AC mappings, corrected event fields | PO Validation |

---
