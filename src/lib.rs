// SPDX-License-Identifier: MPL-2.0
//! `iced_lens` is a simple image viewer built with the Iced GUI framework.
//!
//! It provides basic image viewing capabilities and demonstrates internationalization
//! with Fluent, user preference management, and modular UI design.

#![doc(html_root_url = "https://docs.rs/iced_lens/0.1.0")]

pub mod app;
pub mod config;
pub mod error;
pub mod i18n;
pub mod icon;
pub mod image_handler;
pub mod ui;

#[cfg(test)]
mod tests {
    // This is where common library tests can go
}
