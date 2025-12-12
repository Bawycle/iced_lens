// SPDX-License-Identifier: MPL-2.0
//! Screen enumeration for application navigation.

/// Screens the user can navigate between.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Viewer,
    Settings,
    ImageEditor,
    Help,
    About,
}
