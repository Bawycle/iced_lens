# IcedLens: A Simple Image Viewer

IcedLens is a lightweight image viewer application built with the [Iced GUI framework](https://iced.rs/). It aims to provide a fast and intuitive experience for viewing various image formats, incorporating modern UI/UX principles and robust internationalization capabilities.

## Features

-   **Image Viewing**: Supports common image formats including JPEG, PNG, GIF, TIFF, WEBP, BMP, and ICO.
-   **SVG Support**: Displays Scalable Vector Graphics (SVG) files by rendering them to bitmaps.
-   **Internationalization (i18n)**: Dynamically loads translation files using the Fluent localization system, allowing the application to be easily localized into multiple languages. Users can switch languages at runtime.
-   **User Preferences**: Saves and loads user-specific settings, such as preferred language, to a configuration file.
-   **Modular Design**: Structured with a focus on modularity to facilitate future expansion with new features like image editing, video playback, and more.

## Getting Started

### Prerequisites

-   [Rust](https://www.rust-lang.org/tools/install) (version 1.78 or newer recommended).

### Building the Application

To build IcedLens, navigate to the project root and run:

```bash
cargo build --release
```

The executable will be located in `target/release/iced_lens`.

### Running the Application

You can run IcedLens directly from the command line:

```bash
cargo run
```

To open a specific image file, pass its path as an argument:

```bash
cargo run -- /path/to/your/image.png
```

You can also specify the language using the `--lang` argument:

```bash
cargo run -- --lang fr
```

### Running Tests

To execute all unit and integration tests, run:

```bash
cargo test
```

### Building Documentation

To generate and view the project's API documentation, run:

```bash
cargo doc --all-features --open
```

This will build the documentation and open it in your default web browser.

## Contributing

We welcome contributions! Please see our [CONTRIBUTING.md](./CONTRIBUTING.md) file for guidelines on how to contribute, especially regarding translations.

## License

This project is licensed under the [MIT License](LICENSE) (if there is a LICENSE file). If not, it falls under the typical open-source practices for Rust projects.
(Note: A `LICENSE` file should be added if not already present, with specific license details.)
