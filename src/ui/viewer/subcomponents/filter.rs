// SPDX-License-Identifier: MPL-2.0
//! Filter dropdown sub-component wrapper.

use crate::ui::viewer::filter_dropdown::{DateSegment, DateTarget, FilterDropdownState};
use std::time::SystemTime;

/// Wrapper state encapsulating `FilterDropdownState`.
#[derive(Debug, Clone, Default)]
pub struct State {
    /// Inner state from existing `filter_dropdown` module.
    inner: FilterDropdownState,
}

/// Messages for the filter sub-component.
#[derive(Debug, Clone)]
pub enum Message {
    /// Toggle the dropdown open/closed.
    Toggle,
    /// Close the dropdown.
    Close,
    /// Update a date segment value.
    SetDateSegment {
        target: DateTarget,
        segment: DateSegment,
        value: String,
    },
    /// Clear both date fields.
    ClearDates,
    /// Apply the current filter.
    ApplyFilter,
}

/// Effects produced by filter changes.
#[derive(Debug, Clone)]
pub enum Effect {
    /// No effect.
    None,
    /// Apply date filter with parsed dates.
    ApplyDateFilter {
        start: Option<SystemTime>,
        end: Option<SystemTime>,
    },
}

impl State {
    /// Access inner `FilterDropdownState` (read-only).
    #[must_use]
    pub fn inner(&self) -> &FilterDropdownState {
        &self.inner
    }

    /// Mutable access for view updates.
    pub fn inner_mut(&mut self) -> &mut FilterDropdownState {
        &mut self.inner
    }

    /// Check if the dropdown is open.
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.inner.is_open
    }

    /// Handle a filter message.
    ///
    /// Note: Takes `Message` by value following Iced's `update(message: Message)` pattern.
    #[allow(clippy::needless_pass_by_value)]
    pub fn handle(&mut self, msg: Message) -> Effect {
        match msg {
            Message::Toggle => {
                self.inner.toggle();
                Effect::None
            }
            Message::Close => {
                self.inner.close();
                Effect::None
            }
            Message::SetDateSegment {
                target,
                segment,
                value,
            } => {
                self.inner.set_segment(target, segment, &value);
                Effect::None
            }
            Message::ClearDates => {
                self.inner.start_date.clear();
                self.inner.end_date.clear();
                Effect::None
            }
            Message::ApplyFilter => {
                let start = self.inner.start_date.to_system_time();
                let end = self.inner.end_date.to_system_time();
                self.inner.close();
                Effect::ApplyDateFilter { start, end }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_opens_and_closes() {
        let mut state = State::default();
        assert!(!state.is_open());

        state.handle(Message::Toggle);
        assert!(state.is_open());

        state.handle(Message::Toggle);
        assert!(!state.is_open());
    }

    #[test]
    fn close_closes_dropdown() {
        let mut state = State::default();
        state.handle(Message::Toggle);
        assert!(state.is_open());

        state.handle(Message::Close);
        assert!(!state.is_open());
    }

    #[test]
    fn apply_filter_closes_and_returns_effect() {
        let mut state = State::default();
        state.handle(Message::Toggle);

        let effect = state.handle(Message::ApplyFilter);
        assert!(!state.is_open());
        assert!(matches!(effect, Effect::ApplyDateFilter { .. }));
    }
}
