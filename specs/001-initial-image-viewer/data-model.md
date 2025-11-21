# Data Model: Initial Image Viewer

This document outlines the key data entities for the initial version of IcedLens.

## 1. UserPreferences

Represents user-specific settings that persist between application sessions.

- **Storage**: Stored as a TOML file (`settings.toml`) in the standard, OS-specific user configuration directory. The application will use a library like `dirs` to resolve the correct path at runtime.
- **Format**: TOML

### Fields

| Field Name | Type | Description | Example |
| :--- | :--- | :--- | :--- |
| `language` | String | The user's preferred UI language, stored as a BCP 47 language tag. If not present, the application will attempt to auto-detect the language. | `"fr"` |

### Example `settings.toml`
```toml
# User-preferred language for the application UI.
# Uses BCP 47 language tags.
language = "fr"
```

## 2. Translation Bundle

Represents the collection of UI text for a single language.

- **Storage**: Stored as a plain text file with the `.ftl` extension. One file per language (e.g., `fr.ftl`). Default English and French bundles will be included in the application's `assets` directory.
- **Format**: Fluent (https://projectfluent.org)

### Example `fr.ftl`
```ftl
# Simple text for a window title
window-title = IcedLens

# Error message for a corrupted file
error-corrupted-file = Le fichier "{ $filename }" est corrompu ou illisible.

# Error message with an option to show details
error-unsupported-format =
    .title = Format non supporté
    .body = Le fichier "{ $filename }" n'est pas dans un format supporté.
    .details-button = Détails
```
