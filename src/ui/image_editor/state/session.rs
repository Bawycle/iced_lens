// SPDX-License-Identifier: MPL-2.0
//! Navigation/save helpers that keep the editor facade slim.

use crate::ui::image_editor::{Event, ImageSource, State};

impl State {
    pub(crate) fn toolbar_back_to_viewer(&mut self) -> Event {
        if self.has_unsaved_changes() {
            Event::None
        } else {
            Event::ExitEditor
        }
    }

    pub(crate) fn sidebar_navigate_next(&mut self) -> Event {
        // Navigation is disabled for captured frames
        if self.is_captured_frame() {
            return Event::None;
        }

        if self.has_unsaved_changes() {
            Event::None
        } else {
            self.commit_active_tool_changes();
            Event::NavigateNext
        }
    }

    pub(crate) fn sidebar_navigate_previous(&mut self) -> Event {
        // Navigation is disabled for captured frames
        if self.is_captured_frame() {
            return Event::None;
        }

        if self.has_unsaved_changes() {
            Event::None
        } else {
            self.commit_active_tool_changes();
            Event::NavigatePrevious
        }
    }

    pub(crate) fn sidebar_save(&mut self) -> Event {
        // Save is only available for file mode, not captured frames
        let path = match &self.image_source {
            ImageSource::File(path) => path.clone(),
            ImageSource::CapturedFrame { .. } => return Event::None,
        };

        self.commit_active_tool_changes();
        Event::SaveRequested {
            path,
            overwrite: true,
        }
    }

    pub(crate) fn sidebar_save_as(&mut self) -> Event {
        self.commit_active_tool_changes();
        Event::SaveAsRequested
    }

    pub(crate) fn sidebar_cancel(&mut self) -> Event {
        self.discard_changes();
        Event::None
    }
}
