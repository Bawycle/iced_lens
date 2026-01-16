// SPDX-License-Identifier: MPL-2.0
//! System resource metrics collection for diagnostics.
//!
//! This module provides infrastructure for collecting system metrics (CPU, RAM, disk I/O)
//! at regular intervals on a background thread without blocking the UI.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crossbeam_channel::{bounded, Receiver, Sender};
use serde::{Deserialize, Serialize};
use sysinfo::{Disks, System};

use crate::config::{
    DEFAULT_SAMPLING_INTERVAL_MS, MAX_SAMPLING_INTERVAL_MS, MIN_SAMPLING_INTERVAL_MS,
};

/// Sampling interval for resource metrics collection.
///
/// This newtype enforces validity at the type level, ensuring the value
/// is always within the valid range (100ms–60000ms).
///
/// # Example
///
/// ```
/// use iced_lens::diagnostics::SamplingInterval;
///
/// let interval = SamplingInterval::new(1000);
/// assert_eq!(interval.value(), 1000);
///
/// // Values outside range are clamped
/// let too_low = SamplingInterval::new(10);
/// assert_eq!(too_low.value(), 100); // Clamped to min
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SamplingInterval(u64);

impl SamplingInterval {
    /// Creates a new sampling interval, clamping to valid range.
    #[must_use]
    pub fn new(value_ms: u64) -> Self {
        Self(value_ms.clamp(MIN_SAMPLING_INTERVAL_MS, MAX_SAMPLING_INTERVAL_MS))
    }

    /// Returns the value in milliseconds.
    #[must_use]
    pub fn value(self) -> u64 {
        self.0
    }

    /// Returns the interval as a Duration.
    #[must_use]
    pub fn as_duration(self) -> Duration {
        Duration::from_millis(self.0)
    }

    /// Returns true if this is the minimum value.
    #[must_use]
    pub fn is_min(self) -> bool {
        self.0 <= MIN_SAMPLING_INTERVAL_MS
    }

    /// Returns true if this is the maximum value.
    #[must_use]
    pub fn is_max(self) -> bool {
        self.0 >= MAX_SAMPLING_INTERVAL_MS
    }
}

impl Default for SamplingInterval {
    fn default() -> Self {
        Self(DEFAULT_SAMPLING_INTERVAL_MS)
    }
}

/// System resource metrics snapshot.
///
/// Contains CPU, RAM, and disk I/O measurements at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceMetrics {
    /// CPU usage percentage (0.0–100.0)
    pub cpu_percent: f32,
    /// RAM used in bytes
    pub ram_used_bytes: u64,
    /// Total RAM in bytes
    pub ram_total_bytes: u64,
    /// Total disk bytes read since last sample
    pub disk_read_bytes: u64,
    /// Total disk bytes written since last sample
    pub disk_write_bytes: u64,
}

impl ResourceMetrics {
    /// Creates a new `ResourceMetrics` instance.
    #[must_use]
    pub fn new(
        cpu_percent: f32,
        ram_used_bytes: u64,
        ram_total_bytes: u64,
        disk_read_bytes: u64,
        disk_write_bytes: u64,
    ) -> Self {
        Self {
            cpu_percent: cpu_percent.clamp(0.0, 100.0),
            ram_used_bytes,
            ram_total_bytes,
            disk_read_bytes,
            disk_write_bytes,
        }
    }

    /// Returns RAM usage as a percentage (0.0–100.0).
    #[must_use]
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    pub fn ram_percent(&self) -> f32 {
        if self.ram_total_bytes == 0 {
            0.0
        } else {
            (self.ram_used_bytes as f64 / self.ram_total_bytes as f64 * 100.0) as f32
        }
    }
}

/// Commands that can be sent to the resource collector thread.
#[derive(Debug, Clone, Copy)]
pub enum CollectorCommand {
    /// Stop the collector and exit the thread
    Stop,
}

/// Resource collector that samples system metrics on a background thread.
///
/// The collector can be started with [`ResourceCollector::start`] and stopped
/// by dropping it or sending a stop command.
pub struct ResourceCollector {
    /// Channel to send commands to the collector thread
    command_tx: Sender<CollectorCommand>,
    /// Handle to the collector thread
    thread_handle: Option<JoinHandle<()>>,
    /// Flag indicating if collector is running
    running: Arc<AtomicBool>,
}

impl ResourceCollector {
    /// Starts a new resource collector that sends metrics to the provided channel.
    ///
    /// The collector runs on a background thread and samples metrics at the
    /// specified interval.
    ///
    /// # Arguments
    ///
    /// * `interval` - Sampling interval between metric collections
    /// * `metrics_tx` - Channel to send collected metrics
    #[must_use]
    pub fn start(interval: SamplingInterval, metrics_tx: Sender<ResourceMetrics>) -> Self {
        let (command_tx, command_rx) = bounded::<CollectorCommand>(1);
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = Arc::clone(&running);

        let thread_handle = thread::spawn(move || {
            Self::collector_loop(interval, &metrics_tx, &command_rx, &running_clone);
        });

        Self {
            command_tx,
            thread_handle: Some(thread_handle),
            running,
        }
    }

    /// Returns true if the collector is currently running.
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Stops the collector and waits for the thread to finish.
    pub fn stop(&mut self) {
        if self.is_running() {
            // Send stop command (ignore errors if channel is disconnected)
            let _ = self.command_tx.send(CollectorCommand::Stop);

            // Wait for thread to finish
            if let Some(handle) = self.thread_handle.take() {
                let _ = handle.join();
            }
        }
    }

    /// The main collection loop running on the background thread.
    fn collector_loop(
        interval: SamplingInterval,
        metrics_tx: &Sender<ResourceMetrics>,
        command_rx: &Receiver<CollectorCommand>,
        running: &Arc<AtomicBool>,
    ) {
        let mut sys = System::new_all();
        let disks = Disks::new_with_refreshed_list();

        // Initial refresh to get accurate CPU readings
        sys.refresh_all();
        thread::sleep(Duration::from_millis(100));

        while running.load(Ordering::SeqCst) {
            // Check for stop command (non-blocking)
            if let Ok(CollectorCommand::Stop) = command_rx.try_recv() {
                running.store(false, Ordering::SeqCst);
                break;
            }

            // Refresh system info
            sys.refresh_all();

            // Collect metrics
            let metrics = Self::collect_metrics(&sys, &disks);

            // Send metrics (if channel is full or disconnected, skip this sample)
            if metrics_tx.try_send(metrics).is_err() {
                // Channel full or disconnected, continue to next iteration
            }

            // Sleep for interval (check for stop command periodically)
            let sleep_duration = interval.as_duration();
            let sleep_step = Duration::from_millis(100);
            let mut slept = Duration::ZERO;

            while slept < sleep_duration && running.load(Ordering::SeqCst) {
                if let Ok(CollectorCommand::Stop) = command_rx.try_recv() {
                    running.store(false, Ordering::SeqCst);
                    break;
                }
                let remaining = sleep_duration.saturating_sub(slept);
                thread::sleep(sleep_step.min(remaining));
                slept += sleep_step;
            }
        }
    }

    /// Collects current system metrics.
    fn collect_metrics(sys: &System, disks: &Disks) -> ResourceMetrics {
        // CPU usage (average across all CPUs)
        let cpu_percent = sys.global_cpu_usage();

        // RAM usage
        let ram_used_bytes = sys.used_memory();
        let ram_total_bytes = sys.total_memory();

        // Disk I/O (sum across all disks)
        // Note: sysinfo doesn't provide cumulative read/write bytes per disk,
        // so we use a placeholder. Real I/O tracking would require procfs on Linux.
        let (disk_read_bytes, disk_write_bytes) = Self::get_disk_io(disks);

        ResourceMetrics::new(
            cpu_percent,
            ram_used_bytes,
            ram_total_bytes,
            disk_read_bytes,
            disk_write_bytes,
        )
    }

    /// Gets disk I/O statistics.
    ///
    /// Note: Disk I/O stats are platform-dependent. This returns available
    /// space as a proxy for now. Detailed I/O tracking would require
    /// platform-specific APIs.
    fn get_disk_io(disks: &Disks) -> (u64, u64) {
        // Sum available space as a simple metric
        // Real I/O tracking would need /proc/diskstats on Linux
        let total_available: u64 = disks.iter().map(sysinfo::Disk::available_space).sum();
        let total_space: u64 = disks.iter().map(sysinfo::Disk::total_space).sum();
        let used_space = total_space.saturating_sub(total_available);

        // Return used space as a proxy (not actual I/O, but useful for diagnostics)
        (used_space, 0)
    }
}

impl Drop for ResourceCollector {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    // SamplingInterval tests

    #[test]
    fn sampling_interval_clamps_to_valid_range() {
        assert_eq!(SamplingInterval::new(0).value(), MIN_SAMPLING_INTERVAL_MS);
        assert_eq!(
            SamplingInterval::new(100_000).value(),
            MAX_SAMPLING_INTERVAL_MS
        );
    }

    #[test]
    fn sampling_interval_accepts_valid_values() {
        assert_eq!(SamplingInterval::new(100).value(), 100);
        assert_eq!(SamplingInterval::new(1000).value(), 1000);
        assert_eq!(SamplingInterval::new(5000).value(), 5000);
    }

    #[test]
    fn sampling_interval_default_returns_expected_value() {
        assert_eq!(
            SamplingInterval::default().value(),
            DEFAULT_SAMPLING_INTERVAL_MS
        );
    }

    #[test]
    fn sampling_interval_as_duration() {
        let interval = SamplingInterval::new(1000);
        assert_eq!(interval.as_duration(), Duration::from_millis(1000));
    }

    #[test]
    fn sampling_interval_is_min_detects_minimum() {
        assert!(SamplingInterval::new(MIN_SAMPLING_INTERVAL_MS).is_min());
        assert!(!SamplingInterval::new(1000).is_min());
    }

    #[test]
    fn sampling_interval_is_max_detects_maximum() {
        assert!(SamplingInterval::new(MAX_SAMPLING_INTERVAL_MS).is_max());
        assert!(!SamplingInterval::new(1000).is_max());
    }

    // ResourceMetrics tests

    #[test]
    fn resource_metrics_new_creates_correctly() {
        let metrics = ResourceMetrics::new(50.0, 4_000_000_000, 8_000_000_000, 1000, 2000);

        assert_relative_eq!(metrics.cpu_percent, 50.0, epsilon = 0.01);
        assert_eq!(metrics.ram_used_bytes, 4_000_000_000);
        assert_eq!(metrics.ram_total_bytes, 8_000_000_000);
        assert_eq!(metrics.disk_read_bytes, 1000);
        assert_eq!(metrics.disk_write_bytes, 2000);
    }

    #[test]
    fn resource_metrics_clamps_cpu_percent() {
        let low = ResourceMetrics::new(-10.0, 0, 0, 0, 0);
        assert_relative_eq!(low.cpu_percent, 0.0, epsilon = 0.01);

        let high = ResourceMetrics::new(150.0, 0, 0, 0, 0);
        assert_relative_eq!(high.cpu_percent, 100.0, epsilon = 0.01);
    }

    #[test]
    fn resource_metrics_ram_percent_calculation() {
        let metrics = ResourceMetrics::new(0.0, 4_000_000_000, 8_000_000_000, 0, 0);
        assert_relative_eq!(metrics.ram_percent(), 50.0, epsilon = 0.01);

        let zero_total = ResourceMetrics::new(0.0, 1000, 0, 0, 0);
        assert_relative_eq!(zero_total.ram_percent(), 0.0, epsilon = 0.01);
    }

    #[test]
    fn resource_metrics_serializes_to_json() {
        let metrics = ResourceMetrics::new(25.5, 2_000_000_000, 4_000_000_000, 100, 200);
        let json = serde_json::to_string(&metrics).expect("serialization should succeed");

        assert!(json.contains("\"cpu_percent\":25.5"));
        assert!(json.contains("\"ram_used_bytes\":2000000000"));
    }

    // ResourceCollector tests

    #[test]
    fn resource_collector_starts_and_stops() {
        let (tx, _rx) = bounded::<ResourceMetrics>(10);
        let mut collector = ResourceCollector::start(SamplingInterval::new(100), tx);

        assert!(collector.is_running());

        collector.stop();

        assert!(!collector.is_running());
    }

    #[test]
    fn resource_collector_sends_metrics() {
        let (tx, rx) = bounded::<ResourceMetrics>(10);
        let mut collector = ResourceCollector::start(SamplingInterval::new(100), tx);

        // Wait for at least one metric
        let timeout = Duration::from_secs(2);
        let metrics = rx.recv_timeout(timeout);

        collector.stop();

        assert!(metrics.is_ok(), "Should receive at least one metric");
        let metrics = metrics.unwrap();

        // CPU should be between 0 and 100
        assert!(metrics.cpu_percent >= 0.0);
        assert!(metrics.cpu_percent <= 100.0);

        // RAM total should be non-zero on any real system
        assert!(metrics.ram_total_bytes > 0);
    }

    #[test]
    fn resource_collector_drop_stops_thread() {
        let (tx, _rx) = bounded::<ResourceMetrics>(10);
        let running = {
            let collector = ResourceCollector::start(SamplingInterval::new(100), tx);
            Arc::clone(&collector.running)
        };

        // After drop, running should be false
        // Give it a moment to clean up
        thread::sleep(Duration::from_millis(200));
        assert!(!running.load(Ordering::SeqCst));
    }
}
