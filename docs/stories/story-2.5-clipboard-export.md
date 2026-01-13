# Story 2.5: Clipboard Export

**Epic:** 2 - Anonymization & Export System
**Status:** Approved
**Priority:** High
**Estimate:** 1-2 hours
**Depends On:** Story 2.4

---

## Story

**As a** developer,
**I want** to copy diagnostic reports to the clipboard,
**So that** I can quickly paste them into Claude Code or other tools.

---

## Acceptance Criteria

### Clipboard Function
1. `export_to_clipboard()` method added to `DiagnosticsCollector`
2. Uses `arboard` crate for cross-platform clipboard access
3. Returns `Result<(), ExportError>` with success/failure status

### Content
4. Clipboard contains same anonymized JSON as `export_to_file()`
5. Reuses `build_anonymized_report()` from Story 2.4

### Error Handling
6. `ExportError::Clipboard` variant added for clipboard failures
7. Graceful handling if clipboard unavailable (headless, permissions)

### Cross-Platform
8. Works on Linux (X11/Wayland), Windows, and macOS

---

## Tasks

### Task 1: Add `arboard` dependency (AC: 2)
- [ ] Add `arboard = "3.4"` to `Cargo.toml`
- [ ] Verify compilation on current platform

### Task 2: Add `Clipboard` variant to `ExportError` (AC: 6)
- [ ] Add `Clipboard(String)` variant to `ExportError` in `export.rs`
- [ ] Update `Display` impl with descriptive message

### Task 3: Implement `export_to_clipboard()` (AC: 1, 3, 4, 5)
- [ ] Add method to `DiagnosticsCollector`
- [ ] Call `build_anonymized_report()` (from Story 2.4)
- [ ] Serialize to pretty JSON
- [ ] Create `arboard::Clipboard` instance
- [ ] Set text content
- [ ] Map arboard errors to `ExportError::Clipboard`

### Task 4: Handle platform-specific issues (AC: 7, 8)
- [ ] arboard handles X11/Wayland automatically
- [ ] Handle `arboard::Error` variants appropriately
- [ ] Document headless system limitations

### Task 5: Write unit tests (AC: 3, 6)
- [ ] Test success case (if clipboard available)
- [ ] Test error variant is correctly constructed
- [ ] Use `#[ignore]` for CI environments without clipboard

### Task 6: Run validation
- [ ] `cargo fmt --all`
- [ ] `cargo clippy --all --all-targets -- -D warnings`
- [ ] `cargo test`

### Task 7: Commit changes
- [ ] Stage all changes
- [ ] Commit: `feat(diagnostics): add clipboard export [Story 2.5]`

---

## Dev Notes

### Source Tree

```
src/diagnostics/
├── mod.rs              # MODIFY: export updated ExportError
├── export.rs           # MODIFY: add Clipboard variant, export_to_clipboard()
├── collector.rs        # MODIFY: add export_to_clipboard() method
└── Cargo.toml          # MODIFY: add arboard dependency
```

### Dependency

```toml
# Cargo.toml
[dependencies]
arboard = "3.4"
```

### Implementation

```rust
// In src/diagnostics/export.rs

/// Export error types.
#[derive(Debug)]
pub enum ExportError {
    Io(std::io::Error),
    Serialization(serde_json::Error),
    Cancelled,
    Clipboard(String),  // NEW
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {}", e),
            Self::Serialization(e) => write!(f, "Serialization error: {}", e),
            Self::Cancelled => write!(f, "Export cancelled by user"),
            Self::Clipboard(msg) => write!(f, "Clipboard error: {}", msg),
        }
    }
}
```

```rust
// In src/diagnostics/collector.rs

impl DiagnosticsCollector {
    /// Exports anonymized diagnostic report to the system clipboard.
    ///
    /// # Errors
    ///
    /// Returns `ExportError::Clipboard` if clipboard access fails.
    /// This can happen on headless systems or if permissions are denied.
    pub fn export_to_clipboard(&self) -> Result<(), ExportError> {
        // Reuse anonymized report from Story 2.4
        let report = self.build_anonymized_report();
        let json = serde_json::to_string_pretty(&report)
            .map_err(ExportError::Serialization)?;

        // Copy to clipboard
        let mut clipboard = arboard::Clipboard::new()
            .map_err(|e| ExportError::Clipboard(e.to_string()))?;

        clipboard
            .set_text(&json)
            .map_err(|e| ExportError::Clipboard(e.to_string()))?;

        Ok(())
    }
}
```

### arboard Error Handling

arboard can fail in several scenarios:
- **Headless systems**: No display server available
- **Wayland without wl-copy**: Some Wayland compositors need `wl-clipboard`
- **Permission denied**: Sandboxed environments
- **Clipboard busy**: Another application holding the clipboard

All these are mapped to `ExportError::Clipboard(message)`.

### Platform Notes

| Platform | Backend | Notes |
|----------|---------|-------|
| Linux X11 | X11 selections | Works out of the box |
| Linux Wayland | wl-clipboard | May need `wl-copy` installed |
| Windows | Win32 API | Works out of the box |
| macOS | NSPasteboard | Works out of the box |

---

## Testing

### Unit Tests

| Test | Input | Expected Output |
|------|-------|-----------------|
| `clipboard_export_success` | Valid collector | `Ok(())` (if clipboard available) |
| `clipboard_error_display` | `ExportError::Clipboard("msg")` | `"Clipboard error: msg"` |

### Integration Tests

| Test | Verification | Notes |
|------|--------------|-------|
| `export_to_clipboard_content` | Paste and parse JSON | Manual test, mark `#[ignore]` |

### CI Considerations

```rust
#[test]
#[ignore] // Clipboard not available in CI
fn clipboard_export_works() {
    let collector = DiagnosticsCollector::new(BufferCapacity::default());
    // This may fail in headless CI environments
    let result = collector.export_to_clipboard();
    // Just verify it doesn't panic
    let _ = result;
}
```

### Manual Testing Checklist

- [ ] Linux X11: Export and paste into text editor
- [ ] Linux Wayland: Export and paste into text editor
- [ ] Windows: Export and paste into Notepad
- [ ] macOS: Export and paste into TextEdit
- [ ] Verify pasted JSON is valid and anonymized

---

## Dev Agent Record

### Agent Model Used
<!-- Record which AI model completed this story -->

### Completion Notes
<!-- Dev agent adds notes here during implementation -->

### Change Log
| Date | Change | Author |
|------|--------|--------|
| 2026-01-13 | Story created | PO |
| 2026-01-13 | PO Validation: Fixed dependency (2.4 only), added Task-AC mapping, source tree, code examples, platform notes | Sarah (PO) |

### File List
<!-- Files created or modified -->

---
