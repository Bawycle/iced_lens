# Data Models

## New Data Models

### DiagnosticEvent

**Purpose:** Represents a single diagnostic event captured by the collector.

**Integration:** Stored in circular buffer, serialized to JSON on export.

**Key Attributes:**
- `timestamp`: `Instant` - When the event occurred
- `event_type`: `DiagnosticEventType` - Discriminant for event kind
- `data`: Event-specific payload

**Enum Variants:**
```rust
pub enum DiagnosticEvent {
    ResourceSnapshot(ResourceMetrics),
    UserAction(UserActionData),
    AppState(AppStateData),
    Operation(OperationData),
    Warning(DiagnosticWarning),
    Error(DiagnosticError),
}
```

### ResourceMetrics

**Purpose:** System resource usage snapshot.

**Key Attributes:**
- `cpu_usage_percent`: `f32` - CPU usage percentage
- `memory_used_bytes`: `u64` - RAM currently used
- `memory_total_bytes`: `u64` - Total RAM available
- `disk_read_bytes`: `u64` - Disk read since last sample
- `disk_write_bytes`: `u64` - Disk write since last sample

### UserActionData

**Purpose:** User interaction event.

**Key Attributes:**
- `action`: `UserAction` - Action type enum
- `details`: `Option<String>` - Additional context (anonymized)

### AppStateData

**Purpose:** Application state transition.

**Key Attributes:**
- `state`: `AppStateType` - State enum
- `context`: `Option<String>` - Additional context (anonymized)

### DiagnosticReport

**Purpose:** Complete export-ready report structure.

**Key Attributes:**
- `metadata`: `ReportMetadata` - Report identification and timing
- `system_info`: `SystemInfo` - Hardware/OS information
- `events`: `Vec<AnonymizedEvent>` - All captured events (anonymized)
- `summary`: `ReportSummary` - Aggregate statistics

## Schema Integration Strategy

| Aspect | Approach |
|--------|----------|
| **New Tables** | N/A - in-memory only |
| **Modified Tables** | N/A |
| **New Indexes** | N/A |
| **Migration Strategy** | N/A |

**Backward Compatibility:**
- No persistent storage changes
- JSON schema versioned for future compatibility

---
