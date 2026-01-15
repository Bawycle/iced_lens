# Epic 4 Initial Audit: Diagnostic Collection Points

**Audit Type:** Architectural Compliance & Coverage Assessment
**Auditor:** Quinn (Test Architect)
**Date:** 2025-01-15
**Status:** Complete
**Reference:** [Architecture Review Report](../../reports/architecture-review-diagnostics-navigation.md)

---

## Executive Summary

This audit evaluates all existing diagnostic collection points against the newly established architectural criteria (Section 5.2.1 of architect-checklist.md). The audit identifies violations, gaps, and provides actionable recommendations for Epic 4 stories.

### Key Metrics

| Category | Defined | Logged | Coverage | Target |
|----------|---------|--------|----------|--------|
| UserAction | 35 | 10 | 29% | 100% |
| AppStateEvent | 21 | 14 | 67% | 100% |
| AppOperation | 8 | 3 | 38% | 100% |
| **Total** | **64** | **27** | **42%** | **100%** |

### Critical Findings

| Finding | Severity | Count | Impact |
|---------|----------|-------|--------|
| UI-layer collection points | HIGH | 5 | Violates R1 (handler-level collection) |
| Missing user action coverage | MEDIUM | 25 | Incomplete user behavior tracking |
| Missing state event coverage | MEDIUM | 7 | Incomplete state transition tracking |
| Missing operation coverage | LOW | 5 | Incomplete performance tracking |
| Missing filter events | HIGH | 0 defined | Cannot analyze filter usage |
| Missing navigation context | HIGH | N/A | Cannot distinguish Viewer/Editor |

---

## 1. Architectural Violations

### 1.1 UI-Layer Collection Points (Rule R1 Violations)

**Rule R1:** Collect at handler level, not UI components

The following 5 collection points are located in UI components instead of App handlers:

| # | File | Line | Event | Severity | Recommended Location |
|---|------|------|-------|----------|---------------------|
| V1 | `src/ui/viewer/component.rs` | 652 | `AppStateEvent::MediaLoadingStarted` | HIGH | `src/app/update.rs` handler |
| V2 | `src/ui/viewer/component.rs` | 866 | `AppStateEvent::MediaLoaded` | HIGH | `src/app/update.rs` handler |
| V3 | `src/ui/viewer/component.rs` | 946 | `AppStateEvent::MediaFailed` | HIGH | `src/app/update.rs` handler |
| V4 | `src/ui/viewer/component.rs` | 1147 | `UserAction::TogglePlayback` | MEDIUM | `src/app/update.rs` handler |
| V5 | `src/ui/viewer/component.rs` | 1197 | `UserAction::SeekVideo` | MEDIUM | `src/app/update.rs` handler |

**Impact:** UI components should not have direct knowledge of diagnostics. This creates tight coupling and makes it harder to maintain consistent instrumentation.

**Recommendation:** Move these collection points to the corresponding message handlers in `src/app/update.rs`. The events can be emitted when the App receives the messages/effects from the viewer component.

### 1.2 Context-Deficient Events (Rule R4 Violations)

**Rule R4:** Include sufficient context (self-describing events)

| Event | Current Context | Missing Context |
|-------|-----------------|-----------------|
| `NavigateNext` | None | Navigation context (Viewer/Editor), filter_active, position |
| `NavigatePrevious` | None | Navigation context (Viewer/Editor), filter_active, position |
| `EnterEditor` | None | Source image info, dimensions |
| `TogglePlayback` | None | Current playback state (was playing/paused) |

---

## 2. Coverage Gap Analysis

### 2.1 UserAction Coverage (29% - 10/35)

#### Logged Actions (10)

| Action | Location | Layer | Compliance |
|--------|----------|-------|------------|
| `NavigateNext` | update.rs:1274 | Handler | ✅ OK |
| `NavigatePrevious` | update.rs:1289 | Handler | ✅ OK |
| `LoadMedia` | update.rs:1525,1554 | Handler | ✅ OK |
| `TogglePlayback` | component.rs:1147 | UI | ⚠️ VIOLATION |
| `SeekVideo` | component.rs:1197 | UI | ⚠️ VIOLATION |
| `OpenSettings` | update.rs:947 | Handler | ✅ OK |
| `OpenHelp` | update.rs:952 | Handler | ✅ OK |
| `OpenAbout` | update.rs:957 | Handler | ✅ OK |
| `OpenDiagnostics` | update.rs:962 | Handler | ✅ OK |
| `EnterEditor` | update.rs:967 | Handler | ✅ OK |

#### Missing Actions (25) - Grouped by Story

**Story 4.3a - Editor Actions (8 actions):**
- `ApplyCrop`
- `ApplyResize`
- `ApplyDeblur`
- `ApplyUpscale`
- `SaveImage`
- `Undo`
- `Redo`
- `ReturnToViewer`

**Story 4.3b - Video/Audio Actions (7 actions):**
- `DeleteMedia`
- `StepForward`
- `StepBackward`
- `SetPlaybackSpeed`
- `ToggleLoop`
- `SetVolume`
- `ToggleMute`

**Story 4.3b - Capture/Export Actions (2 actions):**
- `CaptureFrame`
- `ExportFile`

**Story 4.3c - View Controls (8 actions):**
- `ZoomIn`
- `ZoomOut`
- `ResetZoom`
- `ToggleFitToWindow`
- `ToggleFullscreen`
- `ExitFullscreen`
- `RotateClockwise`
- `RotateCounterClockwise`

### 2.2 AppStateEvent Coverage (67% - 14/21)

#### Logged Events (14)

| Event | Location | Layer | Compliance |
|-------|----------|-------|------------|
| `MediaLoadingStarted` | component.rs:652 | UI | ⚠️ VIOLATION |
| `MediaLoaded` | component.rs:866 | UI | ⚠️ VIOLATION |
| `MediaFailed` | component.rs:946 | UI | ⚠️ VIOLATION |
| `VideoPlaying` | state.rs:370 | Domain | ✅ OK |
| `VideoPaused` | state.rs:411 | Domain | ✅ OK |
| `VideoSeeking` | state.rs:495 | Domain | ✅ OK |
| `VideoBuffering` | state.rs:612 | Domain | ✅ OK |
| `VideoError` | state.rs:620 | Domain | ✅ OK |
| `VideoAtEndOfStream` | state.rs:322 | Domain | ✅ OK |
| `EditorOpened` | update.rs:360 | Handler | ✅ OK |
| `EditorClosed` | update.rs:389,699 | Handler | ✅ OK |
| `EditorDeblurStarted` | update.rs:782 | Handler | ✅ OK |
| `EditorDeblurCompleted` | mod.rs:892 | Handler | ✅ OK |

**Note:** Video player state events in `state.rs` (Domain layer) are acceptable as they represent state machine transitions.

#### Missing Events (7) - Grouped by Story

**Story 4.4 - Video Playback Events (2 events):**
- `VideoLoopToggled`
- `VideoSpeedChanged`

**Story 4.4 - Editor Events (3 events):**
- `EditorDeblurProgress`
- `EditorDeblurCancelled`
- `EditorUnsavedChanges`

**Story 4.4 - AI Model Events (3 events):**
- `ModelDownloadStarted`
- `ModelDownloadCompleted`
- `ModelDownloadFailed`

### 2.3 AppOperation Coverage (38% - 3/8)

#### Logged Operations (3)

| Operation | Location | Layer | Compliance |
|-----------|----------|-------|------------|
| `AIDeblurProcess` | mod.rs:886,902 | Handler | ✅ OK |
| `AIUpscaleProcess` | mod.rs:955,972 | Handler | ✅ OK |
| `VideoSeek` | state.rs:236 | Domain | ✅ OK |

#### Missing Operations (5)

| Operation | Priority | Notes |
|-----------|----------|-------|
| `DecodeFrame` | P3 | Performance tracking for frame decoding |
| `ResizeImage` | P3 | Performance tracking for image resize |
| `ApplyFilter` | P3 | Performance tracking for filter application |
| `ExportFrame` | P3 | Performance tracking for frame export |
| `LoadMetadata` | P3 | Performance tracking for metadata loading |

**Note:** These operations are lower priority (P3) as they primarily serve performance analysis rather than user behavior tracking.

---

## 3. Missing Event Definitions

### 3.1 Filter Events (Story 4.1)

The following events need to be **defined** in `events.rs` (they don't exist yet):

```rust
// Proposed additions to AppStateEvent
FilterChanged {
    filter_type: FilterChangeType,
    previous_active: bool,
    new_active: bool,
    filtered_count: usize,
    total_count: usize,
}

FilterCleared {
    had_media_type_filter: bool,
    had_date_filter: bool,
}

// New enum needed
pub enum FilterChangeType {
    MediaType { from: MediaTypeFilter, to: MediaTypeFilter },
    DateRangeEnabled,
    DateRangeDisabled,
    DateFieldChanged { field: DateFilterField },
    DateBoundSet { target: DateTarget },
    DateBoundCleared { target: DateTarget },
}
```

### 3.2 Navigation Context (Story 4.2)

The following modifications are needed:

```rust
// New enum needed
pub enum NavigationContext {
    Viewer,
    Editor,
}

// Modify existing UserAction variants
NavigateNext {
    context: NavigationContext,
    filter_active: bool,
    position_in_filtered: Option<usize>,
    position_in_total: usize,
}

NavigatePrevious {
    context: NavigationContext,
    filter_active: bool,
    position_in_filtered: Option<usize>,
    position_in_total: usize,
}
```

---

## 4. Corrective Actions by Story

### Story 4.1: Filter Events

| Action ID | Description | Severity | Files Affected |
|-----------|-------------|----------|----------------|
| 4.1-A1 | Define `FilterChangeType` enum | HIGH | `events.rs` |
| 4.1-A2 | Define `FilterChanged` event | HIGH | `events.rs` |
| 4.1-A3 | Define `FilterCleared` event | HIGH | `events.rs` |
| 4.1-A4 | Add collection point in `handle_filter_changed()` | HIGH | `update.rs` |
| 4.1-A5 | Add integration tests | MEDIUM | `diagnostics/tests/` |

### Story 4.2: Navigation Context

| Action ID | Description | Severity | Files Affected |
|-----------|-------------|----------|----------------|
| 4.2-A1 | Define `NavigationContext` enum | HIGH | `events.rs` |
| 4.2-A2 | Modify `NavigateNext` variant with context fields | HIGH | `events.rs` |
| 4.2-A3 | Modify `NavigatePrevious` variant with context fields | HIGH | `events.rs` |
| 4.2-A4 | Update collection points in handlers | MEDIUM | `update.rs` |
| 4.2-A5 | Add integration tests | MEDIUM | `diagnostics/tests/` |

### Story 4.3a: Editor Actions

| Action ID | Description | Severity | Files Affected |
|-----------|-------------|----------|----------------|
| 4.3a-A1 | Add logging for `ApplyCrop` | MEDIUM | `update.rs` |
| 4.3a-A2 | Add logging for `ApplyResize` | MEDIUM | `update.rs` |
| 4.3a-A3 | Add logging for `ApplyDeblur` | MEDIUM | `update.rs` |
| 4.3a-A4 | Add logging for `ApplyUpscale` | MEDIUM | `update.rs` |
| 4.3a-A5 | Add logging for `SaveImage` | MEDIUM | `update.rs` |
| 4.3a-A6 | Add logging for `Undo` | MEDIUM | `update.rs` |
| 4.3a-A7 | Add logging for `Redo` | MEDIUM | `update.rs` |
| 4.3a-A8 | Add logging for `ReturnToViewer` | MEDIUM | `update.rs` |

### Story 4.3b: Video/Audio Actions

| Action ID | Description | Severity | Files Affected |
|-----------|-------------|----------|----------------|
| 4.3b-A1 | Add logging for `SetVolume` | MEDIUM | `update.rs` |
| 4.3b-A2 | Add logging for `ToggleMute` | MEDIUM | `update.rs` |
| 4.3b-A3 | Add logging for `DeleteMedia` | MEDIUM | `update.rs` |
| 4.3b-A4 | Add logging for `CaptureFrame` | MEDIUM | `update.rs` |
| 4.3b-A5 | Add logging for `ExportFile` | MEDIUM | `update.rs` |
| 4.3b-A6 | Add logging for `StepForward` | LOW | `update.rs` |
| 4.3b-A7 | Add logging for `StepBackward` | LOW | `update.rs` |
| 4.3b-A8 | Add logging for `SetPlaybackSpeed` | LOW | `update.rs` |
| 4.3b-A9 | Add logging for `ToggleLoop` | LOW | `update.rs` |

### Story 4.3c: View Controls

| Action ID | Description | Severity | Files Affected |
|-----------|-------------|----------|----------------|
| 4.3c-A1 | Add logging for `ZoomIn` | MEDIUM | `update.rs` |
| 4.3c-A2 | Add logging for `ZoomOut` | MEDIUM | `update.rs` |
| 4.3c-A3 | Add logging for `ResetZoom` | MEDIUM | `update.rs` |
| 4.3c-A4 | Add logging for `ToggleFitToWindow` | LOW | `update.rs` |
| 4.3c-A5 | Add logging for `RotateClockwise` | MEDIUM | `update.rs` |
| 4.3c-A6 | Add logging for `RotateCounterClockwise` | MEDIUM | `update.rs` |
| 4.3c-A7 | Add logging for `ToggleFullscreen` | LOW | `update.rs` |
| 4.3c-A8 | Add logging for `ExitFullscreen` | LOW | `update.rs` |

### Story 4.4: State Events

| Action ID | Description | Severity | Files Affected |
|-----------|-------------|----------|----------------|
| 4.4-A1 | Add logging for `VideoLoopToggled` | MEDIUM | `state.rs` |
| 4.4-A2 | Add logging for `VideoSpeedChanged` | MEDIUM | `state.rs` |
| 4.4-A3 | Add logging for `EditorDeblurProgress` | LOW | `update.rs` |
| 4.4-A4 | Add logging for `EditorDeblurCancelled` | MEDIUM | `update.rs` |
| 4.4-A5 | Add logging for `ModelDownloadStarted` | MEDIUM | AI engine code |
| 4.4-A6 | Add logging for `ModelDownloadCompleted` | MEDIUM | AI engine code |
| 4.4-A7 | Add logging for `ModelDownloadFailed` | MEDIUM | AI engine code |

### Violation Fixes (Cross-Story)

| Action ID | Description | Severity | Affected Story |
|-----------|-------------|----------|----------------|
| VF-1 | Move `MediaLoadingStarted` to handler | HIGH | Pre-req |
| VF-2 | Move `MediaLoaded` to handler | HIGH | Pre-req |
| VF-3 | Move `MediaFailed` to handler | HIGH | Pre-req |
| VF-4 | Move `TogglePlayback` to handler | MEDIUM | 4.3b |
| VF-5 | Move `SeekVideo` to handler | MEDIUM | 4.3b |

---

## 5. Recommendations

### 5.1 Immediate Actions (Before Stories)

1. **Fix architectural violations first** - Move the 5 UI-layer collection points to handlers before implementing new instrumentation
2. **Create prerequisite story** - Consider a "Story 4.0: Fix Collection Point Violations" to address VF-1 through VF-5

### 5.2 Story Refinements

Based on this audit, the preliminary stories should be refined:

| Story | Original Scope | Refined Scope |
|-------|---------------|---------------|
| 4.1 | Filter events | Filter events + new enum definitions |
| 4.2 | Navigation context | Navigation context + modify existing events |
| 4.3a | 8 editor actions | 8 editor actions (no change) |
| 4.3b | 5 video/audio actions | 9 video/audio actions (add StepForward, StepBackward, SetPlaybackSpeed, ToggleLoop) + fix 2 violations |
| 4.3c | 8 view controls | 8 view controls (no change) |
| 4.4 | 7 state events | 7 state events (no change) |

### 5.3 Testing Strategy

Each story should include:
1. **Unit tests** for new event serialization
2. **Integration tests** verifying event capture at collection points
3. **Performance tests** confirming < 1ms overhead

---

## 6. Audit Summary

### Compliance Score

| Criterion | Score | Notes |
|-----------|-------|-------|
| Coverage completeness | 42% | 27/64 events logged |
| Architectural placement | 81% | 22/27 logged events at correct layer |
| Context sufficiency | 60% | Several events missing context |
| **Overall** | **61%** | Significant improvement needed |

### Risk Assessment

| Risk | Probability | Impact | Score | Mitigation |
|------|-------------|--------|-------|------------|
| Incomplete user behavior data | HIGH | HIGH | 9 | Implement all missing user actions |
| UI-handler coupling | MEDIUM | MEDIUM | 4 | Move collection points to handlers |
| Missing filter analytics | HIGH | MEDIUM | 6 | Implement filter events (Story 4.1) |
| Indistinguishable navigation | HIGH | MEDIUM | 6 | Implement context (Story 4.2) |

### Next Steps

1. Review this audit with the team
2. Create/refine Epic 4 stories based on corrective actions
3. Prioritize violation fixes (VF-1 to VF-5) as prerequisites
4. Execute stories in dependency order
5. Re-audit after implementation (Phase 3 - Step 3.1)

---

## Appendix: Collection Point Inventory

### Current Collection Points (28 total)

| # | File | Line(s) | Event Type | Layer | Compliant |
|---|------|---------|------------|-------|-----------|
| 1 | update.rs | 947 | UserAction::OpenSettings | Handler | ✅ |
| 2 | update.rs | 952 | UserAction::OpenHelp | Handler | ✅ |
| 3 | update.rs | 957 | UserAction::OpenAbout | Handler | ✅ |
| 4 | update.rs | 962 | UserAction::OpenDiagnostics | Handler | ✅ |
| 5 | update.rs | 967 | UserAction::EnterEditor | Handler | ✅ |
| 6 | update.rs | 1274 | UserAction::NavigateNext | Handler | ✅ |
| 7 | update.rs | 1289 | UserAction::NavigatePrevious | Handler | ✅ |
| 8 | update.rs | 1525,1554 | UserAction::LoadMedia | Handler | ✅ |
| 9 | update.rs | 360 | AppStateEvent::EditorOpened | Handler | ✅ |
| 10 | update.rs | 389,699 | AppStateEvent::EditorClosed | Handler | ✅ |
| 11 | update.rs | 782 | AppStateEvent::EditorDeblurStarted | Handler | ✅ |
| 12 | mod.rs | 886,902 | AppOperation::AIDeblurProcess | Handler | ✅ |
| 13 | mod.rs | 892 | AppStateEvent::EditorDeblurCompleted | Handler | ✅ |
| 14 | mod.rs | 955,972 | AppOperation::AIUpscaleProcess | Handler | ✅ |
| 15 | component.rs | 652 | AppStateEvent::MediaLoadingStarted | UI | ❌ |
| 16 | component.rs | 866 | AppStateEvent::MediaLoaded | UI | ❌ |
| 17 | component.rs | 946 | AppStateEvent::MediaFailed | UI | ❌ |
| 18 | component.rs | 1147 | UserAction::TogglePlayback | UI | ❌ |
| 19 | component.rs | 1197 | UserAction::SeekVideo | UI | ❌ |
| 20 | state.rs | 236 | AppOperation::VideoSeek | Domain | ✅ |
| 21 | state.rs | 322 | AppStateEvent::VideoAtEndOfStream | Domain | ✅ |
| 22 | state.rs | 370 | AppStateEvent::VideoPlaying | Domain | ✅ |
| 23 | state.rs | 411 | AppStateEvent::VideoPaused | Domain | ✅ |
| 24 | state.rs | 495 | AppStateEvent::VideoSeeking | Domain | ✅ |
| 25 | state.rs | 612 | AppStateEvent::VideoBuffering | Domain | ✅ |
| 26 | state.rs | 620 | AppStateEvent::VideoError | Domain | ✅ |
| 27 | manager.rs | 62 | WarningEvent | Infrastructure | ✅ |
| 28 | manager.rs | 67 | ErrorEvent | Infrastructure | ✅ |

---

*Audit completed by Quinn, Test Architect*
*This audit should be referenced when refining Epic 4 stories*
