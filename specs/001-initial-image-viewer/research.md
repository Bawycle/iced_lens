# Research: Image Decoding Libraries

## Decision

- For raster image formats (`jpeg`, `png`, `gif`, `tiff`, `webp`, `bmp`, `ico`), the project will use the `image` crate.
- For the vector image format (`svg`), the project will use the `resvg` crate.

## Rationale

The user input specified that image decoding libraries must be chosen carefully, considering performance, security, and target platforms, and that using different libraries for different needs is acceptable.

- **`image` crate**: This is the most comprehensive and widely-used image library in the Rust ecosystem. It supports all the required raster formats through a system of feature flags, which allows us to compile support only for the formats we need. It is actively maintained by the `image-rs` organization and has a strong focus on safety and performance, especially in recent versions. Its broad support and maturity make it a reliable choice.

- **`resvg` crate**: The `image` crate does not handle SVG rendering, as SVG is a vector format that must be rasterized. `resvg` is a pure-Rust, high-quality SVG rendering library that is perfect for this task. It provides the functionality to take SVG data and render it to a pixel buffer, which can then be displayed in the Iced UI. Using a dedicated library for SVG ensures high-quality rendering and separates the concerns of raster vs. vector graphics.

This two-library approach provides the best combination of broad format support for raster images and high-quality, specialized rendering for vector images, aligning perfectly with the project's requirements.

## Alternatives Considered

- **Using a single library for everything**: No single, high-quality Rust library was found that handles both raster formats and SVG rendering.
- **Using C libraries with Rust bindings**: While libraries like `ImageMagick` or `librsvg` (the C library) could be used via FFI, the goal is to prefer pure-Rust solutions where possible for better security, portability, and easier compilation. `image` and `resvg` are both pure Rust.
