// SPDX-License-Identifier: MPL-2.0
//! Application layer - Use cases and orchestration.
//!
//! This module contains the application layer of the Clean Architecture:
//!
//! - [`port`]: Trait definitions (interfaces) for dependency inversion
//! - [`query`]: Query services (CQRS read-side)
//!
//! # Architecture
//!
//! The application layer sits between the domain layer (pure business logic)
//! and the infrastructure/presentation layers. It defines:
//!
//! - **Ports (Traits)**: Abstract interfaces that infrastructure implements
//! - **Queries**: Read-only operations on domain data
//!
//! # Dependency Rule
//!
//! - Application layer depends on domain layer (uses domain types)
//! - Infrastructure layer implements application layer ports
//! - Presentation layer uses application layer services
//!
//! # Example
//!
//! ```ignore
//! use iced_lens::application::port::VideoDecoder;
//! use iced_lens::application::query::MediaNavigator;
//!
//! // Infrastructure implements the port trait
//! struct FfmpegDecoder { /* ... */ }
//! impl VideoDecoder for FfmpegDecoder { /* ... */ }
//!
//! // Application services use domain types
//! let navigator = MediaNavigator::new();
//! ```

pub mod port;
pub mod query;
