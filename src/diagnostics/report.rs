// SPDX-License-Identifier: MPL-2.0
//! Diagnostic report generation and JSON export.
//!
//! This module provides structures for building diagnostic reports
//! that can be exported as JSON for debugging and analysis.

use std::collections::HashMap;
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
// Disk Type
// =============================================================================

/// Type of storage disk.
///
/// Used to identify whether the system's primary disk is an SSD, HDD, or unknown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DiskType {
    /// Solid State Drive (fast, no moving parts)
    Ssd,
    /// Hard Disk Drive (spinning platters)
    Hdd,
    /// Unknown or undetectable disk type (default)
    #[default]
    Unknown,
}

impl From<sysinfo::DiskKind> for DiskType {
    fn from(kind: sysinfo::DiskKind) -> Self {
        match kind {
            sysinfo::DiskKind::SSD => Self::Ssd,
            sysinfo::DiskKind::HDD => Self::Hdd,
            sysinfo::DiskKind::Unknown(_) => Self::Unknown,
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
    /// Operating system distribution name (e.g., "Linux Mint", "Windows 11", "macOS")
    pub os_name: String,
    /// Operating system version
    pub os_version: String,
    /// Kernel version (e.g., "6.14.0-37-generic")
    pub kernel_version: String,
    /// CPU architecture (e.g., "`x86_64`", "aarch64")
    pub cpu_arch: String,
    /// CPU brand name (e.g., "Intel Core i7-9700K", "Apple M1")
    pub cpu_brand: String,
    /// Number of CPU cores
    pub cpu_cores: usize,
    /// Total RAM in megabytes
    pub ram_total_mb: u64,
    /// Type of primary disk (SSD, HDD, or Unknown)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk_type: Option<DiskType>,
}

impl SystemInfo {
    /// Collects current system information.
    #[must_use]
    pub fn collect() -> Self {
        let sys = System::new_all();

        // Get CPU brand from first CPU
        let cpu_brand = sys
            .cpus()
            .first()
            .map_or_else(|| "unknown".to_string(), |cpu| cpu.brand().to_string());

        // Detect disk type for the disk containing the home directory
        let disk_type = Self::detect_home_disk_type();

        Self {
            os: std::env::consts::OS.to_string(),
            os_name: System::name().unwrap_or_else(|| "unknown".to_string()),
            os_version: System::os_version().unwrap_or_else(|| "unknown".to_string()),
            kernel_version: System::kernel_version().unwrap_or_else(|| "unknown".to_string()),
            cpu_arch: std::env::consts::ARCH.to_string(),
            cpu_brand,
            cpu_cores: sys.cpus().len(),
            ram_total_mb: sys.total_memory() / (1024 * 1024),
            disk_type,
        }
    }

    /// Detects the disk type for the disk containing the user's home directory.
    fn detect_home_disk_type() -> Option<DiskType> {
        use sysinfo::Disks;

        let home_dir = dirs::home_dir()?;
        let home_path = home_dir.to_string_lossy();

        let disks = Disks::new_with_refreshed_list();

        // Find the disk with the longest mount point prefix that matches the home directory
        let mut best_match: Option<(&sysinfo::Disk, usize)> = None;

        for disk in &disks {
            let mount_point = disk.mount_point().to_string_lossy();
            if home_path.starts_with(mount_point.as_ref()) {
                let len = mount_point.len();
                if best_match.is_none_or(|(_, best_len)| len > best_len) {
                    best_match = Some((disk, len));
                }
            }
        }

        best_match.map(|(disk, _)| DiskType::from(disk.kind()))
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
// Report Summary
// =============================================================================

/// Resource usage statistics calculated from `ResourceSnapshot` events.
///
/// All fields are optional since there may be no resource snapshots in a report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceStats {
    /// Minimum CPU usage percentage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_min: Option<f32>,
    /// Maximum CPU usage percentage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_max: Option<f32>,
    /// Average CPU usage percentage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_avg: Option<f32>,
    /// Minimum RAM usage in megabytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ram_min_mb: Option<u64>,
    /// Maximum RAM usage in megabytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ram_max_mb: Option<u64>,
    /// Average RAM usage in megabytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ram_avg_mb: Option<u64>,
}

/// Summary statistics for a diagnostic report.
///
/// Provides quick overview of collected events without parsing all event data.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReportSummary {
    /// Count of events by type (e.g., `"user_action": 15`, `"resource_snapshot": 120`)
    pub event_counts: HashMap<String, usize>,
    /// Resource usage statistics (present only if `ResourceSnapshot` events exist)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_stats: Option<ResourceStats>,
}

impl ReportSummary {
    /// Computes summary statistics from a list of serializable events.
    ///
    /// # Behavior
    ///
    /// - Counts events by their `DiagnosticEventKind` variant
    /// - Calculates min/max/avg CPU and RAM from `ResourceSnapshot` events
    /// - Returns empty counts and `None` stats for empty event list
    #[must_use]
    #[allow(clippy::cast_precision_loss)] // Length of events fits in f32
    pub fn from_events(events: &[SerializableEvent]) -> Self {
        let mut counts: HashMap<String, usize> = HashMap::new();
        let mut cpu_values: Vec<f32> = Vec::new();
        let mut ram_values: Vec<u64> = Vec::new();

        for event in events {
            let type_name = match &event.kind {
                DiagnosticEventKind::UserAction { .. } => "user_action",
                DiagnosticEventKind::AppState { .. } => "app_state",
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
                cpu_min: cpu_values.iter().copied().reduce(f32::min),
                cpu_max: cpu_values.iter().copied().reduce(f32::max),
                cpu_avg: Some(cpu_values.iter().sum::<f32>() / cpu_values.len() as f32),
                ram_min_mb: ram_values.iter().min().copied(),
                ram_max_mb: ram_values.iter().max().copied(),
                ram_avg_mb: Some(ram_values.iter().sum::<u64>() / ram_values.len() as u64),
            })
        };

        Self {
            event_counts: counts,
            resource_stats,
        }
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
    /// Summary statistics (computed from events)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<ReportSummary>,
}

impl DiagnosticReport {
    /// Creates a new diagnostic report with summary computed automatically.
    #[must_use]
    pub fn new(
        metadata: ReportMetadata,
        system_info: SystemInfo,
        events: Vec<SerializableEvent>,
    ) -> Self {
        let summary = Some(ReportSummary::from_events(&events));
        Self {
            metadata,
            system_info,
            events,
            summary,
        }
    }

    /// Creates a new diagnostic report without computing a summary.
    ///
    /// Useful when deserializing an existing report.
    #[must_use]
    pub fn without_summary(
        metadata: ReportMetadata,
        system_info: SystemInfo,
        events: Vec<SerializableEvent>,
    ) -> Self {
        Self {
            metadata,
            system_info,
            events,
            summary: None,
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
    use crate::diagnostics::{NavigationContext, ResourceMetrics, UserAction};

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
    // DiskType Tests
    // =========================================================================

    #[test]
    fn disk_type_from_sysinfo_ssd() {
        let disk_type: DiskType = sysinfo::DiskKind::SSD.into();
        assert_eq!(disk_type, DiskType::Ssd);
    }

    #[test]
    fn disk_type_from_sysinfo_hdd() {
        let disk_type: DiskType = sysinfo::DiskKind::HDD.into();
        assert_eq!(disk_type, DiskType::Hdd);
    }

    #[test]
    fn disk_type_from_sysinfo_unknown() {
        let disk_type: DiskType = sysinfo::DiskKind::Unknown(-1).into();
        assert_eq!(disk_type, DiskType::Unknown);
    }

    #[test]
    fn disk_type_serializes_snake_case() {
        assert_eq!(serde_json::to_string(&DiskType::Ssd).unwrap(), "\"ssd\"");
        assert_eq!(serde_json::to_string(&DiskType::Hdd).unwrap(), "\"hdd\"");
        assert_eq!(
            serde_json::to_string(&DiskType::Unknown).unwrap(),
            "\"unknown\""
        );
    }

    #[test]
    fn disk_type_deserializes_from_json() {
        let ssd: DiskType = serde_json::from_str("\"ssd\"").unwrap();
        assert_eq!(ssd, DiskType::Ssd);

        let hdd: DiskType = serde_json::from_str("\"hdd\"").unwrap();
        assert_eq!(hdd, DiskType::Hdd);

        let unknown: DiskType = serde_json::from_str("\"unknown\"").unwrap();
        assert_eq!(unknown, DiskType::Unknown);
    }

    #[test]
    fn disk_type_default_is_unknown() {
        assert_eq!(DiskType::default(), DiskType::Unknown);
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
    fn system_info_collect_returns_os_name() {
        let info = SystemInfo::collect();

        // os_name should be non-empty (even if it's "unknown")
        assert!(!info.os_name.is_empty());
    }

    #[test]
    fn system_info_collect_returns_kernel_version() {
        let info = SystemInfo::collect();

        // kernel_version should be non-empty (even if it's "unknown")
        assert!(!info.kernel_version.is_empty());
    }

    #[test]
    fn system_info_includes_cpu_arch() {
        let info = SystemInfo::collect();

        assert!(!info.cpu_arch.is_empty());
        // Should be one of the common architectures
        let valid_archs = ["x86_64", "aarch64", "x86", "arm", "riscv64"];
        assert!(
            valid_archs.iter().any(|&arch| info.cpu_arch.contains(arch)),
            "cpu_arch '{}' should be a recognized architecture",
            info.cpu_arch
        );
    }

    #[test]
    fn system_info_includes_cpu_brand() {
        let info = SystemInfo::collect();

        // cpu_brand should not be empty (even if it's "unknown")
        assert!(!info.cpu_brand.is_empty());
    }

    #[test]
    fn system_info_includes_disk_type() {
        let info = SystemInfo::collect();

        // disk_type might be None on some systems (e.g., VMs, unusual setups)
        // If it is Some, it should be a valid DiskType
        if let Some(disk_type) = info.disk_type {
            // Just verify it's one of the valid variants
            assert!(matches!(
                disk_type,
                DiskType::Ssd | DiskType::Hdd | DiskType::Unknown
            ));
        }
    }

    #[test]
    fn system_info_serializes_to_json() {
        let info = SystemInfo::collect();

        let json = serde_json::to_string(&info).expect("serialization should succeed");

        assert!(json.contains("\"os\""));
        assert!(json.contains("\"os_name\""));
        assert!(json.contains("\"os_version\""));
        assert!(json.contains("\"kernel_version\""));
        assert!(json.contains("\"cpu_arch\""));
        assert!(json.contains("\"cpu_brand\""));
        assert!(json.contains("\"cpu_cores\""));
        assert!(json.contains("\"ram_total_mb\""));
        // disk_type is optional, so it might not be present
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
                action: UserAction::NavigateNext {
                    context: NavigationContext::Viewer,
                    filter_active: false,
                    position_in_filtered: None,
                    position_in_total: 0,
                },
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
                action: UserAction::NavigateNext {
                    context: NavigationContext::Viewer,
                    filter_active: false,
                    position_in_filtered: None,
                    position_in_total: 0,
                },
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

    // =========================================================================
    // ReportSummary Tests
    // =========================================================================

    #[test]
    fn summary_empty_events_has_empty_counts_and_no_stats() {
        let summary = ReportSummary::from_events(&[]);

        assert!(summary.event_counts.is_empty());
        assert!(summary.resource_stats.is_none());
    }

    #[test]
    fn summary_user_actions_only() {
        let start = Instant::now();
        let events = vec![
            SerializableEvent::new(
                start,
                start,
                DiagnosticEventKind::UserAction {
                    action: UserAction::NavigateNext {
                        context: NavigationContext::Viewer,
                        filter_active: false,
                        position_in_filtered: None,
                        position_in_total: 0,
                    },
                    details: None,
                },
            ),
            SerializableEvent::new(
                start,
                start,
                DiagnosticEventKind::UserAction {
                    action: UserAction::TogglePlayback,
                    details: None,
                },
            ),
            SerializableEvent::new(
                start,
                start,
                DiagnosticEventKind::UserAction {
                    action: UserAction::ZoomIn,
                    details: None,
                },
            ),
        ];

        let summary = ReportSummary::from_events(&events);

        assert_eq!(summary.event_counts.get("user_action"), Some(&3));
        assert_eq!(summary.event_counts.len(), 1);
        assert!(summary.resource_stats.is_none());
    }

    #[test]
    fn summary_mixed_events() {
        let start = Instant::now();
        let events = vec![
            SerializableEvent::new(
                start,
                start,
                DiagnosticEventKind::UserAction {
                    action: UserAction::NavigateNext {
                        context: NavigationContext::Viewer,
                        filter_active: false,
                        position_in_filtered: None,
                        position_in_total: 0,
                    },
                    details: None,
                },
            ),
            SerializableEvent::new(
                start,
                start,
                DiagnosticEventKind::UserAction {
                    action: UserAction::TogglePlayback,
                    details: None,
                },
            ),
            SerializableEvent::new(
                start,
                start,
                DiagnosticEventKind::Warning {
                    event: crate::diagnostics::WarningEvent {
                        warning_type: crate::diagnostics::WarningType::FileNotFound,
                        message: "test".to_string(),
                        source_module: None,
                    },
                },
            ),
            SerializableEvent::new(
                start,
                start,
                DiagnosticEventKind::Error {
                    event: crate::diagnostics::ErrorEvent {
                        error_type: crate::diagnostics::ErrorType::IoError,
                        error_code: None,
                        message: "test".to_string(),
                        source_module: None,
                    },
                },
            ),
        ];

        let summary = ReportSummary::from_events(&events);

        assert_eq!(summary.event_counts.get("user_action"), Some(&2));
        assert_eq!(summary.event_counts.get("warning"), Some(&1));
        assert_eq!(summary.event_counts.get("error"), Some(&1));
        assert_eq!(summary.event_counts.len(), 3);
        assert!(summary.resource_stats.is_none());
    }

    #[test]
    fn summary_resource_stats_calculated() {
        let start = Instant::now();
        let events = vec![
            SerializableEvent::new(
                start,
                start,
                DiagnosticEventKind::ResourceSnapshot {
                    metrics: ResourceMetrics::new(
                        10.0,
                        1024 * 1024 * 1024, // 1 GB
                        8 * 1024 * 1024 * 1024,
                        0,
                        0,
                    ),
                },
            ),
            SerializableEvent::new(
                start,
                start,
                DiagnosticEventKind::ResourceSnapshot {
                    metrics: ResourceMetrics::new(
                        20.0,
                        2 * 1024 * 1024 * 1024, // 2 GB
                        8 * 1024 * 1024 * 1024,
                        0,
                        0,
                    ),
                },
            ),
            SerializableEvent::new(
                start,
                start,
                DiagnosticEventKind::ResourceSnapshot {
                    metrics: ResourceMetrics::new(
                        30.0,
                        3 * 1024 * 1024 * 1024, // 3 GB
                        8 * 1024 * 1024 * 1024,
                        0,
                        0,
                    ),
                },
            ),
        ];

        let summary = ReportSummary::from_events(&events);

        assert_eq!(summary.event_counts.get("resource_snapshot"), Some(&3));

        let stats = summary.resource_stats.expect("should have resource stats");
        assert_eq!(stats.cpu_min, Some(10.0));
        assert_eq!(stats.cpu_max, Some(30.0));
        assert_eq!(stats.cpu_avg, Some(20.0));
        assert_eq!(stats.ram_min_mb, Some(1024));
        assert_eq!(stats.ram_max_mb, Some(3072));
        assert_eq!(stats.ram_avg_mb, Some(2048));
    }

    #[test]
    fn summary_serializes_to_json() {
        let start = Instant::now();
        let events = vec![SerializableEvent::new(
            start,
            start,
            DiagnosticEventKind::ResourceSnapshot {
                metrics: ResourceMetrics::new(
                    50.0,
                    2 * 1024 * 1024 * 1024,
                    8 * 1024 * 1024 * 1024,
                    0,
                    0,
                ),
            },
        )];

        let summary = ReportSummary::from_events(&events);
        let json = serde_json::to_string(&summary).expect("serialization should succeed");

        assert!(json.contains("\"event_counts\""));
        assert!(json.contains("\"resource_snapshot\":1"));
        assert!(json.contains("\"resource_stats\""));
        assert!(json.contains("\"cpu_min\":50.0"));
        assert!(json.contains("\"cpu_max\":50.0"));
        assert!(json.contains("\"cpu_avg\":50.0"));
    }

    #[test]
    fn summary_skips_none_resource_stats_in_json() {
        let start = Instant::now();
        let events = vec![SerializableEvent::new(
            start,
            start,
            DiagnosticEventKind::UserAction {
                action: UserAction::NavigateNext {
                    context: NavigationContext::Viewer,
                    filter_active: false,
                    position_in_filtered: None,
                    position_in_total: 0,
                },
                details: None,
            },
        )];

        let summary = ReportSummary::from_events(&events);
        let json = serde_json::to_string(&summary).expect("serialization should succeed");

        // resource_stats should be omitted when None
        assert!(!json.contains("\"resource_stats\""));
    }

    #[test]
    fn diagnostic_report_includes_summary() {
        let start = Utc::now();
        let metadata = ReportMetadata::new(start, 1000, 2);
        let system_info = SystemInfo::collect();

        let instant_start = Instant::now();
        let events = vec![
            SerializableEvent::new(
                instant_start,
                instant_start,
                DiagnosticEventKind::UserAction {
                    action: UserAction::NavigateNext {
                        context: NavigationContext::Viewer,
                        filter_active: false,
                        position_in_filtered: None,
                        position_in_total: 0,
                    },
                    details: None,
                },
            ),
            SerializableEvent::new(
                instant_start,
                instant_start,
                DiagnosticEventKind::ResourceSnapshot {
                    metrics: ResourceMetrics::new(
                        25.0,
                        1024 * 1024 * 1024,
                        8 * 1024 * 1024 * 1024,
                        0,
                        0,
                    ),
                },
            ),
        ];

        let report = DiagnosticReport::new(metadata, system_info, events);
        let json = report.to_json().expect("JSON export should succeed");

        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(parsed.get("summary").is_some());
        let summary = parsed.get("summary").unwrap();
        assert!(summary.get("event_counts").is_some());
        assert!(summary.get("resource_stats").is_some());
    }
}
