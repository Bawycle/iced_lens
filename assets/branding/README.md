# Application Branding

This directory contains the IcedLens application icon and branding assets.

## Files

- `iced_lens.svg` - Master scalable source (512x512). All raster exports originate here.
- `iced_lens-{size}.png` - Pre-rendered PNG icons at various sizes (16, 32, 48, 64, 128, 256, 512)
- `iced_lens.ico` - Windows executable icon (multi-size ICO format)
- `iced_lens.icns` - macOS application icon (multi-size ICNS format)

## Usage

- **Window icon**: The SVG is embedded in the binary and rasterized at runtime (see `src/icon.rs`)
- **Windows executable**: The ICO is embedded via `build.rs` using winresource
- **macOS bundle**: The ICNS is referenced in Info.plist when creating a .app bundle
- **Linux desktop integration**: Use PNG files with `.desktop` file

## Design Goals

- Convey a **lens** (viewer) with a subtle **crystalline / icy** inner motif.
- Use a warm yellow accent (#F2C94C) to add personality while remaining legible on light & dark themes.
- Keep geometry simple to preserve clarity at small sizes.

## Color Palette

| Role            | Hex       |
|-----------------|-----------|
| Accent Yellow   | `#F2C94C` / gradient variant `#F5D059` |
| Dark Outline    | `#2E3440` |
| Light Surface   | `#ECEFF4` |
| Mid Neutral     | `#D8DEE9` |
| Highlight       | `#FFFFFF` (semi-transparent) |

## Regenerating Icons

```bash
# Generate all PNG sizes from SVG
for size in 16 32 48 64 128 256 512; do
  rsvg-convert -w $size -h $size iced_lens.svg -o iced_lens-${size}.png
done

# Generate ICO for Windows (requires ImageMagick)
convert iced_lens-16.png iced_lens-32.png iced_lens-48.png \
        iced_lens-64.png iced_lens-128.png iced_lens-256.png \
        iced_lens.ico

# Generate ICNS for macOS (requires icnsutils: apt install icnsutils)
png2icns iced_lens.icns iced_lens-16.png iced_lens-32.png \
         iced_lens-48.png iced_lens-128.png iced_lens-256.png \
         iced_lens-512.png
```

## License

This icon is **not** under MPL-2.0. It uses a restricted license.
See `ICON_LICENSE.md` at the project root.

Summary (informative only; refer to the full text):
- May be redistributed **unmodified** solely to represent the IcedLens application.
- May not be altered, recolored, cropped, or used to represent another product/company.
- No derivative logos or brand usage.
- SPDX reference: `LicenseRef-IcedLens-Icon`.
