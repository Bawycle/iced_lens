// SPDX-License-Identifier: MPL-2.0
//! Message sanitization and warning/error type definitions.
//!
//! This module provides:
//! - Type enums for categorizing warnings and errors
//! - Message sanitization to remove sensitive data (file paths, PII)

use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Serialize};

// =============================================================================
// Warning and Error Type Enums
// =============================================================================

/// Categories of warnings that can occur in the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WarningType {
    /// A requested file was not found.
    FileNotFound,
    /// The media format is not supported.
    UnsupportedFormat,
    /// Permission was denied for an operation.
    PermissionDenied,
    /// A network-related issue occurred.
    NetworkError,
    /// A configuration issue was detected.
    ConfigurationIssue,
    /// Metadata could not be written to the saved file.
    MetadataIssue,
    /// Other warning type not covered by specific categories.
    Other,
}

/// Categories of errors that can occur in the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorType {
    /// Input/output error (file read/write failures).
    IoError,
    /// Media decoding error.
    DecodeError,
    /// Export/save operation error.
    ExportError,
    /// AI model loading or inference error.
    #[serde(rename = "ai_model_error")]
    AIModelError,
    /// Internal application error.
    InternalError,
    /// Other error type not covered by specific categories.
    Other,
}

// =============================================================================
// Message Sanitization
// =============================================================================

/// Compiled regex patterns for path detection.
static PATH_PATTERNS: LazyLock<Regex> = LazyLock::new(|| {
    // Matches:
    // - Unix paths: /home/..., /Users/..., /tmp/..., /var/..., /etc/..., /opt/...
    // - Windows paths: C:\..., D:\..., etc.
    // - Windows UNC paths: \\server\share\...
    // - Home shortcut: ~/...
    // Path continues until whitespace or common delimiters (quotes, parens, brackets)
    // Character class [^\s"'()\[\]] excludes whitespace and common string delimiters
    Regex::new(concat!(
        r#"("#,
        r#"/home/[^\s"'()\[\]]+"#,       // Linux home
        r#"|/Users/[^\s"'()\[\]]+"#,     // macOS home
        r#"|/tmp/[^\s"'()\[\]]+"#,       // Temp directory
        r#"|/var/[^\s"'()\[\]]+"#,       // Variable data
        r#"|/etc/[^\s"'()\[\]]+"#,       // Config files
        r#"|/opt/[^\s"'()\[\]]+"#,       // Optional software
        r#"|~/[^\s"'()\[\]]+"#,          // Home shortcut (all platforms)
        r#"|[A-Za-z]:\\[^\s"'()\[\]]+"#, // Windows drive paths (C:\, D:\, etc.)
        r#"|\\\\[^\s"'()\[\]]+"#,        // Windows UNC paths (\\server\share)
        r#")"#,
    ))
    .expect("path regex should compile")
});

/// Sanitizes a message by removing sensitive information.
///
/// Currently removes:
/// - Unix file paths (`/home/...`, `/Users/...`, `/tmp/...`, etc.)
/// - Windows file paths (`C:\...`, `D:\...`, etc.)
///
/// Paths are replaced with `<path>` placeholder to preserve message structure
/// while protecting user privacy.
///
/// # Examples
///
/// ```
/// use iced_lens::diagnostics::sanitize_message;
///
/// let msg = "Failed to open /home/user/photos/image.jpg";
/// assert_eq!(sanitize_message(msg), "Failed to open <path>");
///
/// let msg = "Cannot read C:\\Users\\name\\file.txt";
/// assert_eq!(sanitize_message(msg), "Cannot read <path>");
///
/// let msg = "Invalid format";
/// assert_eq!(sanitize_message(msg), "Invalid format");
/// ```
#[must_use]
pub fn sanitize_message(message: &str) -> String {
    PATH_PATTERNS.replace_all(message, "<path>").into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // WarningType Tests
    // =========================================================================

    #[test]
    fn warning_type_serializes_to_snake_case() {
        assert_eq!(
            serde_json::to_string(&WarningType::FileNotFound).unwrap(),
            "\"file_not_found\""
        );
        assert_eq!(
            serde_json::to_string(&WarningType::UnsupportedFormat).unwrap(),
            "\"unsupported_format\""
        );
        assert_eq!(
            serde_json::to_string(&WarningType::PermissionDenied).unwrap(),
            "\"permission_denied\""
        );
        assert_eq!(
            serde_json::to_string(&WarningType::NetworkError).unwrap(),
            "\"network_error\""
        );
        assert_eq!(
            serde_json::to_string(&WarningType::ConfigurationIssue).unwrap(),
            "\"configuration_issue\""
        );
        assert_eq!(
            serde_json::to_string(&WarningType::Other).unwrap(),
            "\"other\""
        );
    }

    #[test]
    fn warning_type_deserializes_from_snake_case() {
        assert_eq!(
            serde_json::from_str::<WarningType>("\"file_not_found\"").unwrap(),
            WarningType::FileNotFound
        );
        assert_eq!(
            serde_json::from_str::<WarningType>("\"network_error\"").unwrap(),
            WarningType::NetworkError
        );
    }

    // =========================================================================
    // ErrorType Tests
    // =========================================================================

    #[test]
    fn error_type_serializes_to_snake_case() {
        assert_eq!(
            serde_json::to_string(&ErrorType::IoError).unwrap(),
            "\"io_error\""
        );
        assert_eq!(
            serde_json::to_string(&ErrorType::DecodeError).unwrap(),
            "\"decode_error\""
        );
        assert_eq!(
            serde_json::to_string(&ErrorType::ExportError).unwrap(),
            "\"export_error\""
        );
        assert_eq!(
            serde_json::to_string(&ErrorType::AIModelError).unwrap(),
            "\"ai_model_error\""
        );
        assert_eq!(
            serde_json::to_string(&ErrorType::InternalError).unwrap(),
            "\"internal_error\""
        );
        assert_eq!(
            serde_json::to_string(&ErrorType::Other).unwrap(),
            "\"other\""
        );
    }

    #[test]
    fn error_type_deserializes_from_snake_case() {
        assert_eq!(
            serde_json::from_str::<ErrorType>("\"io_error\"").unwrap(),
            ErrorType::IoError
        );
        assert_eq!(
            serde_json::from_str::<ErrorType>("\"ai_model_error\"").unwrap(),
            ErrorType::AIModelError
        );
    }

    // =========================================================================
    // Sanitizer Tests
    // =========================================================================

    #[test]
    fn sanitize_message_removes_unix_home_paths() {
        let msg = "Failed to open /home/user/photos/image.jpg";
        assert_eq!(sanitize_message(msg), "Failed to open <path>");
    }

    #[test]
    fn sanitize_message_removes_unix_users_paths() {
        let msg = "Cannot read /Users/john/Documents/file.txt";
        assert_eq!(sanitize_message(msg), "Cannot read <path>");
    }

    #[test]
    fn sanitize_message_removes_unix_tmp_paths() {
        let msg = "Temp file /tmp/iced_lens_12345/cache.bin not found";
        assert_eq!(sanitize_message(msg), "Temp file <path> not found");
    }

    #[test]
    fn sanitize_message_removes_unix_var_paths() {
        let msg = "Log file at /var/log/iced_lens.log";
        assert_eq!(sanitize_message(msg), "Log file at <path>");
    }

    #[test]
    fn sanitize_message_removes_windows_paths() {
        let msg = "Cannot read C:\\Users\\name\\file.txt";
        assert_eq!(sanitize_message(msg), "Cannot read <path>");
    }

    #[test]
    fn sanitize_message_removes_windows_paths_double_backslash() {
        let msg = "Cannot read C:\\\\Users\\\\name\\\\file.txt";
        assert_eq!(sanitize_message(msg), "Cannot read <path>");
    }

    #[test]
    fn sanitize_message_removes_multiple_paths() {
        let msg = "Copy from /home/user/src.jpg to /tmp/dest.jpg failed";
        assert_eq!(sanitize_message(msg), "Copy from <path> to <path> failed");
    }

    #[test]
    fn sanitize_message_preserves_messages_without_paths() {
        let msg = "Invalid image format";
        assert_eq!(sanitize_message(msg), "Invalid image format");
    }

    #[test]
    fn sanitize_message_preserves_empty_messages() {
        assert_eq!(sanitize_message(""), "");
    }

    #[test]
    fn sanitize_message_handles_paths_at_end() {
        let msg = "Error reading /home/user/test.png";
        assert_eq!(sanitize_message(msg), "Error reading <path>");
    }

    #[test]
    fn sanitize_message_handles_paths_in_quotes() {
        let msg = "File \"/home/user/test.png\" not found";
        assert_eq!(sanitize_message(msg), "File \"<path>\" not found");
    }

    #[test]
    fn sanitize_message_removes_home_shortcut_paths() {
        let msg = "Cannot read ~/Documents/secret.txt";
        assert_eq!(sanitize_message(msg), "Cannot read <path>");
    }

    #[test]
    fn sanitize_message_removes_home_shortcut_nested() {
        let msg = "Config at ~/.config/iced_lens/settings.json";
        assert_eq!(sanitize_message(msg), "Config at <path>");
    }

    #[test]
    fn sanitize_message_removes_windows_unc_paths() {
        let msg = "Cannot access \\\\server\\share\\file.txt";
        assert_eq!(sanitize_message(msg), "Cannot access <path>");
    }

    #[test]
    fn sanitize_message_removes_windows_unc_with_username() {
        let msg = "Mapped drive \\\\fileserver\\users\\john\\documents";
        assert_eq!(sanitize_message(msg), "Mapped drive <path>");
    }
}
