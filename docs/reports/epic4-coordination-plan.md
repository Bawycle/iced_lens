# Epic 4 Coordination Plan: Diagnostics Collection Completeness

**Document Type:** Inter-Agent Coordination Plan
**Created By:** Sarah (Product Owner)
**Date:** 2025-01-15
**Status:** In Progress
**Reference:** [Architecture Review Report](./architecture-review-diagnostics-navigation.md)

---

## Overview

This document coordinates the work between agents to address the diagnostics collection gaps identified by Winston (Architect) in his architecture review report. Each agent has specific tasks with clear instructions and must document their completion in this file.

### Execution Sequence

```
Phase 0: QA Process Updates (Prerequisite)
    ├── Step 0.1: Architect updates architect-checklist.md
    ├── Step 0.2: PO updates story-dod-checklist.md
    └── Step 0.3: QA updates qa-gate-tmpl.yaml

Phase 1: Initial Audit
    └── Step 1.1: QA audits existing collection points

Phase 2: Story Implementation (Dev executes stories)
    ├── Story 4.1: Filter Events
    ├── Story 4.2: Navigation Context
    ├── Story 4.3a: Editor Actions Instrumentation
    ├── Story 4.3b: Video/Audio Actions Instrumentation
    ├── Story 4.3c: View Controls Instrumentation
    └── Story 4.4: Missing State Events

Phase 3: Final Validation
    └── Step 3.1: QA performs conformance audit
```

---

# Phase 0: QA Process Updates

> **Goal:** Update all QA checklists and templates with the new diagnostic instrumentation validation criteria BEFORE any implementation work begins.

---

## Step 0.1: Update Architect Checklist

| Field | Value |
|-------|-------|
| **Agent** | Winston (Architect) |
| **Command** | `/architect` |
| **File to Modify** | `.bmad-core/checklists/architect-checklist.md` |

### Instructions for Winston

Add a new section **5.2.1 Diagnostic Collection Point Validation** to the architect checklist. Insert it after section 5.2 (Monitoring & Observability).

**Content to add:**

```markdown
### 5.2.1 Diagnostic Collection Point Validation

- [ ] All primary collection points documented in architecture
  - [ ] User action capture locations identified
  - [ ] State transition points identified
  - [ ] Error/warning capture mechanisms identified

- [ ] Collection point placement follows architectural principles
  - [ ] Collection at appropriate abstraction levels (handler level preferred)
  - [ ] No cross-layer direct logging (UI components should not log directly)
  - [ ] Event data colocated with event source
  - [ ] No duplicate collection points for same user intent

- [ ] Instrumentation completeness documented
  - [ ] Critical user flows have collection coverage defined
  - [ ] State machine transitions have emission points documented
  - [ ] Async operations have start/completion tracking points
  - [ ] Error paths have failure tracking points

- [ ] Collection testing strategy defined
  - [ ] Integration tests verify event capture at documented points
  - [ ] Performance tests confirm < 1ms overhead per event
  - [ ] Data quality validation (timestamps, categorization, context)
```

**Placement rules to encode (add as subsection or comments):**

| Rule | Description |
|------|-------------|
| R1 | Collect at handler level, not UI components |
| R2 | Collect at state transitions |
| R3 | One event per user intent (no duplicates) |
| R4 | Include sufficient context (self-describing events) |
| R5 | Non-blocking always (channel-based) |

### Completion Record

| Field | Value |
|-------|-------|
| **Completed By** | Winston (Architect) |
| **Date** | 2025-01-15 |
| **Commit/Branch** | feature/diagnostics (pending commit) |
| **Notes** | Added section 5.2.1 with LLM guidance block, placement rules table (R1-R5), and 4 validation categories: Documentation, Architectural Placement, Instrumentation Completeness, Testing Strategy. Total 16 new checklist items. |

---

## Step 0.2: Update Story Definition of Done

| Field | Value |
|-------|-------|
| **Agent** | Sarah (Product Owner) |
| **Command** | `/po` |
| **File to Modify** | `.bmad-core/checklists/story-dod-checklist.md` |

### Instructions for Sarah

Add a new section **6.1 Diagnostic Instrumentation** to the Story DoD checklist. This section applies conditionally when a story affects user-visible functionality.

**Content to add:**

```markdown
## 6.1 Diagnostic Instrumentation

> **Applicability:** Complete this section if the story adds, modifies, or removes user-visible functionality or system state transitions.

### User Action Coverage
- [ ] All new user actions have corresponding `UserAction` enum variants
- [ ] All new user action handlers call `log_action()` at handler level
- [ ] Action events include sufficient context (not just action name)

### State Event Coverage
- [ ] All new state transitions have corresponding `AppStateEvent` variants
- [ ] State events are emitted at transition points (not before/after)
- [ ] State events include before/after context where relevant

### Operation Tracking
- [ ] Long-running operations (>100ms) have duration tracking
- [ ] Async operations have start/completion event pairs
- [ ] Cancellable operations have cancellation events

### Quality Requirements
- [ ] Collection point placement validated (handler level, not UI)
- [ ] No duplicate event logging from multiple sources
- [ ] Events are non-blocking (channel-based)
- [ ] No sensitive data in event details (paths sanitized)
- [ ] Integration tests verify event capture
```

### Completion Record

| Field | Value |
|-------|-------|
| **Completed By** | Sarah (Product Owner) |
| **Date** | 2025-01-15 |
| **Commit/Branch** | feature/diagnostics (pending commit) |
| **Notes** | Added section 6.1 "Diagnostic Instrumentation (If Applicable)" with LLM guidance block and applicability note. Contains 4 sub-categories: User Action Coverage (3 items), State Event Coverage (3 items), Operation Tracking (3 items), Quality Requirements (5 items). Total 14 new checklist items. Section is conditional - applies only when story affects user-visible functionality or state transitions. |

---

## Step 0.3: Update QA Gate Template

| Field | Value |
|-------|-------|
| **Agent** | Quinn (QA) |
| **Command** | `/qa` |
| **File to Modify** | `.bmad-core/templates/qa-gate-tmpl.yaml` |

### Instructions for Quinn

Add an optional `instrumentation_audit` section to the QA gate template. This section should be completed when reviewing stories that affect diagnostics or user-visible functionality.

**Content to add:**

```yaml
# Optional section - complete when story affects diagnostics or user-visible functionality
instrumentation_audit:
  # Whether instrumentation coverage was validated for this story
  coverage_validated: true|false

  # List of user actions verified to have logging
  user_actions_checked:
    - action_name: "ExampleAction"
      handler: "handle_example"
      file: "src/app/update.rs:123"
      test: "test_example_action_logged"

  # List of state events verified
  state_events_checked:
    - event_name: "ExampleStateEvent"
      emission_point: "state_transition_method"
      file: "src/module/state.rs:456"
      test: "test_state_event_emitted"

  # Any gaps identified during review
  gaps_identified:
    - description: "Description of missing instrumentation"
      severity: "low|medium|high"
      deferred_to: "Story X.Y or N/A"

  # Collection point placement validation
  placement_validated:
    - checked: "No UI-level logging added"
    - checked: "Events emitted at handler level"
    - checked: "No duplicate events"
```

Also add guidance text explaining when this section should be completed:

```yaml
# INSTRUMENTATION AUDIT GUIDANCE
# Complete the instrumentation_audit section when:
# - Story adds new user-visible actions
# - Story modifies existing action handlers
# - Story adds state transitions or state machines
# - Story touches diagnostics module directly
# If none apply, set: instrumentation_audit: N/A
```

### Completion Record

| Field | Value |
|-------|-------|
| **Completed By** | Quinn (Test Architect) |
| **Date** | 2025-01-15 |
| **Commit/Branch** | feature/diagnostics (pending commit) |
| **Notes** | Added `instrumentation_audit` section to optional_fields_examples with: guidance comments (5 trigger conditions), coverage_validated flag, user_actions_checked list, state_events_checked list, gaps_identified list with severity, placement_validated checklist (3 items). Section is optional - use when story affects user-visible actions or state transitions. |

---

# Phase 1: Initial Audit

> **Goal:** Audit all existing diagnostic collection points against the new criteria to identify specific gaps and corrective actions needed.

---

## Step 1.1: Audit Existing Collection Points

| Field | Value |
|-------|-------|
| **Agent** | Quinn (QA) |
| **Command** | `/qa` |
| **Input** | Architecture review report (Appendix A: Current Event Logging Locations) |
| **Output** | Audit findings document |

### Instructions for Quinn

Perform a comprehensive audit of all existing diagnostic collection points using the new criteria from Step 0.1.

**Audit Scope:**

1. **Review all 28 current logging locations** (listed in Appendix A of Winston's report)

2. **For each collection point, validate:**
   - [ ] Placement is at appropriate architectural layer (handler preferred)
   - [ ] No duplicate events for same user intent
   - [ ] Sufficient context is captured
   - [ ] Non-blocking implementation

3. **Identify architectural violations:**
   - Collection points in UI components that should be in handlers
   - Missing context that should be added
   - Duplicate collection points

4. **Cross-reference with event taxonomy:**
   - List all `UserAction` variants that are defined but never logged
   - List all `AppStateEvent` variants that are defined but never logged
   - List all `AppOperation` variants that are defined but never logged

5. **Produce audit report with:**
   - Summary of findings
   - List of violations with severity
   - Recommended corrective actions for each violation
   - Input for story refinement (what each story should address)

**Reference Data from Winston's Report:**

Current logging locations to audit:
- `src/app/update.rs` (13 logging calls)
- `src/app/mod.rs` (5 logging calls)
- `src/ui/viewer/component.rs` (5 logging calls) ← **Potential violations here**
- `src/video_player/state.rs` (7 logging calls)
- `src/ui/notifications/manager.rs` (2 logging calls)

Events defined but never logged (from report section 6.3):
- Priority 1: 18 user actions
- Priority 2: 7 state events
- Priority 3: 4 operations

### Audit Report Location

Create the audit report at: `docs/qa/assessments/epic4-initial-audit.md`

### Completion Record

| Field | Value |
|-------|-------|
| **Completed By** | Quinn (Test Architect) |
| **Date** | 2025-01-15 |
| **Audit Report** | `docs/qa/assessments/epic4-initial-audit.md` |
| **Violations Found** | 5 architectural violations (UI-layer collection points in viewer/component.rs) |
| **Stories Impacted** | All stories (4.1, 4.2, 4.3a, 4.3b, 4.3c, 4.4) |
| **Notes** | **Coverage:** 42% (27/64 events logged). **Missing:** 25 UserActions, 7 AppStateEvents, 5 AppOperations. **Prerequisite:** Must relocate 5 UI-layer collection points to handler level before story implementation. Detailed corrective actions documented per story in audit report. |

---

# Phase 2: Story Implementation

> **Goal:** Implement all stories to achieve complete diagnostic collection coverage.

**Note:** Stories have been created based on audit findings. Story files are located in `docs/stories/`.

**Story 4.0 (Prerequisite):** Added based on audit recommendation to fix architectural violations before implementing new instrumentation.

---

## Story 4.0: Fix Collection Point Architectural Violations (Prerequisite)

| Field | Value |
|-------|-------|
| **Agent** | Dev |
| **Command** | `/dev` |
| **Priority** | P0 |
| **Effort** | S |
| **Depends On** | Phase 0, Phase 1 |
| **Story File** | `docs/stories/story-4.0-fix-collection-point-violations.md` |

### Story Summary

Move 5 UI-layer collection points to handler level to establish correct architectural pattern before adding new instrumentation.

### Implementation Record

| Field | Value |
|-------|-------|
| **Implemented By** | |
| **Date** | |
| **Branch** | |
| **QA Gate** | |
| **Notes** | |

---

## Story 4.1: Navigation Filter Diagnostic Events

| Field | Value |
|-------|-------|
| **Agent** | Dev |
| **Command** | `/dev` |
| **Priority** | P1 |
| **Effort** | M |
| **Depends On** | Story 4.0 |
| **Story File** | `docs/stories/story-4.1-navigation-filter-events.md` |

### Story Summary

Add diagnostic events for all navigation filter changes to enable analysis of filter usage patterns.

### Acceptance Criteria (Preliminary)

- [ ] `FilterChanged` event emitted for all filter modifications
- [ ] `FilterCleared` event emitted on reset
- [ ] Events include: filter_type, previous_active, new_active, filtered_count, total_count
- [ ] Collection point at `handle_filter_changed()` (src/app/update.rs)
- [ ] Integration tests verify event capture
- [ ] No performance regression (< 1ms overhead)

### Implementation Record

| Field | Value |
|-------|-------|
| **Implemented By** | |
| **Date** | |
| **Branch** | |
| **QA Gate** | |
| **Notes** | |

---

## Story 4.2: Viewer/Editor Navigation Context

| Field | Value |
|-------|-------|
| **Agent** | Dev |
| **Command** | `/dev` |
| **Priority** | P1 |
| **Effort** | S |
| **Depends On** | Story 4.0 |
| **Story File** | `docs/stories/story-4.2-navigation-context.md` |

### Story Summary

Enrich navigation events with explicit Viewer/Editor context to distinguish navigation patterns between modes.

### Acceptance Criteria (Preliminary)

- [ ] `NavigateNext/Previous` events include `context: NavigationContext` enum
- [ ] `NavigationContext` enum with `Viewer` and `Editor` variants
- [ ] Events include: filter_active, position_in_filtered, position_in_total
- [ ] Editor navigation explicitly marked (not inferred)
- [ ] Integration tests verify context is correctly captured

### Implementation Record

| Field | Value |
|-------|-------|
| **Implemented By** | |
| **Date** | |
| **Branch** | |
| **QA Gate** | |
| **Notes** | |

---

## Story 4.3a: Editor Actions Instrumentation

| Field | Value |
|-------|-------|
| **Agent** | Dev |
| **Command** | `/dev` |
| **Priority** | P1 |
| **Effort** | M |
| **Depends On** | Story 4.0 |
| **Story File** | `docs/stories/story-4.3a-editor-actions-instrumentation.md` |

### Story Summary

Instrument all editor-related user actions that are currently defined but not logged.

### Actions to Instrument

- `ApplyCrop`
- `ApplyResize`
- `ApplyDeblur`
- `ApplyUpscale`
- `SaveImage`
- `Undo`
- `Redo`
- `ReturnToViewer`

### Acceptance Criteria (Preliminary)

- [ ] All listed actions emit events when triggered
- [ ] Collection points at handler level (not in UI components)
- [ ] Events include relevant context (tool used, image dimensions, etc.)
- [ ] Integration tests verify each action is captured

### Implementation Record

| Field | Value |
|-------|-------|
| **Implemented By** | |
| **Date** | |
| **Branch** | |
| **QA Gate** | |
| **Notes** | |

---

## Story 4.3b: Video/Audio Actions Instrumentation

| Field | Value |
|-------|-------|
| **Agent** | Dev |
| **Command** | `/dev` |
| **Priority** | P1 |
| **Effort** | M |
| **Depends On** | Story 4.0 |
| **Story File** | `docs/stories/story-4.3b-video-audio-actions-instrumentation.md` |

### Story Summary

Instrument all video and audio control actions that are currently defined but not logged.

### Actions to Instrument

- `SetVolume`
- `ToggleMute`
- `CaptureFrame`
- `ExportFile`
- `DeleteMedia`

### Acceptance Criteria (Preliminary)

- [ ] All listed actions emit events when triggered
- [ ] `SetVolume` includes volume level
- [ ] Collection points at handler level
- [ ] Integration tests verify each action is captured

### Implementation Record

| Field | Value |
|-------|-------|
| **Implemented By** | |
| **Date** | |
| **Branch** | |
| **QA Gate** | |
| **Notes** | |

---

## Story 4.3c: View Controls Instrumentation

| Field | Value |
|-------|-------|
| **Agent** | Dev |
| **Command** | `/dev` |
| **Priority** | P1 |
| **Effort** | S |
| **Depends On** | Story 4.0 |
| **Story File** | `docs/stories/story-4.3c-view-controls-instrumentation.md` |

### Story Summary

Instrument all view control actions that are currently defined but not logged.

### Actions to Instrument

- `ZoomIn`
- `ZoomOut`
- `ResetZoom`
- `ToggleFitToWindow`
- `RotateClockwise`
- `RotateCounterClockwise`
- `ToggleFullscreen`
- `ExitFullscreen`

### Acceptance Criteria (Preliminary)

- [ ] All listed actions emit events when triggered
- [ ] Zoom events include current zoom level
- [ ] Rotation events include resulting angle
- [ ] Collection points at handler level
- [ ] Integration tests verify each action is captured

### Implementation Record

| Field | Value |
|-------|-------|
| **Implemented By** | |
| **Date** | |
| **Branch** | |
| **QA Gate** | |
| **Notes** | |

---

## Story 4.4: Missing State Events Instrumentation

| Field | Value |
|-------|-------|
| **Agent** | Dev |
| **Command** | `/dev` |
| **Priority** | P2 |
| **Effort** | M |
| **Depends On** | Stories 4.0-4.3 |
| **Story File** | `docs/stories/story-4.4-missing-state-events.md` |

### Story Summary

Instrument all state events that are currently defined but not logged.

### State Events to Instrument

**Video Playback:**
- `VideoLoopToggled { enabled: bool }`
- `VideoSpeedChanged { speed: f64 }`

**Editor:**
- `EditorDeblurProgress { percent: f32 }`
- `EditorDeblurCancelled`

**AI Model Lifecycle:**
- `ModelDownloadStarted { model: AIModel }`
- `ModelDownloadCompleted { model: AIModel }`
- `ModelDownloadFailed { model: AIModel, reason: String }`

### Acceptance Criteria (Preliminary)

- [ ] All listed state events emit when state changes occur
- [ ] Events emitted at state transition points
- [ ] AI model events cover download lifecycle
- [ ] Integration tests verify each state event

### Implementation Record

| Field | Value |
|-------|-------|
| **Implemented By** | |
| **Date** | |
| **Branch** | |
| **QA Gate** | |
| **Notes** | |

---

## Story 4.5: Final Conformance Audit

| Field | Value |
|-------|-------|
| **Agent** | QA |
| **Command** | `/qa` |
| **Priority** | P1 |
| **Effort** | S |
| **Depends On** | Stories 4.0-4.4 |
| **Story File** | `docs/stories/story-4.5-final-conformance-audit.md` |

### Story Summary

Perform final conformance audit validating 100% event coverage, architectural compliance, and quality requirements.

### Implementation Record

| Field | Value |
|-------|-------|
| **Auditor** | |
| **Date** | |
| **Final Report** | |
| **Coverage Achieved** | |
| **Epic Status** | |

---

# Phase 3: Final Validation

> **Goal:** Validate that all implementation work meets the diagnostic collection requirements.

---

## Step 3.1: Conformance Audit

| Field | Value |
|-------|-------|
| **Agent** | Quinn (QA) |
| **Command** | `/qa` |
| **Depends On** | All Phase 2 stories completed |
| **Output** | Final audit report |

### Instructions for Quinn

Perform a final conformance audit after all Phase 2 stories are implemented.

**Audit Checklist:**

1. **Coverage Validation**
   - [ ] All `UserAction` variants are now logged (100% coverage)
   - [ ] All `AppStateEvent` variants are now logged (100% coverage)
   - [ ] All `AppOperation` variants are now logged (100% coverage)

2. **Placement Validation**
   - [ ] No collection points in UI components (except justified exceptions)
   - [ ] All collection points at handler level or domain state machines
   - [ ] No duplicate events for same user intent

3. **Context Validation**
   - [ ] Navigation events include Viewer/Editor context
   - [ ] Filter events include before/after state
   - [ ] All events are self-describing (sufficient context)

4. **Quality Validation**
   - [ ] All events are non-blocking
   - [ ] No sensitive data in events
   - [ ] All events have integration tests

5. **Regression Check**
   - [ ] No existing functionality broken
   - [ ] Performance overhead < 1ms per event
   - [ ] All existing tests still pass

**Produce final report with:**
- Coverage metrics (before/after comparison)
- Any remaining gaps or technical debt
- Recommendations for future maintenance

### Audit Report Location

Create the final audit report at: `docs/qa/assessments/epic4-final-audit.md`

### Completion Record

| Field | Value |
|-------|-------|
| **Completed By** | |
| **Date** | |
| **Final Report** | |
| **Coverage Achieved** | |
| **Remaining Gaps** | |
| **Epic Status** | |
| **Notes** | |

---

# Quick Reference: Agent Routing

| Phase | Step | Agent | Command | What to Say |
|-------|------|-------|---------|-------------|
| 0 | 0.1 | Winston | `/architect` | "Execute Step 0.1 from epic4-coordination-plan.md" |
| 0 | 0.2 | Sarah | `/po` | "Execute Step 0.2 from epic4-coordination-plan.md" |
| 0 | 0.3 | Quinn | `/qa` | "Execute Step 0.3 from epic4-coordination-plan.md" |
| 1 | 1.1 | Quinn | `/qa` | "Execute Step 1.1 from epic4-coordination-plan.md" |
| 2 | Stories | Dev | `/dev` | "Implement Story 4.X from epic4-coordination-plan.md" |
| 3 | 3.1 | Quinn | `/qa` | "Execute Step 3.1 from epic4-coordination-plan.md" |

---

# Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-01-15 | Sarah (PO) | Initial coordination plan |
| 1.1 | 2025-01-15 | Sarah (PO) | Created Epic 4 + 8 story files, added Story 4.0 prerequisite, updated story file references |

---

*Coordination plan created by Sarah, Product Owner*
*Each agent must update their Completion Record section when done*
