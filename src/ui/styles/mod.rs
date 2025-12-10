// SPDX-License-Identifier: MPL-2.0
//! Styles centralisés pour tous les composants UI.

pub mod button;
pub mod container;
pub mod editor;
pub mod overlay;

// Re-exports pour backward compatibility
pub use button::{overlay as button_overlay, primary as button_primary};
