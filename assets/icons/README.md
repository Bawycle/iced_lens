# UI Icons

This directory contains the UI icons used throughout the IcedLens application.

## Directory Structure

```
icons/
├── source/           # SVG source files (not embedded in binary)
│   └── *.svg
└── png/
    ├── dark/         # Dark icons for light backgrounds
    │   └── *.png
    └── light/        # Light icons for dark backgrounds (overlays, HUD)
        └── *.png
```

## Icon Variants

- **dark/**: Standard dark icons used on light backgrounds (default UI)
- **light/**: Light (white) icons used on dark backgrounds (video overlays, HUD indicators)

## Naming Convention

Icons use generic visual names describing the icon's appearance, not the action context:
- `trash` not `delete_image`
- `play` not `start_video`
- `loop` not `repeat_playlist`

## Adding New Icons

1. Create the SVG source in `source/` (32x32 recommended)
2. Generate PNG versions:
   ```bash
   # Dark variant (for light backgrounds)
   rsvg-convert -w 32 -h 32 source/icon.svg -o png/dark/icon.png

   # Light variant (for dark backgrounds) - if needed
   rsvg-convert -w 32 -h 32 source/icon.svg | convert - -negate png/light/icon.png
   ```
3. Add the icon definition in `src/ui/icons.rs`

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
