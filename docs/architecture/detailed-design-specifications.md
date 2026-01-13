# Detailed Design Specifications

## Newtypes

Following the established pattern (see `src/video_player/frame_cache_size.rs`):

### BufferCapacity

```rust
// src/diagnostics/buffer.rs

use crate::config::defaults::{
    DEFAULT_DIAGNOSTICS_BUFFER_EVENTS,
    MIN_DIAGNOSTICS_BUFFER_EVENTS,
    MAX_DIAGNOSTICS_BUFFER_EVENTS,
};

/// Circular buffer capacity in number of events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufferCapacity(usize);

impl BufferCapacity {
    #[must_use]
    pub fn new(value: usize) -> Self {
        Self(value.clamp(MIN_DIAGNOSTICS_BUFFER_EVENTS, MAX_DIAGNOSTICS_BUFFER_EVENTS))
    }

    #[must_use]
    pub fn value(self) -> usize {
        self.0
    }
}

impl Default for BufferCapacity {
    fn default() -> Self {
        Self(DEFAULT_DIAGNOSTICS_BUFFER_EVENTS)
    }
}
```

### SamplingInterval

```rust
// src/diagnostics/resource_collector.rs

use std::time::Duration;

/// Resource sampling interval.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SamplingInterval(Duration);

impl SamplingInterval {
    pub const MIN: Duration = Duration::from_millis(100);
    pub const MAX: Duration = Duration::from_secs(10);
    pub const DEFAULT: Duration = Duration::from_secs(1);

    #[must_use]
    pub fn new(duration: Duration) -> Self {
        Self(duration.clamp(Self::MIN, Self::MAX))
    }

    #[must_use]
    pub fn value(self) -> Duration {
        self.0
    }
}

impl Default for SamplingInterval {
    fn default() -> Self {
        Self(Self::DEFAULT)
    }
}
```

## Message Integration

```rust
// src/app/message.rs (additions)

pub enum Message {
    // ... existing variants ...

    /// Diagnostics screen messages
    Diagnostics(diagnostics_screen::Message),

    /// Diagnostics collector status update (from subscription)
    DiagnosticsStatusUpdate(DiagnosticsStatus),

    /// Export completed
    DiagnosticsExportCompleted(Result<ExportResult, String>),
}
```

## Screen Integration

```rust
// src/app/screen.rs (modification)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Viewer,
    Settings,
    ImageEditor,
    Help,
    About,
    Diagnostics,  // NEW
}
```

## JSON Schema

```json
{
  "schema_version": "1.0",
  "metadata": {
    "report_id": "uuid-v4",
    "generated_at": "2026-01-13T14:30:00Z",
    "iced_lens_version": "0.6.0",
    "collection_started_at": "2026-01-13T14:25:00Z",
    "collection_duration_ms": 300000,
    "event_count": 847
  },
  "system_info": {
    "os": "Linux",
    "os_version": "6.14.0",
    "cpu_model": "AMD Ryzen 7 5800X",
    "cpu_cores": 8,
    "ram_total_mb": 32768,
    "disk_type": "SSD"
  },
  "events": [
    {
      "timestamp_ms": 0,
      "type": "resource_snapshot",
      "data": {
        "cpu_percent": 12.5,
        "ram_used_mb": 1024,
        "ram_total_mb": 32768,
        "disk_read_kb": 0,
        "disk_write_kb": 128
      }
    },
    {
      "timestamp_ms": 150,
      "type": "user_action",
      "data": {
        "action": "navigate_next",
        "details": null
      }
    },
    {
      "timestamp_ms": 200,
      "type": "app_state",
      "data": {
        "state": "media_loading_started",
        "context": "a1b2c3d4.jpg"
      }
    }
  ],
  "summary": {
    "event_counts": {
      "resource_snapshot": 300,
      "user_action": 45,
      "app_state": 120,
      "operation": 380,
      "warning": 2,
      "error": 0
    },
    "resource_stats": {
      "cpu_min": 2.1,
      "cpu_max": 85.3,
      "cpu_avg": 15.7,
      "ram_min_mb": 512,
      "ram_max_mb": 2048,
      "ram_avg_mb": 1024
    }
  }
}
```

## Anonymization Rules

| Data Type | Detection | Transformation |
|-----------|-----------|----------------|
| File paths | Contains `/` or `\` | Hash each segment, preserve extension |
| IPv4 | Regex `\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}` | Hash entire match |
| IPv6 | Regex `[0-9a-fA-F:]{2,39}` | Hash entire match |
| Domain | Regex `[a-zA-Z0-9-]+\.[a-zA-Z]{2,}` | Hash entire match |
| Username | Equals `whoami()` or common patterns | Hash entire match |

**Hash format:** First 8 characters of blake3 hash (e.g., `a1b2c3d4`)

---
