# Application Branding

This directory contains the IcedLens application icon source.

## Files

- `iced_lens.svg` - Master scalable source (512x512). This is the **single source of truth** for all branding icons.

## Generated Files

All raster formats are generated automatically during `cargo build` by `build.rs`:

| File | Location | Usage |
|------|----------|-------|
| `iced_lens-{16,32,48,64,128,256,512}.png` | `target/branding/` | Linux desktop integration |
| `iced_lens.ico` | `target/branding/` | Windows executable icon |
| `iced_lens.icns` | `target/branding/` | macOS application icon |

## Usage

- **Window icon**: The SVG is embedded in the binary and rasterized at runtime (see `src/icon.rs`)
- **Windows executable**: The ICO is embedded via `build.rs` using winresource
- **macOS bundle**: The ICNS is referenced in Info.plist when creating a .app bundle
- **Linux desktop integration**: Use generated PNG files with `.desktop` file
- **AppImage/Flatpak**: Build scripts reference generated icons from `target/branding/`

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

Icons are regenerated automatically on every build. To manually trigger regeneration:

```bash
cargo build
# Generated icons are in target/branding/
ls target/branding/
```

The build system uses:
- `resvg` for SVG parsing and rendering
- `ico` crate for Windows ICO generation
- `icns` crate for macOS ICNS generation
- `image` crate for PNG encoding

## License

This icon is **not** under MPL-2.0. It uses a restricted license.
See `ICON_LICENSE.md` at the project root.

Summary (informative only; refer to the full text):
- May be redistributed **unmodified** solely to represent the IcedLens application.
- May not be altered, recolored, cropped, or used to represent another product/company.
- No derivative logos or brand usage.
- SPDX reference: `LicenseRef-IcedLens-Icon`.
