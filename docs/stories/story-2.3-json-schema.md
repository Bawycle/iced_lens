# Story 2.3: Report Summary Statistics

**Epic:** 2 - Anonymization & Export System
**Status:** Approved
**Priority:** High
**Estimate:** 1-2 hours
**Depends On:** Story 1.6

---

## Story

**As a** developer,
**I want** summary statistics included in diagnostic reports,
**So that** I can quickly understand patterns without parsing all events.

---

## Acceptance Criteria

1. `ReportSummary` struct implemented with event counts and resource statistics
2. `event_counts` field: count of events by type (user_action, state_change, etc.)
3. `resource_stats` field: min/max/avg for CPU and RAM from ResourceSnapshot events
4. `DiagnosticReport` extended with optional `summary` field
5. Summary computed automatically when building report via `build_report()`
6. Summary gracefully handles empty events (zero counts, None for stats)
7. Unit tests verify summary calculation correctness

**Note:** Report structure (metadata, system_info, events) already exists from Story 1.6. This story adds only the summary.

---

## Tasks

### Task 1: Create `ReportSummary` struct (AC: 1)
- [ ] Add struct to `src/diagnostics/report.rs`
- [ ] Fields: `event_counts`, `resource_stats`
- [ ] Derive `Serialize`, `Deserialize`, `Debug`, `Clone`, `PartialEq`

### Task 2: Create `ResourceStats` struct (AC: 3)
- [ ] Add struct to `src/diagnostics/report.rs`
- [ ] Fields: `cpu_min`, `cpu_max`, `cpu_avg`, `ram_min_mb`, `ram_max_mb`, `ram_avg_mb`
- [ ] All fields are `Option<f64>` (None if no ResourceSnapshot events)

### Task 3: Implement `EventCounts` type (AC: 2)
- [ ] Define as `HashMap<String, usize>` or dedicated struct
- [ ] Keys: "user_action", "state_change", "operation", "warning", "error", "resource_snapshot"
- [ ] Count events by `DiagnosticEventKind` variant

### Task 4: Add `summary` field to `DiagnosticReport` (AC: 4)
- [ ] Add `pub summary: Option<ReportSummary>` field
- [ ] Use `#[serde(skip_serializing_if = "Option::is_none")]`

### Task 5: Implement summary calculation (AC: 5, 6)
- [ ] Add `ReportSummary::from_events(events: &[SerializableEvent]) -> Self`
- [ ] Count events by type (iterate and match on `kind`)
- [ ] Calculate resource stats from `ResourceSnapshot` events
- [ ] Handle empty events: counts = 0, stats = None
- [ ] Call from `build_report()` in collector.rs

### Task 6: Write unit tests (AC: 7)
- [ ] Test empty events → zero counts, None stats
- [ ] Test mixed events → correct counts per type
- [ ] Test resource stats calculation (min/max/avg)
- [ ] Test JSON serialization includes summary

### Task 7: Run validation
- [ ] `cargo fmt --all`
- [ ] `cargo clippy --all --all-targets -- -D warnings`
- [ ] `cargo test`

### Task 8: Commit changes
- [ ] Stage all changes
- [ ] Commit: `feat(diagnostics): add report summary statistics [Story 2.3]`

---

## Dev Notes

### Source Tree

```
src/diagnostics/
├── mod.rs              # MODIFY: export ReportSummary, ResourceStats
├── report.rs           # MODIFY: add ReportSummary, ResourceStats, update DiagnosticReport
├── collector.rs        # MODIFY: update build_report() to compute summary
└── events.rs           # EXISTING: DiagnosticEventKind for matching
```

### Existing Code (Story 1.6)

`src/diagnostics/report.rs` already contains:
- `ReportMetadata` - report_id, generated_at, version, duration, event_count
- `SystemInfo` - os, os_version, cpu_cores, ram_total_mb
- `SerializableEvent` - timestamp_ms, kind (flattened)
- `DiagnosticReport` - metadata, system_info, events

### New Structures

```rust
/// Resource usage statistics from ResourceSnapshot events.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceStats {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_max: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_avg: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ram_min_mb: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ram_max_mb: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ram_avg_mb: Option<u64>,
}

/// Summary statistics for a diagnostic report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReportSummary {
    /// Count of events by type
    pub event_counts: HashMap<String, usize>,
    /// Resource usage statistics (if ResourceSnapshot events present)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_stats: Option<ResourceStats>,
}
```

### Summary Calculation Pattern

```rust
impl ReportSummary {
    pub fn from_events(events: &[SerializableEvent]) -> Self {
        let mut counts: HashMap<String, usize> = HashMap::new();
        let mut cpu_values: Vec<f64> = Vec::new();
        let mut ram_values: Vec<u64> = Vec::new();

        for event in events {
            // Count by event type
            let type_name = match &event.kind {
                DiagnosticEventKind::UserAction { .. } => "user_action",
                DiagnosticEventKind::StateChange { .. } => "state_change",
                DiagnosticEventKind::Operation { .. } => "operation",
                DiagnosticEventKind::Warning { .. } => "warning",
                DiagnosticEventKind::Error { .. } => "error",
                DiagnosticEventKind::ResourceSnapshot { metrics } => {
                    cpu_values.push(metrics.cpu_percent);
                    ram_values.push(metrics.ram_used_bytes / (1024 * 1024));
                    "resource_snapshot"
                }
            };
            *counts.entry(type_name.to_string()).or_insert(0) += 1;
        }

        let resource_stats = if cpu_values.is_empty() {
            None
        } else {
            Some(ResourceStats {
                cpu_min: Some(cpu_values.iter().cloned().fold(f64::INFINITY, f64::min)),
                cpu_max: Some(cpu_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max)),
                cpu_avg: Some(cpu_values.iter().sum::<f64>() / cpu_values.len() as f64),
                ram_min_mb: ram_values.iter().min().copied(),
                ram_max_mb: ram_values.iter().max().copied(),
                ram_avg_mb: Some(ram_values.iter().sum::<u64>() / ram_values.len() as u64),
            })
        };

        Self { event_counts: counts, resource_stats }
    }
}
```

### JSON Output Example

```json
{
  "metadata": { ... },
  "system_info": { ... },
  "events": [ ... ],
  "summary": {
    "event_counts": {
      "user_action": 15,
      "state_change": 8,
      "resource_snapshot": 120,
      "warning": 2,
      "error": 1
    },
    "resource_stats": {
      "cpu_min": 5.2,
      "cpu_max": 78.5,
      "cpu_avg": 23.4,
      "ram_min_mb": 1024,
      "ram_max_mb": 2048,
      "ram_avg_mb": 1536
    }
  }
}
```

---

## Testing

### Unit Tests

| Test | Input | Expected Output |
|------|-------|-----------------|
| `summary_empty_events` | `[]` | `event_counts: {}`, `resource_stats: None` |
| `summary_user_actions_only` | 3 UserAction events | `{"user_action": 3}`, `resource_stats: None` |
| `summary_mixed_events` | 2 UserAction, 1 Warning, 1 Error | Correct counts per type |
| `summary_resource_stats` | 3 ResourceSnapshot (cpu: 10, 20, 30) | `cpu_min: 10, cpu_max: 30, cpu_avg: 20` |
| `summary_serializes_correctly` | Report with summary | JSON contains "summary" object |
| `summary_skips_none_stats` | No ResourceSnapshot | JSON has no "resource_stats" key |

### Integration Tests

- Verify `export_json()` includes summary when events present
- Verify summary counts match `metadata.event_count`

---

## Dev Agent Record

### Agent Model Used
<!-- Record which AI model completed this story -->

### Completion Notes
<!-- Dev agent adds notes here during implementation -->

### Change Log
| Date | Change | Author |
|------|--------|--------|
| 2026-01-13 | Story created | PO |
| 2026-01-13 | PO Validation: Complete rewrite - removed redundant ACs, focused on ReportSummary, added Task-AC mapping, source tree, code examples | Sarah (PO) |

### File List
<!-- Files created or modified -->

---
