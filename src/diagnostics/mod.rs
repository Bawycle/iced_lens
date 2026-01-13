// SPDX-License-Identifier: MPL-2.0
//! Diagnostics module for collecting and exporting anonymized activity reports.
//!
//! This module provides infrastructure for capturing diagnostic events during
//! application usage, storing them in a memory-bounded circular buffer, and
//! exporting them as anonymized JSON reports for performance analysis.
//!
//! # Architecture
//!
//! - [`CircularBuffer`]: Generic ring buffer with configurable capacity
//! - [`DiagnosticEvent`]: Enum representing different types of diagnostic events
//! - [`BufferCapacity`]: Newtype for validated buffer capacity bounds
//!
//! # Privacy
//!
//! All exported data is anonymized before export. File paths, IP addresses,
//! and usernames are hashed to protect user privacy while preserving
//! diagnostic value.

mod buffer;
mod events;

pub use buffer::{BufferCapacity, CircularBuffer};
pub use events::DiagnosticEvent;
