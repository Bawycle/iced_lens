# UI Icons

This directory contains the SVG source files for UI icons used throughout the IcedLens application.

## Directory Structure

```
icons/
└── source/           # SVG source files
    └── *.svg
```

PNG icons are **generated at build time** from these SVG sources via `build.rs`.
The generated PNGs are placed in `OUT_DIR/icons/` and embedded via `include_bytes!`.

## Icon Variants

Two variants are generated for each icon:

- **dark/**: Dark icons (black) for light backgrounds (default UI)
- **light/**: Light icons (white) for dark backgrounds (video overlays, HUD indicators)

Light variants are only generated for icons that need them (defined in `build.rs::needs_light_variant()`).

## Naming Convention

Icons use generic visual names describing the icon's appearance, not the action context:
- `trash` not `delete_image`
- `play` not `start_video`
- `loop` not `repeat_playlist`

## Adding New Icons

1. Create the SVG source in `source/` with:
   - ViewBox: `0 0 32 32`
   - Fill color: `white` (will be inverted for dark variant)

2. Add the icon definition in `src/ui/icons.rs`:
   ```rust
   define_icon!(icon_name, dark, "icon_name.png", "Description.");
   ```

3. If the icon needs a light variant (for overlays/dark backgrounds):
   - Add it to `build.rs::needs_light_variant()`
   - Add the light variant definition in `src/ui/icons.rs`:
     ```rust
     // In the light or overlay module
     define_icon!(icon_name, light, "icon_name.png", "Description (white).");
     ```

4. Run `cargo build` - the icon will be generated automatically.

## Application Icon

The application icon (logo) is located in `assets/branding/`.

## License

These icons are **not** under MPL-2.0. They use the same restricted license as the application icon.
See `ICON_LICENSE.md` at the project root.

Summary (informative only; refer to the full text):
- May be redistributed **unmodified** solely to represent the IcedLens application.
- May not be altered, recolored, cropped, or used to represent another product/company.
- No derivative logos or brand usage.
- SPDX reference: `LicenseRef-IcedLens-Icon`.
