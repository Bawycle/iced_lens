// SPDX-License-Identifier: MPL-2.0
//! User interface components and state management.
//!
//! This module organizes all UI-related code following a component-based architecture
//! with the Elm-style "state down, messages up" pattern.
//!
//! # Screens
//!
//! - [`viewer`] - Main image/video viewer with zoom, pan, and navigation
//! - [`image_editor`] - Image editing with rotate, crop, resize, and flip tools
//! - [`settings`] - Application preferences and configuration
//! - [`help`] - Keyboard shortcuts and usage documentation
//! - [`about`] - Application version and credits
//!
//! # Shared Infrastructure
//!
//! - [`state`] - Reusable state management (zoom, viewport, drag)
//! - [`components`] - Reusable UI components (error display, checkerboard)
//! - [`widgets`] - Custom Iced widgets (spinner, video canvas)
//! - [`styles`] - Centralized styling (buttons, containers, overlays)
//! - [`design_tokens`] - Design system constants (colors, spacing, sizing)
//! - [`theme`] - Theme colors and styling helpers
//! - [`theming`] - Light/Dark/System theme mode management
//! - [`icons`] - SVG icon loading and rendering (visual primitives)
//! - [`action_icons`] - Semantic action-to-icon mapping
//! - [`navbar`] - Navigation bar with hamburger menu
//! - [`notifications`] - Toast notification system for user feedback

pub mod about;
pub mod action_icons;
pub mod components;
pub mod design_tokens;
pub mod help;
pub mod icons;
pub mod image_editor;
pub mod metadata_panel;
pub mod navbar;
pub mod notifications;
pub mod settings;
pub mod state;
pub mod styles;
pub mod theme;
pub mod theming;
pub mod viewer;
pub mod widgets;
