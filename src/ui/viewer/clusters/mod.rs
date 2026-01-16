// SPDX-License-Identifier: MPL-2.0
//! Feature clusters for the viewer component.
//!
//! Each cluster groups related functionality by responsibility:
//! - `image_transform`: zoom, pan, rotation
//! - `media_lifecycle`: loading, navigation, media holder, errors (future)
//! - `video_playback`: player, controls, timeline (future)
//!
//! ## Architecture
//!
//! ```text
//! component.rs (orchestrator ~700 LOC)
//!     ├── image_transform   - Zoom + pan + rotation
//!     ├── media_lifecycle   - Loading + nav + media + error (planned)
//!     ├── video_playback    - Video state (planned)
//!     └── overlay           - Standalone (in subcomponents/)
//! ```

pub mod image_transform;
pub mod media_lifecycle;
