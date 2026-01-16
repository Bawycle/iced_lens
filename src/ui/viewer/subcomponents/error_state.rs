// SPDX-License-Identifier: MPL-2.0
//! Error display state sub-component.

use crate::i18n::fluent::I18n;

/// Error state for displaying user-friendly errors with optional details.
#[derive(Debug, Clone)]
pub struct State {
    /// i18n key for the friendly error message.
    friendly_key: &'static str,
    /// Translated friendly error message.
    friendly_text: String,
    /// Technical error details.
    details: String,
    /// Whether to show the technical details.
    show_details: bool,
}

/// Messages for the error state sub-component.
#[derive(Debug, Clone)]
pub enum Message {
    /// Toggle visibility of technical details.
    ToggleDetails,
    /// Clear the error (handled by orchestrator).
    Clear,
}

impl State {
    /// Create a new error state with the given i18n key and details.
    #[must_use]
    pub fn new(friendly_key: &'static str, details: String, i18n: &I18n) -> Self {
        Self {
            friendly_key,
            friendly_text: i18n.tr(friendly_key),
            details,
            show_details: false,
        }
    }

    /// Handle an error state message.
    pub fn handle(&mut self, msg: Message) {
        match msg {
            Message::ToggleDetails => self.show_details = !self.show_details,
            Message::Clear => { /* handled by orchestrator */ }
        }
    }

    /// Refresh the translation when locale changes.
    pub fn refresh_translation(&mut self, i18n: &I18n) {
        self.friendly_text = i18n.tr(self.friendly_key);
    }

    /// Get the friendly error message.
    #[must_use]
    pub fn friendly_text(&self) -> &str {
        &self.friendly_text
    }

    /// Get the technical error details.
    #[must_use]
    pub fn details(&self) -> &str {
        &self.details
    }

    /// Check if details are currently shown.
    #[must_use]
    pub fn show_details(&self) -> bool {
        self.show_details
    }

    /// Get the i18n key.
    #[must_use]
    pub fn friendly_key(&self) -> &'static str {
        self.friendly_key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_details_flips_state() {
        let i18n = I18n::default();
        let mut state = State::new("error-key", "details".into(), &i18n);
        assert!(!state.show_details());
        state.handle(Message::ToggleDetails);
        assert!(state.show_details());
        state.handle(Message::ToggleDetails);
        assert!(!state.show_details());
    }

    #[test]
    fn getters_return_correct_values() {
        let i18n = I18n::default();
        let state = State::new("test-key", "test details".into(), &i18n);
        assert_eq!(state.friendly_key(), "test-key");
        assert_eq!(state.details(), "test details");
    }
}
