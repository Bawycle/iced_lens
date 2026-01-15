# Architecture Review Report: Diagnostics Collection & Navigation Context

**Report Type:** Architecture Analysis & Recommendations
**Date:** 2025-01-15
**Author:** Winston (Architect Agent)
**Status:** Ready for PO Review
**Target Audience:** Product Owner, Technical Lead

---

## Executive Summary

This report analyzes two architectural concerns raised during the diagnostics feature development:

1. **Navigation Filter Diagnostics Gap**: Filter activation/deactivation events are not collected, and there is no distinction between Viewer and Editor navigation contexts in diagnostic data.

2. **QA Process Gap**: The current QA process validates functional correctness but does not systematically validate the architectural placement of diagnostic collection points.

**Key Findings:**
- 56% of defined diagnostic events are never logged in production code
- Navigation filter changes generate no diagnostic data
- Viewer vs Editor context is only implicitly inferable from event sequences
- QA checklists lack criteria for collection point placement validation

**Recommendation:** Create a new Epic to address these gaps with 4-6 stories covering event taxonomy enrichment, collection point audit, and QA process updates.

---

## Table of Contents

1. [Current State Analysis](#1-current-state-analysis)
2. [Gap Analysis: Navigation Filters](#2-gap-analysis-navigation-filters)
3. [Gap Analysis: Viewer vs Editor Context](#3-gap-analysis-viewer-vs-editor-context)
4. [Gap Analysis: QA Architectural Validation](#4-gap-analysis-qa-architectural-validation)
5. [Recommended Changes](#5-recommended-changes)
6. [Proposed Event Taxonomy Updates](#6-proposed-event-taxonomy-updates)
7. [Collection Point Architecture](#7-collection-point-architecture)
8. [QA Process Enhancements](#8-qa-process-enhancements)
9. [Implementation Roadmap](#9-implementation-roadmap)
10. [Appendices](#appendices)

---

## 1. Current State Analysis

### 1.1 Diagnostics Event Coverage

| Category | Defined | Logged | Coverage |
|----------|---------|--------|----------|
| User Actions | 36 | 18 | 50% |
| App State Events | 24 | 17 | 71% |
| App Operations | 8 | 5 | 63% |
| Warning/Error | 2 | 2 | 100% |
| **Total** | **70** | **42** | **60%** |

### 1.2 Navigation Filter System

The application supports two filter types:
- **MediaTypeFilter**: All / Images Only / Videos Only
- **DateRangeFilter**: By Modified or Created date with start/end bounds

Filter state is stored in `MediaNavigator.filter` and applied via `peek_*_filtered()` methods.

### 1.3 Current Collection Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    COLLECTION LAYERS                         │
├─────────────────────────────────────────────────────────────┤
│  UI Components          → Local state changes (no logging)  │
│  App Handlers           → UserAction, Screen transitions    │
│  Domain Logic           → Video playback state events       │
│  Notification Manager   → Warnings and Errors (auto-logged) │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. Gap Analysis: Navigation Filters

### 2.1 Current Behavior

| User Action | Diagnostic Event | Status |
|-------------|------------------|--------|
| Open filter dropdown | None | ❌ Not collected |
| Select media type filter | None | ❌ Not collected |
| Enable date filter | None | ❌ Not collected |
| Set date range bounds | None | ❌ Not collected |
| Clear filters | None | ❌ Not collected |

### 2.2 Data Flow Without Collection

```
User clicks filter → FilterDropdown::Message → Navbar::Event::FilterChanged
    → App::handle_filter_changed() → MediaNavigator::set_filter()
                                            ↓
                                   NO DIAGNOSTIC EVENT
```

### 2.3 Impact

Without filter diagnostics, we cannot answer:
- Which filters are most commonly used?
- Do users prefer date filtering by Modified or Created?
- How often do users reset filters vs. manually clearing?
- Is the filtered navigation UX effective?

### 2.4 Available Context at Collection Point

At `handle_filter_changed()` (src/app/update.rs:1600-1667):

| Data | Availability | Notes |
|------|--------------|-------|
| Previous filter state | ✅ Available | Via `ctx.media_navigator.filter()` |
| New filter state | ✅ Available | From message payload |
| Total file count | ✅ Available | Via `ctx.media_navigator.len()` |
| Filtered file count | ✅ Available | Via `ctx.media_navigator.filtered_count()` |
| Current media path | ✅ Available | Via `ctx.media_navigator.current()` |
| Filter persistence setting | ✅ Available | Via `ctx.config.display.persist_filters` |

---

## 3. Gap Analysis: Viewer vs Editor Context

### 3.1 Architectural Distinction

The codebase correctly separates Viewer and Editor navigation:

| Context | Navigation Method | Filter Behavior |
|---------|-------------------|-----------------|
| **Viewer** | `peek_next_filtered()` | Respects user filters |
| **Editor** | `peek_next_image()` | Images only, ignores filters |

### 3.2 Diagnostic Context Gap

Current events do not capture navigation context:

```rust
// Current: No context
UserAction::NavigateNext  // Viewer or Editor? Unknown.

// Needed: With context
UserAction::NavigateNext {
    context: NavigationContext::Viewer,
    filter_active: true,
    filtered_position: 5,
    total_position: 12,
}
```

### 3.3 Implicit Context (Current Workaround)

Context can be *inferred* from event sequences:
1. `UserAction::EnterEditor` → subsequent events are in Editor
2. `AppStateEvent::EditorClosed` → subsequent events are in Viewer

**Problems with inference:**
- Requires event sequence analysis (complex)
- Session boundaries may lose context
- No explicit confirmation of context at event time

### 3.4 Impact

Without explicit context:
- Cannot compare Viewer vs Editor navigation patterns
- Cannot measure filter effectiveness (only applicable in Viewer)
- Cannot analyze Editor-specific workflows separately

---

## 4. Gap Analysis: QA Architectural Validation

### 4.1 Current QA Checklist Coverage

| Validation Area | Architect Checklist | Story DoD | QA Gate |
|-----------------|---------------------|-----------|---------|
| Monitoring exists | ✅ Section 5.2 | ❌ | ❌ |
| Collection point placement | ❌ | ❌ | ❌ |
| Instrumentation completeness | ❌ | ❌ | ❌ |
| Event taxonomy consistency | ❌ | ❌ | ❌ |
| Duplicate detection | ❌ | ❌ | ❌ |
| Performance overhead | ❌ | Mentioned | ❌ |

### 4.2 Evidence from QA Gates

Analysis of 25 QA gate files from Epic 1-3:

**What IS validated:**
- Test coverage counts (e.g., "40 tests reviewed")
- NFR compliance (Security, Performance, Reliability, Maintainability)
- Pattern adherence (Newtype, Elm/Iced architecture)

**What IS NOT validated:**
- Whether all user-visible actions have corresponding events
- Whether collection points are at appropriate architectural layers
- Whether event data captures sufficient context
- Whether there are gaps or duplicates in instrumentation

### 4.3 Specific Example

**Story 1.7** (State/Operation Instrumentation) QA Gate:
- ✅ Validates that instrumentation exists
- ❌ Does not verify ALL handlers are instrumented
- ❌ Does not audit for missing events vs. defined taxonomy
- ❌ Does not validate placement follows architectural principles

---

## 5. Recommended Changes

### 5.1 Event Taxonomy Changes

| Change Type | Event | Rationale |
|-------------|-------|-----------|
| **ADD** | `FilterChanged` | Track all filter modifications |
| **ADD** | `FilterCleared` | Distinct event for reset action |
| **MODIFY** | `NavigateNext/Previous` | Add context fields |
| **ADD** | `NavigationContextChanged` | Track Viewer↔Editor transitions |
| **ADD** | Multiple missing actions | Complete coverage (see §6) |

### 5.2 Collection Point Changes

| Current Location | Issue | Recommendation |
|------------------|-------|----------------|
| `handle_filter_changed()` | No logging | Add `FilterChanged` event |
| `NavigateNext/Previous` | No context | Enrich with mode, filter state |
| `EnterEditor` | Minimal data | Add source image info |
| Video player state | Missing loop/speed | Add `VideoLoopToggled`, `VideoSpeedChanged` |

### 5.3 QA Process Changes

| Document | Change |
|----------|--------|
| `architect-checklist.md` | Add Section 5.2.1: Diagnostic Collection Point Validation |
| `story-dod-checklist.md` | Add Section 6.1: Diagnostic Instrumentation |
| `qa-gate-tmpl.yaml` | Add `instrumentation_audit` section |
| New task | Create `instrumentation-audit.md` |

---

## 6. Proposed Event Taxonomy Updates

### 6.1 New Events to Add

```rust
// Navigation Filter Events
pub enum AppStateEvent {
    // ... existing ...

    /// Emitted when any filter setting changes
    FilterChanged {
        filter_type: FilterChangeType,
        previous_active: bool,
        new_active: bool,
        filtered_count: usize,
        total_count: usize,
    },

    /// Emitted when filters are completely reset
    FilterCleared {
        had_media_type_filter: bool,
        had_date_filter: bool,
    },

    /// Emitted when navigation context changes
    NavigationContextChanged {
        from: NavigationContext,
        to: NavigationContext,
    },
}

pub enum FilterChangeType {
    MediaType { from: MediaTypeFilter, to: MediaTypeFilter },
    DateRangeEnabled,
    DateRangeDisabled,
    DateFieldChanged { field: DateFilterField },
    DateBoundSet { target: DateTarget },
    DateBoundCleared { target: DateTarget },
}

pub enum NavigationContext {
    Viewer,
    Editor,
}
```

### 6.2 Events to Modify

```rust
pub enum UserAction {
    // MODIFY: Add context to navigation
    NavigateNext {
        context: NavigationContext,
        filter_active: bool,
        position_in_filtered: Option<usize>,
        position_in_total: usize,
    },

    NavigatePrevious {
        context: NavigationContext,
        filter_active: bool,
        position_in_filtered: Option<usize>,
        position_in_total: usize,
    },

    // ... rest unchanged ...
}
```

### 6.3 Events Currently Defined but Never Logged

**Priority 1 - User-Visible Actions (Must Add):**
- `ApplyCrop`, `ApplyResize`, `ApplyDeblur`, `ApplyUpscale`
- `SaveImage`, `DeleteMedia`, `CaptureFrame`, `ExportFile`
- `SetVolume`, `ToggleMute`
- `ZoomIn`, `ZoomOut`, `ResetZoom`, `ToggleFitToWindow`
- `RotateClockwise`, `RotateCounterClockwise`
- `Undo`, `Redo`, `ReturnToViewer`

**Priority 2 - State Events (Should Add):**
- `VideoLoopToggled`, `VideoSpeedChanged`
- `EditorDeblurProgress`, `EditorDeblurCancelled`
- `ModelDownloadStarted`, `ModelDownloadCompleted`, `ModelDownloadFailed`

**Priority 3 - Operations (Nice to Have):**
- `ResizeImage`, `ApplyFilter`, `ExportFrame`, `LoadMetadata`

---

## 7. Collection Point Architecture

### 7.1 Recommended Collection Layer

```
┌─────────────────────────────────────────────────────────────────┐
│                 OPTIMAL COLLECTION POINTS                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   ┌─────────────────┐                                           │
│   │  UI Component   │  ← NO collection here                     │
│   └────────┬────────┘    (violates separation of concerns)      │
│            │ Message                                             │
│            ▼                                                     │
│   ┌─────────────────┐                                           │
│   │  App Handler    │  ← PRIMARY collection point               │
│   │  (update.rs)    │    • Full context available               │
│   └────────┬────────┘    • Single responsibility                │
│            │ Effect       • Consistent timestamp                 │
│            ▼                                                     │
│   ┌─────────────────┐                                           │
│   │  Domain Logic   │  ← SECONDARY collection point             │
│   │  (video_player) │    • State machine transitions            │
│   └────────┬────────┘    • Performance-critical ops             │
│            │                                                     │
│            ▼                                                     │
│   ┌─────────────────┐                                           │
│   │  Infrastructure │  ← TERTIARY collection point              │
│   │  (notifications)│    • Error/warning auto-capture           │
│   └─────────────────┘                                           │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 7.2 Collection Point Placement Rules

| Rule | Description | Rationale |
|------|-------------|-----------|
| **R1** | Collect at handler level, not UI | UI components should not know about diagnostics |
| **R2** | Collect at state transitions | Captures meaningful state changes |
| **R3** | One event per user intent | Avoid duplicate events from multiple sources |
| **R4** | Include sufficient context | Event should be self-describing |
| **R5** | Non-blocking always | Never impact UI responsiveness |

### 7.3 Specific Placement Recommendations

| Event Category | Recommended Location | File |
|----------------|---------------------|------|
| Filter changes | `handle_filter_changed()` | `src/app/update.rs` |
| Navigation | `handle_navigate_*()` | `src/app/update.rs` |
| Editor actions | `handle_image_editor_effect()` | `src/app/update.rs` |
| Video state | State transition methods | `src/video_player/state.rs` |
| Warnings/Errors | `NotificationManager::push()` | `src/ui/notifications/manager.rs` |

---

## 8. QA Process Enhancements

### 8.1 Architect Checklist Addition

**New Section 5.2.1: Diagnostic Collection Point Validation**

```markdown
### 5.2.1 Diagnostic Collection Point Validation

- [ ] All primary collection points documented in architecture
  - [ ] User action capture locations identified
  - [ ] State transition points identified
  - [ ] Error/warning capture mechanisms identified

- [ ] Collection point placement follows architectural principles
  - [ ] Collection at appropriate abstraction levels
  - [ ] No cross-layer direct logging
  - [ ] Event data colocated with event source
  - [ ] No duplicate collection points

- [ ] Instrumentation completeness documented
  - [ ] Critical user flows have collection coverage
  - [ ] State machine transitions have emission points
  - [ ] Async operations have start/completion tracking
  - [ ] Error paths have failure tracking

- [ ] Collection testing strategy defined
  - [ ] Integration tests verify event capture
  - [ ] Performance tests confirm acceptable overhead
  - [ ] Data quality validation (timestamps, categorization)
```

### 8.2 Story DoD Addition

**New Section 6.1: Diagnostic Instrumentation**

```markdown
## 6.1 Diagnostic Instrumentation (If story affects user-visible functionality)

- [ ] All new user actions have diagnostic instrumentation
- [ ] All state transitions properly instrumented
- [ ] Long-running operations have duration tracking
- [ ] Collection point placement validated
- [ ] No duplicate event logging
- [ ] Events are non-blocking
- [ ] No sensitive data in event details
```

### 8.3 QA Gate Template Addition

```yaml
instrumentation_audit:
  coverage_validated: true|false
  user_actions_checked:
    - action_name: "FilterChanged"
      handler: "handle_filter_changed"
      file: "src/app/update.rs:1600"
      test: "test_filter_change_logged"
  gaps_identified:
    - description: "Missing XYZ event"
      severity: "medium"
      deferred_to: "Story X.Y"
```

---

## 9. Implementation Roadmap

### 9.1 Proposed Epic Structure

**Epic 4: Diagnostics Collection Completeness**

| Story | Title | Priority | Effort |
|-------|-------|----------|--------|
| 4.1 | Add navigation filter diagnostic events | P1 | M |
| 4.2 | Add Viewer/Editor navigation context | P1 | S |
| 4.3 | Instrument missing user actions (Priority 1) | P1 | L |
| 4.4 | Instrument missing state events (Priority 2) | P2 | M |
| 4.5 | Update QA checklists for instrumentation validation | P1 | S |
| 4.6 | Audit existing collection points against new criteria | P2 | M |

### 9.2 Story Dependencies

```
4.5 (QA Updates) ─────────────────────────────────┐
                                                   │
4.1 (Filter Events) ──┬── 4.2 (Context) ──────────┼── 4.6 (Audit)
                      │                            │
4.3 (User Actions) ───┴── 4.4 (State Events) ─────┘
```

### 9.3 Acceptance Criteria Summary

**Story 4.1 - Filter Events:**
- [ ] `FilterChanged` event emitted for all filter modifications
- [ ] `FilterCleared` event emitted on reset
- [ ] Events include before/after state and counts
- [ ] Integration tests verify event capture
- [ ] No performance regression (< 1ms overhead)

**Story 4.2 - Navigation Context:**
- [ ] `NavigateNext/Previous` include context enum
- [ ] `NavigationContextChanged` emitted on mode switch
- [ ] Events include filter state and position info
- [ ] Editor navigation explicitly marked

**Story 4.5 - QA Updates:**
- [ ] Architect checklist section 5.2.1 added
- [ ] Story DoD section 6.1 added
- [ ] QA gate template updated
- [ ] Instrumentation audit task created

---

## Appendices

### Appendix A: Current Event Logging Locations

| File | Line | Event | Layer |
|------|------|-------|-------|
| `src/app/update.rs` | 947 | `UserAction::OpenSettings` | App Handler |
| `src/app/update.rs` | 952 | `UserAction::OpenHelp` | App Handler |
| `src/app/update.rs` | 957 | `UserAction::OpenAbout` | App Handler |
| `src/app/update.rs` | 962 | `UserAction::OpenDiagnostics` | App Handler |
| `src/app/update.rs` | 967 | `UserAction::EnterEditor` | App Handler |
| `src/app/update.rs` | 1274 | `UserAction::NavigateNext` | App Handler |
| `src/app/update.rs` | 1289 | `UserAction::NavigatePrevious` | App Handler |
| `src/app/update.rs` | 1525,1554 | `UserAction::LoadMedia` | App Handler |
| `src/app/update.rs` | 360 | `AppStateEvent::EditorOpened` | App Handler |
| `src/app/update.rs` | 389,699 | `AppStateEvent::EditorClosed` | App Handler |
| `src/app/update.rs` | 782 | `AppStateEvent::EditorDeblurStarted` | App Handler |
| `src/app/mod.rs` | 886,902 | `AppOperation::AIDeblurProcess` | App Handler |
| `src/app/mod.rs` | 892 | `AppStateEvent::EditorDeblurCompleted` | App Handler |
| `src/app/mod.rs` | 955,972 | `AppOperation::AIUpscaleProcess` | App Handler |
| `src/ui/viewer/component.rs` | 652 | `AppStateEvent::MediaLoadingStarted` | UI Component |
| `src/ui/viewer/component.rs` | 866 | `AppStateEvent::MediaLoaded` | UI Component |
| `src/ui/viewer/component.rs` | 946 | `AppStateEvent::MediaFailed` | UI Component |
| `src/ui/viewer/component.rs` | 1147 | `UserAction::TogglePlayback` | UI Component |
| `src/ui/viewer/component.rs` | 1197 | `UserAction::SeekVideo` | UI Component |
| `src/video_player/state.rs` | 236 | `AppOperation::VideoSeek` | Domain |
| `src/video_player/state.rs` | 322 | `AppStateEvent::VideoAtEndOfStream` | Domain |
| `src/video_player/state.rs` | 370 | `AppStateEvent::VideoPlaying` | Domain |
| `src/video_player/state.rs` | 411 | `AppStateEvent::VideoPaused` | Domain |
| `src/video_player/state.rs` | 495 | `AppStateEvent::VideoSeeking` | Domain |
| `src/video_player/state.rs` | 612 | `AppStateEvent::VideoBuffering` | Domain |
| `src/video_player/state.rs` | 620 | `AppStateEvent::VideoError` | Domain |
| `src/ui/notifications/manager.rs` | 62 | `WarningEvent` | Infrastructure |
| `src/ui/notifications/manager.rs` | 67 | `ErrorEvent` | Infrastructure |

### Appendix B: Filter Message Flow

```
FilterDropdown::Message::MediaTypeChanged(filter)
    │
    ▼
Navbar::Event::FilterChanged(msg)
    │
    ▼
Message::Viewer(component::Message::FilterDropdown(msg))
    │
    ▼
component::Effect::FilterChanged(msg)
    │
    ▼
App::handle_filter_changed(ctx, msg)  ← COLLECTION POINT
    │
    ├─► Clone current filter
    ├─► Apply change based on message
    ├─► MediaNavigator::set_filter(new_filter)
    └─► Persist if enabled
```

### Appendix C: Navigation Context Distinction

| Method | Used By | Filter Behavior | Should Track |
|--------|---------|-----------------|--------------|
| `peek_next_filtered()` | Viewer | Applies user filter | `context: Viewer` |
| `peek_previous_filtered()` | Viewer | Applies user filter | `context: Viewer` |
| `peek_next_image()` | Editor | Images only, no filter | `context: Editor` |
| `peek_previous_image()` | Editor | Images only, no filter | `context: Editor` |

---

## Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-01-15 | Winston (Architect) | Initial report |

---

*Report generated by Winston, Architect Agent*
*For questions or clarifications, consult with the Architecture team*
