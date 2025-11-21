<!--
---
Sync Impact Report
---
- **Version Change**: None → 1.0.0
- **Summary**: Initialized the project constitution with 10 core principles covering code quality, security, testing, TDD, UX/UI, performance, Git practices, and code standards.
- **Added Sections**:
    - Core Principles (I-V)
    - Development Workflow
    - Non-Functional Requirements
    - Governance
- **Removed Sections**:
    - All placeholder sections from the initial template.
- **Templates Requiring Updates**:
    - ✅ `.specify/templates/plan-template.md` (No changes needed, aligns with new principles)
    - ✅ `.specify/templates/spec-template.md` (No changes needed, aligns with new principles)
    - ✅ `.specify/templates/tasks-template.md` (No changes needed, aligns with new principles)
- **Follow-up TODOs**:
    - None
-->
# Iced Lens Constitution

## Core Principles

### I. Code Quality and Functional Programming
All code must meet high standards of quality, emphasizing clarity, maintainability, and robustness. Where reasonable, functional programming principles (e.g., immutability, pure functions) should be adopted to enhance predictability and reduce side effects.

### II. Security
Security is a primary concern. All development must follow secure coding practices, including input validation, vulnerability scanning, and proactive mitigation of common threats. Dependencies must be regularly audited.

### III. Comprehensive Testing
All code must be accompanied by a suite of tests. For Rust projects, this includes unit, integration, and documentation tests (`cargo test`). The testing strategy must ensure correctness, reliability, and regression prevention.

### IV. Test-Driven Development (TDD)
A "test-first" approach is mandatory. For any new feature or bug fix, unit tests must be written first to define the requirements. Code is then developed to make the tests pass. The full cycle is: write failing tests, write code to pass tests, run tests, and refactor as needed.

### V. User-Centric Design
All user-facing features must be designed with modern UX/UI best practices in mind. The goal is a clean, elegant, and intuitive interface that prioritizes user needs and workflow efficiency.

## Development Workflow

**Version Control:** Git usage must follow established best practices. This includes a consistent branching model (e.g., GitFlow or a variant), descriptive commit messages, and regular, small commits.

**Code Verification:** Code must be regularly checked with `cargo check` and `cargo clippy`. Warnings must be addressed; any decision to ignore a warning requires explicit and strong justification.

**Code Comments:** All code comments must be written in English. They should explain the "why" behind a piece of logic, not the "what." The goal is to provide context that is not apparent from the code itself.

## Non-Functional Requirements

**Performance:** Performance is a critical requirement. Code should be written with efficiency in mind, and performance bottlenecks must be identified and addressed. Performance regressions are to be treated as bugs.

## Governance

This constitution is the guiding document for all development practices. All code reviews and contributions must ensure compliance with these principles. Deviations are not permitted without a formal amendment to this document.

**Versioning Policy:**
- **MAJOR**: Backward incompatible governance/principle removals or redefinitions.
- **MINOR**: New principle/section added or materially expanded guidance.
- **PATCH**: Clarifications, wording, typo fixes, non-semantic refinements.

**Version**: 1.0.0 | **Ratified**: 2025-11-21 | **Last Amended**: 2025-11-21