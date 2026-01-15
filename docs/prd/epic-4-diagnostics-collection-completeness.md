# Epic 4: Diagnostics Collection Completeness

**Goal:** Achieve 100% diagnostic event coverage with proper architectural placement, ensuring all user actions, state events, and operations are instrumented at the correct abstraction layer.

## Background

An architectural review identified significant gaps in diagnostic instrumentation:
- **Current Coverage:** 42% (27/64 events logged)
- **Architectural Violations:** 5 collection points in UI layer instead of handler level
- **Missing Events:** Filter changes, navigation context, editor actions, view controls
- **Reference:** [Architecture Review Report](../reports/architecture-review-diagnostics-navigation.md)
- **Initial Audit:** [Epic 4 Initial Audit](../qa/assessments/epic4-initial-audit.md)

## Story 4.0: Fix Collection Point Architectural Violations

**As a** developer maintaining the diagnostics system,
**I want** all collection points to be at the handler level,
**So that** UI components remain decoupled from diagnostics and instrumentation is consistent.

**Acceptance Criteria:**
1. `MediaLoadingStarted` event moved from `component.rs:652` to `update.rs` handler
2. `MediaLoaded` event moved from `component.rs:866` to `update.rs` handler
3. `MediaFailed` event moved from `component.rs:946` to `update.rs` handler
4. `TogglePlayback` logging moved from `component.rs:1147` to `update.rs` handler
5. `SeekVideo` logging moved from `component.rs:1197` to `update.rs` handler
6. All 5 collection points removed from `ui/viewer/component.rs`
7. No functional regression in event capture
8. Integration tests verify events still captured correctly

## Story 4.1: Navigation Filter Diagnostic Events

**As a** developer analyzing user behavior,
**I want** filter activation/deactivation events captured,
**So that** I can understand how users use navigation filters.

**Acceptance Criteria:**
1. `FilterChangeType` enum defined in `events.rs` with variants: `MediaType`, `DateRangeEnabled`, `DateRangeDisabled`, `DateFieldChanged`, `DateBoundSet`, `DateBoundCleared`
2. `FilterChanged` AppStateEvent defined with: `filter_type`, `previous_active`, `new_active`, `filtered_count`, `total_count`
3. `FilterCleared` AppStateEvent defined with: `had_media_type_filter`, `had_date_filter`
4. Collection point in `handle_filter_changed()` at handler level
5. Events emitted for all filter operations: set, modify, clear
6. Integration tests verify event capture for each filter operation
7. No performance regression (< 1ms overhead per event)

## Story 4.2: Viewer/Editor Navigation Context

**As a** developer analyzing navigation patterns,
**I want** navigation events to include Viewer/Editor context,
**So that** I can distinguish navigation behavior between modes.

**Acceptance Criteria:**
1. `NavigationContext` enum defined with `Viewer` and `Editor` variants
2. `NavigateNext` UserAction modified to include: `context: NavigationContext`, `filter_active: bool`, `position_in_filtered: Option<usize>`, `position_in_total: usize`
3. `NavigatePrevious` UserAction modified with same fields
4. Viewer navigation events explicitly marked with `NavigationContext::Viewer`
5. Editor navigation events explicitly marked with `NavigationContext::Editor`
6. Integration tests verify context is correctly captured in both modes
7. Existing navigation tests updated to expect new event structure

## Story 4.3a: Editor Actions Instrumentation

**As a** developer analyzing feature usage,
**I want** all editor actions logged,
**So that** I can understand which editing features are most used.

**Acceptance Criteria:**
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

## Story 4.3b: Video/Audio Actions Instrumentation

**As a** developer analyzing playback behavior,
**I want** all video and audio control actions logged,
**So that** I can understand playback interaction patterns.

**Acceptance Criteria:**
1. `SetVolume` action logged with volume level
2. `ToggleMute` action logged with resulting mute state
3. `DeleteMedia` action logged
4. `CaptureFrame` action logged with capture timestamp
5. `ExportFile` action logged with export format
6. `StepForward` action logged
7. `StepBackward` action logged
8. `SetPlaybackSpeed` action logged with speed value
9. `ToggleLoop` action logged with resulting loop state
10. Relocate `TogglePlayback` and `SeekVideo` logging from UI to handler (from Story 4.0)
11. All collection points at handler level
12. Integration tests verify each action is captured

## Story 4.3c: View Controls Instrumentation

**As a** developer analyzing view preferences,
**I want** all view control actions logged,
**So that** I can understand how users interact with view controls.

**Acceptance Criteria:**
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

## Story 4.4: Missing State Events Instrumentation

**As a** developer analyzing state transitions,
**I want** all state events logged,
**So that** I can track complete state machine behavior.

**Acceptance Criteria:**
1. `VideoLoopToggled` event emitted with `enabled: bool`
2. `VideoSpeedChanged` event emitted with `speed: f64`
3. `EditorDeblurProgress` event emitted with `percent: f32`
4. `EditorDeblurCancelled` event emitted when deblur cancelled
5. `ModelDownloadStarted` event emitted with `model: AIModel`
6. `ModelDownloadCompleted` event emitted with `model: AIModel`
7. `ModelDownloadFailed` event emitted with `model: AIModel`, `reason: String`
8. Video events emitted from `state.rs` (Domain layer - acceptable)
9. AI model events emitted from AI engine code at lifecycle points
10. Integration tests verify each state event is captured

## Story 4.5: Final Conformance Audit

**As a** QA architect,
**I want** a final audit validating 100% coverage,
**So that** I can confirm Epic 4 objectives are met.

**Acceptance Criteria:**
1. All `UserAction` variants are logged (100% coverage)
2. All `AppStateEvent` variants are logged (100% coverage)
3. All `AppOperation` variants are logged (100% coverage)
4. No collection points in UI components (handler level only, except Domain state machines)
5. All events include sufficient context (self-describing)
6. All events are non-blocking (channel-based)
7. No sensitive data in event details (paths sanitized)
8. All stories 4.0-4.4 have passing integration tests
9. Performance overhead < 1ms per event
10. Final audit report produced with before/after comparison

---

## Compatibility Requirements

- [x] Existing event serialization format unchanged (additive changes only)
- [x] DiagnosticsCollector API unchanged
- [x] Export functionality continues to work
- [x] Existing tests continue to pass
- [ ] No breaking changes to diagnostic report schema

## Risk Mitigation

- **Primary Risk:** Modifying collection points could break event capture
- **Mitigation:** Integration tests verify event capture before and after changes
- **Rollback Plan:** Revert commits if event capture regresses

## Definition of Done

- [ ] All 8 stories completed with acceptance criteria met
- [ ] Event coverage at 100% (64/64 events logged)
- [ ] No architectural violations (0 UI-layer collection points)
- [ ] All integration tests pass
- [ ] Final audit report approved
- [ ] No regression in existing functionality

---

## Story Dependencies

```
Story 4.0 (Prerequisite)
    ├── Story 4.1 (can start after 4.0)
    ├── Story 4.2 (can start after 4.0)
    ├── Story 4.3a (can start after 4.0)
    ├── Story 4.3b (can start after 4.0, includes 4.0 relocations)
    └── Story 4.3c (can start after 4.0)
           │
           ▼
       Story 4.4 (after 4.1-4.3)
           │
           ▼
       Story 4.5 (after all others)
```

---

## Reference Documents

- [Architecture Review Report](../reports/architecture-review-diagnostics-navigation.md)
- [Epic 4 Coordination Plan](../reports/epic4-coordination-plan.md)
- [Epic 4 Initial Audit](../qa/assessments/epic4-initial-audit.md)
- [Architect Checklist - Section 5.2.1](../../.bmad-core/checklists/architect-checklist.md)
- [Story DoD Checklist - Section 6.1](../../.bmad-core/checklists/story-dod-checklist.md)

---
