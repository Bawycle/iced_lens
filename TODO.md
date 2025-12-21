# TODO - Next Release

> This file tracks planned work for the next release. It lives only in the `dev` branch and is not included in releases.

## Bugs to Fix


## Planned Features

### Viewer
- [ ] Temporary rotation in viewer (90° increments, session-only) — currently complex to implement

#### Media Filters for Navigation
- [ ] Add filterable navigation to show only matching media in the current directory

**Filter categories to consider:**

| Category | Filters | Notes |
|----------|---------|-------|
| **Media type** | Images only, Videos only, All | Basic, high priority |
| **Format** | JPEG, PNG, GIF, WebP, MP4, MKV... | Useful for batch processing |
| **Aspect ratio** | Landscape, Portrait, Square | Common photography need |
| **Animation** | Animated (GIF/WebP/video), Static | Distinguish animated content |
| **Audio** | Videos with audio, Silent videos | Filter silent screen recordings |
| **Resolution** | By megapixels ranges, 4K+, HD, SD | For quality filtering |
| **File size** | Small (<1MB), Medium, Large (>10MB) | Storage management |
| **Date** | Today, This week, This month, Custom range | Time-based browsing |
| **Metadata** | Has EXIF, Has GPS, Has title/description | Find tagged content |
| **Camera** | By make/model (from EXIF) | Organize by device |

**UX considerations:**
- Filter UI: dropdown in navbar? sidebar panel? keyboard shortcut?
- Show filter status indicator (e.g., "Showing 42/156 images")
- Remember last filter per directory? Or global preference?
- Quick toggle vs advanced filter panel

**Image editor interaction:**
- When filters are active and user enters editor, what happens on navigation?
  - Option A: Editor ignores filters, navigates all images (may be confusing)
  - Option B: Editor respects filters (consistent but may limit access)
  - Option C: Editor auto-filters to "images only" regardless of viewer filter
- Need to handle case where filter excludes all images (disable Edit button?)

#### Metadata Sidebar
- [ ] Allow text selection and copying in the metadata sidebar (blocked, pending native support in Iced 0.15.0)
- [ ] Add video metadata editing support (Phase 2 - future work)

### Video Player

### Help
- [ ] Allow text selection and copying in the help screen (blocked, pending native support in Iced 0.15.0)

### Video Editor
- [ ] Create a simple video editor allowing users to trim videos by removing segments. The editor should let users play the video, seek to any position, step forward/backward frame by frame, and change the playback speed.

## Code Quality / Refactoring

### Newtype Pattern for Bounded Config Values
- [ ] Refactor bounded configuration values to use the Newtype pattern for compile-time safety

**Rationale:**
- Currently, bounded values use `clamp()` at multiple points (config loading, settings state)
- No compile-time guarantee that values are within valid range
- Risk of mixing up different `u32` values (e.g., `frame_cache_mb` vs `max_skip_attempts`)

**Values to refactor:**

| Current | Newtype | Range |
|---------|---------|-------|
| `max_skip_attempts: u32` | `MaxSkipAttempts` | 1-20 |
| `frame_cache_mb: u32` | `FrameCacheMb` | MIN-MAX |
| `frame_history_mb: u32` | `FrameHistoryMb` | MIN-MAX |
| `overlay_timeout_secs: u32` | `OverlayTimeoutSecs` | MIN-MAX |
| `keyboard_seek_step_secs: f64` | `KeyboardSeekStep` | MIN-MAX |
| `zoom_step_percent: f32` | `ZoomStepPercent` | MIN-MAX |

**Implementation approach:**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MaxSkipAttempts(u32);

impl MaxSkipAttempts {
    pub const MIN: u32 = 1;
    pub const MAX: u32 = 20;
    pub const DEFAULT: Self = Self(5);

    pub fn new(value: u32) -> Self {
        Self(value.clamp(Self::MIN, Self::MAX))
    }

    pub fn get(self) -> u32 {
        self.0
    }
}
```

**Benefits:**
- Invariant enforced by type system
- Single source of truth for validation
- Self-documenting code
- Prevents accidental value mix-ups

## Notes

- Test videos can be generated with `scripts/generate-test-videos.sh`
