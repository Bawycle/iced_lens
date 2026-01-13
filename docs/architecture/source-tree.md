# Source Tree

## Existing Project Structure (Relevant Parts)

```
src/
├── app/
│   ├── message.rs          # Top-level Message enum
│   ├── update.rs           # Message handlers
│   ├── screen.rs           # Screen enum
│   └── ...
├── ui/
│   ├── mod.rs              # UI module exports
│   ├── about.rs            # About screen (reference pattern)
│   ├── settings.rs         # Settings screen
│   ├── help.rs             # Help screen
│   ├── design_tokens.rs    # Design system
│   ├── notifications/      # Toast notification system
│   └── ...
├── media/                  # Media handling
├── video_player/           # Video playback (newtypes here)
└── lib.rs                  # Library root
```

## New File Organization

```
src/
├── diagnostics/                    # NEW: Diagnostics module
│   ├── mod.rs                      # Module exports
│   ├── collector.rs                # DiagnosticsCollector
│   ├── buffer.rs                   # CircularBuffer<T>
│   ├── events.rs                   # DiagnosticEvent types
│   ├── resource_collector.rs       # System metrics collector
│   ├── anonymizer.rs               # Privacy anonymization
│   ├── report.rs                   # DiagnosticReport, JSON schema
│   └── export.rs                   # File and clipboard export
├── app/
│   ├── message.rs                  # MODIFY: Add Diagnostics variant
│   ├── update.rs                   # MODIFY: Add handler
│   └── screen.rs                   # MODIFY: Add Screen::Diagnostics
├── ui/
│   ├── mod.rs                      # MODIFY: Export diagnostics_screen
│   ├── diagnostics_screen.rs       # NEW: Diagnostics UI
│   └── navbar.rs                   # MODIFY: Add menu entry
└── lib.rs                          # MODIFY: Export diagnostics module
```

## Integration Guidelines

| Aspect | Guideline |
|--------|-----------|
| **File Naming** | Snake_case matching existing (e.g., `diagnostics_screen.rs`) |
| **Folder Organization** | New module in `src/diagnostics/`, screen in `src/ui/` |
| **Import/Export Patterns** | Re-export public API via `mod.rs`, use `pub(crate)` for internals |

---
