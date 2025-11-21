# Contributing to IcedLens

We welcome contributions to IcedLens! This document outlines how you can help improve the project.

## Translations

IcedLens uses Fluent for its internationalization (i18n). You can help translate the application into new languages or improve existing translations.

### How to Add or Update Translations

1.  **Locate Translation Files**: All translation files are located in the `assets/i18n/` directory.
2.  **Naming Convention**: Translation files use the `.ftl` (Fluent Translation List) extension and are named according to their [Language Identifier](https://docs.rs/unic-langid/latest/unic_langid/struct.LanguageIdentifier.html) (e.g., `en-US.ftl` for American English, `fr.ftl` for French).
    *   If you're adding a new language, create a new `.ftl` file with the appropriate language identifier (e.g., `es.ftl` for Spanish).
    *   If you're updating an existing translation, modify the corresponding `.ftl` file.
3.  **Translation Keys**:
    *   Translation keys are used in the application code to retrieve localized strings. For example, `window-title` is a key for the application's window title.
    *   You can find existing keys in `en-US.ftl` or `fr.ftl`.
    *   When adding new keys, try to use descriptive names.
4.  **Fluent Syntax**: Translations are written using [Fluent syntax](https://projectfluent.org/fluent/guide/). It's a powerful and easy-to-learn language for natural-sounding translations, including pluralization, gender, and variations.

    Example of Fluent syntax:
    ```fluent
    window-title = IcedLens Image Viewer
    language-name-en-US = English
    language-name-fr = Français
    ```

5.  **Test Your Changes (Optional but Recommended)**: If you have a Rust development environment set up, you can run the application with your new translation to see it in action:
    ```bash
    cargo run -- --lang <your-locale-id>
    ```
    For example: `cargo run -- --lang es`

6.  **Submit Your Contribution**: Once you're happy with your translation, please submit a Pull Request to the main repository.

## Code Contributions

(To be added later)
