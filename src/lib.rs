// SPDX-License-Identifier: MPL-2.0
//! `iced_lens` is a simple image viewer built with the Iced GUI framework.
//!
//! It provides basic image viewing capabilities and demonstrates internationalization
//! with Fluent, user preference management, and modular UI design.

#![doc(html_root_url = "https://docs.rs/iced_lens/0.1.0")]

pub mod app;
pub mod diagnostics;
pub mod directory_scanner;
pub mod error;
pub mod icon;
pub mod media;
pub mod ui;
pub mod video_player;

// Re-export config and i18n from app for backwards compatibility
pub use app::config;
pub use app::i18n;

#[cfg(test)]
pub mod test_utils;

#[cfg(test)]
mod tests {
    // This is where common library tests can go
}
