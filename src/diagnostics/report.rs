// SPDX-License-Identifier: MPL-2.0
//! Diagnostic report generation and JSON export.
//!
//! This module provides structures for building diagnostic reports
//! that can be exported as JSON for debugging and analysis.

use std::time::Instant;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sysinfo::System;
use uuid::Uuid;

use super::DiagnosticEventKind;

// =============================================================================
// Report Metadata
// =============================================================================

/// Metadata about a diagnostic report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReportMetadata {
    /// Unique identifier for this report (UUID v4)
    pub report_id: String,
    /// When the report was generated (ISO 8601)
    pub generated_at: String,
    /// Version of `IcedLens` that generated the report
    pub iced_lens_version: String,
    /// When diagnostic collection started (ISO 8601)
    pub collection_started_at: String,
    /// Duration of collection in milliseconds
    pub collection_duration_ms: u64,
    /// Total number of events in the report
    pub event_count: usize,
}

impl ReportMetadata {
    /// Creates new report metadata.
    #[must_use]
    pub fn new(
        collection_started_at: DateTime<Utc>,
        collection_duration_ms: u64,
        event_count: usize,
    ) -> Self {
        Self {
            report_id: Uuid::new_v4().to_string(),
            generated_at: Utc::now().to_rfc3339(),
            iced_lens_version: env!("CARGO_PKG_VERSION").to_string(),
            collection_started_at: collection_started_at.to_rfc3339(),
            collection_duration_ms,
            event_count,
        }
    }
}

// =============================================================================
// System Information
// =============================================================================

/// System information for diagnostic context.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemInfo {
    /// Operating system name (e.g., "linux", "windows", "macos")
    pub os: String,
    /// Operating system version
    pub os_version: String,
    /// Number of CPU cores
    pub cpu_cores: usize,
    /// Total RAM in megabytes
    pub ram_total_mb: u64,
}

impl SystemInfo {
    /// Collects current system information.
    #[must_use]
    pub fn collect() -> Self {
        let sys = System::new_all();

        Self {
            os: std::env::consts::OS.to_string(),
            os_version: System::os_version().unwrap_or_else(|| "unknown".to_string()),
            cpu_cores: sys.cpus().len(),
            ram_total_mb: sys.total_memory() / (1024 * 1024),
        }
    }
}

// =============================================================================
// Serializable Event
// =============================================================================

/// A diagnostic event that can be serialized to JSON.
///
/// This wrapper converts `DiagnosticEvent` timestamps (which use `Instant`)
/// to relative milliseconds since collection started.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SerializableEvent {
    /// Milliseconds since collection started
    pub timestamp_ms: u64,
    /// The event data
    #[serde(flatten)]
    pub kind: DiagnosticEventKind,
}

impl SerializableEvent {
    /// Creates a serializable event from a diagnostic event.
    ///
    /// # Arguments
    ///
    /// * `event_timestamp` - The event's `Instant` timestamp
    /// * `collection_start` - When collection started (for relative calculation)
    /// * `kind` - The event data
    #[must_use]
    #[allow(clippy::cast_possible_truncation)] // Duration in ms fits comfortably in u64
    pub fn new(
        event_timestamp: Instant,
        collection_start: Instant,
        kind: DiagnosticEventKind,
    ) -> Self {
        let timestamp_ms = event_timestamp.duration_since(collection_start).as_millis() as u64;

        Self { timestamp_ms, kind }
    }
}

// =============================================================================
// Diagnostic Report
// =============================================================================

/// A complete diagnostic report ready for JSON export.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiagnosticReport {
    /// Report metadata
    pub metadata: ReportMetadata,
    /// System information
    pub system_info: SystemInfo,
    /// Collected events
    pub events: Vec<SerializableEvent>,
}

impl DiagnosticReport {
    /// Creates a new diagnostic report.
    #[must_use]
    pub fn new(
        metadata: ReportMetadata,
        system_info: SystemInfo,
        events: Vec<SerializableEvent>,
    ) -> Self {
        Self {
            metadata,
            system_info,
            events,
        }
    }

    /// Exports the report as pretty-printed JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if JSON serialization fails.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::{ResourceMetrics, UserAction};

    // =========================================================================
    // ReportMetadata Tests
    // =========================================================================

    #[test]
    fn report_metadata_new_creates_valid_metadata() {
        let start = Utc::now();
        let metadata = ReportMetadata::new(start, 5000, 10);

        assert!(!metadata.report_id.is_empty());
        assert!(!metadata.generated_at.is_empty());
        assert_eq!(metadata.iced_lens_version, env!("CARGO_PKG_VERSION"));
        assert_eq!(metadata.collection_duration_ms, 5000);
        assert_eq!(metadata.event_count, 10);
    }

    #[test]
    fn report_metadata_serializes_to_json() {
        let start = Utc::now();
        let metadata = ReportMetadata::new(start, 1000, 5);

        let json = serde_json::to_string(&metadata).expect("serialization should succeed");

        assert!(json.contains("\"report_id\""));
        assert!(json.contains("\"generated_at\""));
        assert!(json.contains("\"iced_lens_version\""));
        assert!(json.contains("\"collection_duration_ms\":1000"));
        assert!(json.contains("\"event_count\":5"));
    }

    #[test]
    fn report_metadata_report_id_is_unique() {
        let start = Utc::now();
        let meta1 = ReportMetadata::new(start, 1000, 5);
        let meta2 = ReportMetadata::new(start, 1000, 5);

        assert_ne!(meta1.report_id, meta2.report_id);
    }

    // =========================================================================
    // SystemInfo Tests
    // =========================================================================

    #[test]
    fn system_info_collect_returns_valid_data() {
        let info = SystemInfo::collect();

        assert!(!info.os.is_empty());
        assert!(info.cpu_cores > 0);
        assert!(info.ram_total_mb > 0);
    }

    #[test]
    fn system_info_serializes_to_json() {
        let info = SystemInfo::collect();

        let json = serde_json::to_string(&info).expect("serialization should succeed");

        assert!(json.contains("\"os\""));
        assert!(json.contains("\"os_version\""));
        assert!(json.contains("\"cpu_cores\""));
        assert!(json.contains("\"ram_total_mb\""));
    }

    // =========================================================================
    // SerializableEvent Tests
    // =========================================================================

    #[test]
    fn serializable_event_calculates_relative_timestamp() {
        let start = Instant::now();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let event_time = Instant::now();

        let event = SerializableEvent::new(
            event_time,
            start,
            DiagnosticEventKind::UserAction {
                action: UserAction::NavigateNext,
                details: None,
            },
        );

        // Should be at least 10ms
        assert!(event.timestamp_ms >= 10);
    }

    #[test]
    fn serializable_event_serializes_with_flattened_kind() {
        let start = Instant::now();
        let event = SerializableEvent::new(
            start,
            start,
            DiagnosticEventKind::UserAction {
                action: UserAction::TogglePlayback,
                details: None,
            },
        );

        let json = serde_json::to_string(&event).expect("serialization should succeed");

        assert!(json.contains("\"timestamp_ms\":0"));
        assert!(json.contains("\"type\":\"user_action\""));
        assert!(json.contains("\"action\":\"toggle_playback\""));
    }

    #[test]
    fn serializable_event_serializes_resource_snapshot() {
        let start = Instant::now();
        let metrics = ResourceMetrics::new(50.0, 4_000_000_000, 8_000_000_000, 1000, 2000);
        let event = SerializableEvent::new(
            start,
            start,
            DiagnosticEventKind::ResourceSnapshot { metrics },
        );

        let json = serde_json::to_string(&event).expect("serialization should succeed");

        assert!(json.contains("\"type\":\"resource_snapshot\""));
        assert!(json.contains("\"cpu_percent\":50.0"));
    }

    // =========================================================================
    // DiagnosticReport Tests
    // =========================================================================

    #[test]
    fn diagnostic_report_to_json_produces_valid_json() {
        let start = Utc::now();
        let metadata = ReportMetadata::new(start, 1000, 1);
        let system_info = SystemInfo::collect();

        let instant_start = Instant::now();
        let events = vec![SerializableEvent::new(
            instant_start,
            instant_start,
            DiagnosticEventKind::UserAction {
                action: UserAction::NavigateNext,
                details: None,
            },
        )];

        let report = DiagnosticReport::new(metadata, system_info, events);
        let json = report.to_json().expect("JSON export should succeed");

        // Verify it's valid JSON by parsing it back
        let parsed: serde_json::Value =
            serde_json::from_str(&json).expect("JSON should be parseable");

        assert!(parsed.get("metadata").is_some());
        assert!(parsed.get("system_info").is_some());
        assert!(parsed.get("events").is_some());
    }

    #[test]
    fn diagnostic_report_with_empty_events() {
        let start = Utc::now();
        let metadata = ReportMetadata::new(start, 0, 0);
        let system_info = SystemInfo::collect();

        let report = DiagnosticReport::new(metadata, system_info, vec![]);
        let json = report.to_json().expect("JSON export should succeed");

        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let events = parsed.get("events").unwrap().as_array().unwrap();

        assert!(events.is_empty());
    }
}
