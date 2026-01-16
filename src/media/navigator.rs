// SPDX-License-Identifier: MPL-2.0
//! Media navigation re-export for backward compatibility.
//!
//! This module re-exports [`MediaNavigator`] and [`NavigationInfo`] from their
//! new location in the application layer (`application::query::navigation`).
//!
//! # Migration Note
//!
//! The navigation types have been moved to the application layer as part of
//! the DDD + Clean Architecture migration. They are application-level query
//! services, not domain types.
//!
//! Prefer importing from the new location:
//! ```ignore
//! use iced_lens::application::query::{MediaNavigator, NavigationInfo};
//! ```

pub use crate::application::query::navigation::*;
