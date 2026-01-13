# Testing Strategy

## Integration with Existing Tests

| Aspect | Current State |
|--------|---------------|
| **Existing Test Framework** | Rust built-in (`#[test]`), Criterion for benchmarks |
| **Test Organization** | Unit tests in-file, integration in `tests/` |
| **Coverage Requirements** | No formal coverage target, but comprehensive unit tests expected |

## New Testing Requirements

### Unit Tests for New Components

| Aspect | Requirement |
|--------|-------------|
| **Framework** | Rust `#[test]`, `approx` for float comparison |
| **Location** | `#[cfg(test)] mod tests` in each source file |
| **Coverage Target** | All public APIs, edge cases for buffer/anonymizer |
| **Integration with Existing** | Run with `cargo test` |

**Required Unit Tests:**
- `buffer.rs`: Add, overflow, iteration, clear, capacity
- `anonymizer.rs`: Path hashing, extension preservation, IP detection, username detection
- `events.rs`: Serialization roundtrip for all variants
- `report.rs`: JSON schema validation
- `collector.rs`: Start/stop, event logging, export

### Integration Tests

| Aspect | Requirement |
|--------|-------------|
| **Scope** | Full pipeline: collect → buffer → anonymize → export |
| **Existing System Verification** | Ensure diagnostics don't affect normal app operation |
| **New Feature Testing** | End-to-end export produces valid, anonymized JSON |

### Regression Testing

| Aspect | Requirement |
|--------|-------------|
| **Existing Feature Verification** | Run full test suite before merge |
| **Automated Regression Suite** | `cargo test --all` in CI |
| **Manual Testing Requirements** | Cross-platform clipboard, UI navigation |

---
