# Coding Standards

## Existing Standards Compliance

| Aspect | Standard |
|--------|----------|
| **Code Style** | `cargo fmt`, Clippy pedantic |
| **Linting Rules** | `#[warn(clippy::pedantic)]` in Cargo.toml |
| **Testing Patterns** | Unit tests in same file (`#[cfg(test)]` mod), integration tests in `tests/` |
| **Documentation Style** | Rustdoc with `//!` module docs, `///` item docs |

## Enhancement-Specific Standards

| Standard | Description |
|----------|-------------|
| **Newtype pattern** | Use for bounded values (BufferCapacity, SamplingInterval) |
| **Channel communication** | Use `crossbeam-channel` for thread communication |
| **Error handling** | Return `Result<T, DiagnosticsError>` for fallible operations |
| **Anonymization** | All string data must pass through Anonymizer before export |

## Critical Integration Rules

| Aspect | Rule |
|--------|------|
| **Existing API Compatibility** | Extend `Message` enum, don't modify existing variants |
| **Database Integration** | N/A |
| **Error Handling** | Use existing `Notification` system for user feedback |
| **Logging Consistency** | Use `log_event()` channel, not direct buffer access from UI thread |

---
