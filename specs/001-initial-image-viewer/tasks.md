# Tasks: Initial Image Viewer and Application Foundations

**Input**: Design documents from `specs/001-initial-image-viewer/`
**Prerequisites**: plan.md, spec.md, data-model.md, research.md

## Phase 1: Setup (Shared Infrastructure)

- [x] T001 Initialize a new binary Rust project using `cargo new iced_lens --vcs git`
- [x] T002 Create the `rust-toolchain.toml` file to pin the Rust version (e.g., 1.78)
- [x] T003 [P] Add core dependencies to `Cargo.toml`: `iced`, `fluent-rs`, `image`, `resvg`, `toml`, `dirs`, `serde`
- [x] T004 [P] Create the source code directory structure as defined in `plan.md` (`src/ui`, `src/i18n`, etc.)
- [x] T005 [P] Create the `assets/i18n` directory and add initial `en-US.ftl` and `fr.ftl` translation files.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented.

- [x] T006 Create custom error types and result aliases in `src/error.rs`
- [x] T007 Implement the basic Iced application structure in `src/main.rs` and `src/app.rs` (state, message enum, view logic)
- [x] T008 [P] Implement the user preferences module in `src/config/mod.rs` to load and save `settings.toml`.
- [x] T009 [P] Implement the internationalization module in `src/i18n/fluent.rs` to load `.ftl` files and manage the active language.

---

## Phase 3: User Story 1 - View an Image (Priority: P1) 🎯 MVP

**Goal**: A user can open a supported image file and see it displayed in the application.
**Independent Test**: Launching the app with a file path argument should display the image correctly centered at 1:1 size.

### Tests for User Story 1
- [x] T010 [P] [US1] Write unit tests for the image handler in `src/image_handler/mod.rs` for both valid and corrupted image data.
- [x] T011 [P] [US1] Write an integration test in `tests/integration/` that simulates opening an image file and verifies the application state.

### Implementation for User Story 1
- [x] T012 [P] [US1] Implement the image loading and decoding logic in `src/image_handler/mod.rs`, using the `image` and `resvg` crates.
- [x] T013 [P] [US1] Implement the main image viewer UI component in `src/ui/viewer.rs`.
- [x] T014 [US1] Integrate the image handler into the main application state in `src/app.rs` to manage the currently displayed image.
- [x] T015 [US1] Handle command-line arguments in `src/main.rs` to open an image on startup.
- [x] T016 [US1] Implement the error handling UI to display a dialog for unsupported or corrupted files as per `FR-010`.

---

## Phase 4: User Story 2 - Use the App in a Preferred Language (Priority: P2)

**Goal**: The application UI appears in the user's preferred language.
**Independent Test**: The UI language should correctly follow the priority (CLI > settings > OS > default) and update dynamically.

### Tests for User Story 2
- [x] T017 [P] [US2] Write unit tests for the language resolution logic in `src/i18n/fluent.rs`.
- [ ] T018 [P] [US2] Write an integration test in `tests/integration/` to verify the UI language changes when the setting is modified.

### Implementation for User Story 2
- [x] T019 [US2] Implement the language resolution logic in `src/i18n/fluent.rs` (CLI > settings > OS > default).
- [ ] T020 [US2] Add a 'Settings' or 'Preferences' menu to the UI in `src/app.rs`.
- [ ] T021 [US2] Implement a language selection submenu that lists available languages.
- [ ] T022 [US2] Connect the language selection menu to the i18n module to update the UI text at runtime without a restart.
- [ ] T023 [US2] Update the `UserPreferences` in `src/config/mod.rs` when the user selects a new language.

---

## Phase 5: User Story 3 - Contribute Translations (Priority: P3)

**Goal**: A community member can easily add new translations.
**Independent Test**: Adding a new `.ftl` file to the correct directory makes it available in the language selection menu on next launch.

### Implementation for User Story 3
- [ ] T024 [US3] Ensure the translation loading logic in `src/i18n/fluent.rs` dynamically discovers all `.ftl` files in the translations directory.
- [ ] T025 [P] [US3] Add a section to the project's `README.md` or a new `CONTRIBUTING.md` file explaining the translation process.

---

## Phase 6: Polish & Cross-Cutting Concerns

- [ ] T026 [P] Add documentation tests (`#![doc(html_root_url = "https://docs.rs/iced_lens/0.1.0")]`) to all public modules and functions.
- [ ] T027 [P] Create a `tests/data` directory and populate it with sample images for each supported format.
- [ ] T028 Write the main `README.md` for the project.
- [ ] T029 Run `cargo clippy` and `cargo fmt` and address all warnings.
- [ ] T030 [P] Perform security audit of all dependencies using `cargo audit` and report findings.
- [ ] T031 [P] Create a benchmark test to measure image loading time and ensure it meets the <2s performance goal.

---

## Dependencies & Execution Order

- **Phase 1 & 2**: Must be completed before any user story work begins.
- **User Stories**:
  - **US1 (View Image)**: Can start after Phase 2. This is the MVP.
  - **US2 (Language)**: Depends on US1 for the UI shell.
  - **US3 (Contribute)**: Depends on US2 for the language selection mechanism.
- **Phase 6**: Can be done after all user stories are complete.
