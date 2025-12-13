// SPDX-License-Identifier: MPL-2.0
//! Internationalization (i18n) support for the application.
//!
//! This module provides localization capabilities using the Fluent localization system.
//! It handles language detection, translation file loading, and string formatting.
//!
//! # Features
//!
//! - Automatic locale detection from CLI, config, or system settings
//! - Dynamic loading of `.ftl` translation files
//! - Runtime language switching
//! - Fallback to default locale when translations are missing

pub mod fluent;
