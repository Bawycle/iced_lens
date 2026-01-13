# Technical Assumptions

## Repository Structure

**Monorepo** - The Diagnostics module will be added to the existing IcedLens repository as a new module under `src/diagnostics/`.

## Service Architecture

**Monolith with modular design** - IcedLens is a single desktop application. The Diagnostics module will integrate as a new domain following the existing Elm/Iced architecture pattern:
- Own message types and state
- Effect-based communication with other modules
- Separate collector thread for non-blocking data collection

## Testing Requirements

**Unit + Integration testing:**
- Unit tests for anonymization functions (verify hashing, extension preservation)
- Unit tests for circular buffer operations
- Unit tests for JSON serialization
- Integration tests for collection pipeline
- Manual testing for UI integration and cross-platform behavior

## Additional Technical Assumptions and Requests

- Use `sysinfo` crate or similar for cross-platform system metrics
- Use existing `serde` and `serde_json` for serialization
- Use `blake3` or `sha2` for fast, secure hashing
- Use `arboard` crate for cross-platform clipboard access
- Follow existing newtype patterns for bounded values (e.g., buffer size limits)
- Integrate with existing message/event system for action logging
- Use existing notification system integration for error capture
- The collector should use channels (e.g., `crossbeam-channel`) for thread communication

---
