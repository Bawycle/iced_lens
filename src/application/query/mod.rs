// SPDX-License-Identifier: MPL-2.0
//! Query services (CQRS read-side).
//!
//! This module contains query services for reading and navigating domain data.
//! These services do not modify state; they provide read-only access.
//!
//! # Available Services
//!
//! - [`navigation`]: Media list navigation (`MediaNavigator`)
//!
//! # Design Notes
//!
//! Query services are part of the application layer because they:
//! - Coordinate domain operations
//! - May use infrastructure for data access
//! - Implement application-specific use cases
//!
//! They are not domain entities because they manage external state (file lists).

pub mod navigation;

// Re-export main types
pub use navigation::{MediaNavigator, NavigationInfo};
