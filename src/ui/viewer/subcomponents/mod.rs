// SPDX-License-Identifier: MPL-2.0
//! Nested TEA sub-components for the viewer.
//!
//! Each sub-component has its own State, Message, Effect, and handle() method.
//! The main component.rs orchestrates these sub-components.
//!
//! ## Architecture
//!
//! ```text
//! component.rs (orchestrator)
//!     ├── error_state   - Error display state
//!     ├── loading       - Loading spinner
//!     ├── rotation      - Temporary rotation
//!     ├── filter        - Filter dropdown wrapper
//!     ├── overlay       - Fullscreen controls visibility
//!     ├── zoom          - Encapsulates ZoomState
//!     ├── drag          - Encapsulates DragState
//!     ├── navigation    - Nav + skip logic
//!     ├── media_holder  - Current media container
//!     └── video_playback - Video player state
//! ```

pub mod drag;
pub mod error_state;
pub mod filter;
pub mod loading;
pub mod media_holder;
pub mod navigation;
pub mod overlay;
pub mod rotation;
pub mod video_playback;
pub mod zoom;
