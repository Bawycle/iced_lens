# Contributing to IcedLens

Thank you for your interest in contributing to IcedLens! We welcome contributions of all kinds: bug reports, feature suggestions, documentation improvements, translations, and code contributions.

## Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [How Can I Contribute?](#how-can-i-contribute)
3. [Reporting Bugs](#reporting-bugs)
4. [Suggesting Features](#suggesting-features)
5. [Translation Contributions](#translation-contributions)
6. [Code Contributions](#code-contributions)
7. [Development Workflow](#development-workflow)
8. [Pull Request Process](#pull-request-process)

## Code of Conduct

This project adheres to a Code of Conduct that all contributors are expected to follow. Please read [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) before participating.

## How Can I Contribute?

There are many ways to contribute to IcedLens:

- **Report bugs** you encounter while using the application
- **Suggest new features** or improvements to existing ones
- **Translate** the interface into new languages
- **Improve documentation** (README, code comments, examples)
- **Submit code** for bug fixes or new features
- **Review pull requests** from other contributors
- **Share feedback** on your user experience

## Reporting Bugs

Before submitting a bug report:
1. Check the [issue tracker](https://codeberg.org/Bawycle/iced_lens/issues) to see if the issue has already been reported
2. Ensure you're using the latest version of IcedLens
3. Verify the bug is reproducible

When submitting a bug report, please include:
- **Operating System** (name and version, e.g., "Linux Mint 22.2", "macOS 14.0", "Windows 11")
- **IcedLens version** (from `--help` output or release version)
- **Steps to reproduce** the issue (be as specific as possible)
- **Expected behavior** vs. **actual behavior**
- **Logs or error messages** (if applicable)
- **Sample image** (if the issue is image-specific)

## Suggesting Features

Feature suggestions are welcome! Before opening a feature request:
1. Check if a similar feature request already exists
2. Consider whether the feature aligns with the project's goals (lightweight, privacy-focused image viewing and editing)

When suggesting a feature, please:
- Describe the **problem** the feature would solve
- Explain **why** this feature would be useful
- Provide **examples** or **mockups** if applicable
- Discuss potential **implementation approaches** (if you have ideas)

## Translation Contributions

IcedLens uses [Fluent](https://projectfluent.org/) for internationalization. Contributing translations is a great way to help make IcedLens accessible to more users worldwide.

**You don't need to be a developer to contribute translations!** The process is simple and accessible to anyone.

### How to Add or Update Translations

1.  **Locate Translation Files**: All translation files are in the `assets/i18n/` directory in the repository.

2.  **Naming Convention**: Translation files use the `.ftl` extension and are named according to their language code:
    - `en-US.ftl` for American English
    - `fr.ftl` for French
    - `es.ftl` for Spanish (example for a new language)
    - `de.ftl` for German (example for a new language)

3.  **Create or Edit Translation File**:
    - **For a new language**:
      1. Download or view the [`en-US.ftl`](assets/i18n/en-US.ftl) file as a reference
      2. Create a new file named after your language code (e.g., `pt-BR.ftl` for Brazilian Portuguese)
      3. Copy all the keys from `en-US.ftl` and translate the values
    - **For updates to an existing language**:
      1. Find and edit the corresponding `.ftl` file (e.g., `fr.ftl` for French)

4.  **Translation Format**: Each line follows this simple pattern:
    ```fluent
    key-name = Translated text here
    ```

    **Example** (comparing English and French):
    ```fluent
    # English (en-US.ftl)
    window-title = IcedLens Image Viewer
    zoom-in = Zoom In
    zoom-out = Zoom Out

    # French (fr.ftl)
    window-title = Visionneuse d'images IcedLens
    zoom-in = Zoom avant
    zoom-out = Zoom arrière
    ```

5.  **Important Translation Tips**:
    - **Keep the key names unchanged** (the part before `=`)
    - Only translate the text after the `=` sign
    - Preserve special placeholders like `{$variable}` if you see them
    - Maintain the same line structure as the original file
    - Don't worry if you're unsure about technical terms—we'll help during review!

6.  **Testing Your Translation** (optional):

    **Option A: If you have IcedLens installed**
    - Download a [release binary](https://codeberg.org/Bawycle/iced_lens/releases) for your system
    - Place your `.ftl` file in a custom directory (e.g., `/home/user/my_translations/`)
    - Run IcedLens with the custom translation directory:
      ```bash
      iced_lens --i18n-dir /home/user/my_translations/ --lang <your-language-code>
      ```
      Example: `iced_lens --i18n-dir /home/user/my_translations/ --lang es`

    **Option B: If you're a developer with Rust installed**
    - Use the development environment:
      ```bash
      cargo run -- --lang <your-language-code> /path/to/image.png
      ```

    **Option C: Submit without testing**
    - If testing isn't possible for you, that's perfectly fine! Submit your translation and the maintainers will test it for you.

7.  **Submit Your Translation**:
    - **Via Pull Request** (if you're familiar with Git/Codeberg):
      1. Fork the repository
      2. Add or modify the `.ftl` file in `assets/i18n/`
      3. Commit your changes
      4. Open a Pull Request

    - **Via Issue** (if you're not familiar with Git):
      1. Open a [new issue](https://codeberg.org/Bawycle/iced_lens/issues/new)
      2. Title: "Translation: [Language Name]"
      3. Attach your `.ftl` file or paste its contents
      4. We'll handle adding it to the repository for you!

### Translation Questions?

If you have any questions about translating, feel free to:
- Open an issue asking for clarification
- Check the existing translation files for examples
- Ask in your Pull Request—we're here to help!

## Code Contributions

Code contributions should follow the project's development practices and quality standards.

### Prerequisites

- **Rust 1.78 or newer** (install via [rustup](https://rustup.rs/))
- Familiarity with the [Iced GUI framework](https://iced.rs/)
- Understanding of the project structure (see below)

### Before You Start

1. **Open an issue** to discuss your proposed changes (unless it's a trivial fix)
2. **Wait for feedback** from maintainers to ensure alignment with project goals
3. **Fork the repository** and create a feature branch from `dev`

### Code Quality Standards

IcedLens follows strict quality standards to maintain code quality and reliability:

#### Test-Driven Development (TDD)

**Tests should be written before or alongside implementation code.** This ensures:
- Features work as expected from the start
- Changes don't break existing functionality
- Code is maintainable and well-documented

The TDD cycle:
1. Write tests that define expected behavior
2. Write code to make tests pass
3. Run `cargo test` to verify
4. Refactor while keeping tests green

#### Code Style

- Run `cargo fmt --all` before committing to format code consistently
- Run `cargo clippy --all --all-targets -- -D warnings` and fix all warnings
- Use English for all code comments and documentation
- Comments should explain **why**, not **what** (the code shows what)
- Favor clear, readable code over clever tricks

#### Testing Requirements

All code should include appropriate tests:
- **Unit tests** for individual functions/modules (`#[cfg(test)]` modules)
- **Integration tests** for multi-component workflows (`tests/` directory)
- **Documentation tests** for public APIs (examples in doc comments)

#### Security

- Follow secure coding practices
- Validate all user inputs (file paths, zoom values, etc.)
- Use proper error handling (avoid `unwrap()` on user-provided data)
- Run `cargo audit` to check for vulnerable dependencies

### Development Workflow

```bash
# Fork and clone the repository
git clone https://codeberg.org/YourUsername/iced_lens.git
cd iced_lens

# Create a feature branch from dev
git checkout dev
git checkout -b feature/your-feature-name

# Make changes following TDD:
# 1. Write tests first (or alongside implementation)
# 2. Implement feature
# 3. Ensure tests pass
cargo test

# Check code quality
cargo clippy --all --all-targets -- -D warnings
cargo fmt --all

# Build release version for testing
cargo build --release

# Run the application
./target/release/iced_lens /path/to/image.png

# Commit with descriptive messages
git add .
git commit -m "feat: Add descriptive commit message"

# Push to your fork
git push origin feature/your-feature-name
```

### Commit Message Guidelines

Follow conventional commits format for clarity:

- `feat: Add new feature description`
- `fix: Fix bug description`
- `docs: Update documentation`
- `test: Add tests for X`
- `refactor: Refactor component Y`
- `perf: Improve performance of Z`
- `chore: Update dependencies`

## Pull Request Process

1. **Ensure all tests pass**: `cargo test`
2. **Ensure code quality checks pass**: `cargo clippy --all --all-targets -- -D warnings`
3. **Format your code**: `cargo fmt --all`
4. **Update documentation** if needed (README.md, CHANGELOG.md, code comments)
5. **Provide a clear PR description**:
   - What problem does this solve?
   - How does it solve it?
   - Are there any breaking changes?
   - Screenshots (for UI changes)
6. **Reference related issues**: Use "Fixes #123" or "Relates to #456"
7. **Be responsive** to feedback and review comments
8. **Keep PRs focused**: One feature or fix per PR (split large changes into smaller PRs)

### PR Checklist

- [ ] Tests written and passing (`cargo test`)
- [ ] Clippy warnings addressed (`cargo clippy --all --all-targets -- -D warnings`)
- [ ] Code formatted (`cargo fmt --all`)
- [ ] Documentation updated (if applicable)
- [ ] CHANGELOG.md updated (for notable changes)
- [ ] Commit messages follow conventional commits format
- [ ] PR description is clear and complete

## Project Structure

Key files and directories:

```
iced_lens/
├── src/
│   ├── app.rs              # Main application logic and state
│   ├── config/             # Configuration persistence
│   ├── i18n/               # Internationalization system
│   ├── image_handler/      # Image loading and decoding
│   ├── ui/                 # UI components (viewer, settings, editor)
│   ├── error.rs            # Error types
│   └── icon.rs             # Application icon loading
├── assets/
│   ├── i18n/               # Translation files (.ftl)
│   └── icons/              # Application icons
├── tests/                  # Integration tests
├── benches/                # Performance benchmarks
├── CONTRIBUTING.md         # This file
├── CHANGELOG.md            # Release notes
├── README.md               # User-facing documentation
└── Cargo.toml              # Project metadata and dependencies
```

## Getting Help

- Read the [README.md](README.md) for user documentation
- Check existing [issues](https://codeberg.org/Bawycle/iced_lens/issues)
- Open a new issue for questions or discussion

---

Thank you for contributing to IcedLens!
