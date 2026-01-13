# Next Steps

## Story Manager Handoff

> The Diagnostics Tool architecture is complete. Begin implementation with Epic 1, Story 1.1 (Module Structure and Circular Buffer). Key integration requirements:
>
> - Create `src/diagnostics/` module following existing patterns
> - Use newtype pattern for `BufferCapacity`
> - Ensure all public APIs are documented with rustdoc
> - Unit tests required for buffer operations
>
> Reference this architecture document for component design. The first story establishes the foundation; subsequent stories build incrementally.

## Developer Handoff

> **Architecture Reference:** `docs/architecture.md`
>
> **Key Technical Decisions:**
> 1. Collector runs on background thread with channel communication
> 2. Events stored in generic `CircularBuffer<T>` (reusable)
> 3. Anonymization applied at export time, not collection time
> 4. Follow existing newtype pattern (see `src/video_player/frame_cache_size.rs`)
> 5. Follow existing screen pattern (see `src/ui/about.rs`)
>
> **Implementation Sequence:**
> 1. Epic 1: Core module, buffer, collectors (can test without UI)
> 2. Epic 2: Anonymization, export (can test via unit tests)
> 3. Epic 3: UI integration (requires Epic 1-2 complete)
>
> **Dependencies to Add:**
> ```toml
> sysinfo = "0.32"
> arboard = "3.4"
> crossbeam-channel = "0.5"
> ```
>
> **Verification:** Run `cargo test --all` and `cargo clippy --all` after each story.

---

*Generated using the BMAD-METHOD Brownfield Architecture Template v2.0*
