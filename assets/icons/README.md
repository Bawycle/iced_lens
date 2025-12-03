# Icon Assets

This directory contains the vector source for the IcedLens application icon.

## Design Goals
- Convey a **lens** (viewer) with a subtle **crystalline / icy** inner motif.
- Use a warm yellow accent (#F2C94C) to add personality while remaining legible on light & dark themes.
- Keep geometry simple to preserve clarity at small sizes.
- Provide a neutral base that can adapt if future theming adds a dark or high-contrast variant.

## Files
- `iced_lens.svg` – Master scalable source (512×512). All raster exports should originate here.

## Recommended Raster Sizes
Generate PNGs for common desktop integration targets:
- 16×16, 24×24, 32×32 (toolbars, small lists)
- 48×48, 64×64 (application menus)
- 128×128, 256×256 (high‑DPI launcher grids)
- 512×512 (storefronts / marketing)

## Color Palette
| Role            | Hex      |
|-----------------|----------|
| Accent Yellow   | `#F2C94C` / gradient variant `#F5D059` |
| Dark Outline    | `#2E3440` |
| Light Surface   | `#ECEFF4` |
| Mid Neutral     | `#D8DEE9` |
| Highlight       | `#FFFFFF` (semi‑transparent) |

## Export Script (Suggested)
A helper script can automate PNG generation (requires `rsvg-convert` or `inkscape`). Place in `scripts/generate-icons.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail
SRC="assets/icons/iced_lens.svg"
OUT="assets/icons"
SIZES=(16 24 32 48 64 128 256 512)
for s in "${SIZES[@]}"; do
  rsvg-convert -w "$s" -h "$s" "$SRC" -o "$OUT/iced_lens_${s}.png" || \
  inkscape "$SRC" --export-type=png -w "$s" -h "$s" -o "$OUT/iced_lens_${s}.png"
  echo "Generated $OUT/iced_lens_${s}.png"
done
```

Make executable:
```bash
chmod +x scripts/generate-icons.sh
```
Run:
```bash
./scripts/generate-icons.sh
```

## Desktop Integration
Add a `.desktop` file later (e.g. `dist/iced_lens.desktop`):
```ini
[Desktop Entry]
Type=Application
Name=IcedLens
Exec=iced_lens
Icon=iced_lens
Categories=Graphics;Viewer;
```
Install icon PNGs under appropriate XDG icon theme directories if packaging.

## Future Variants
- Monochrome (outline only) for status indicators.
- High contrast version (swap yellow for pure white accent on dark backgrounds).

## License
This icon is **not** under MPL-2.0. It uses a restricted license: see `ICON_LICENSE.md` at the project root.

Summary (informative only; refer to the full text):
- May be redistributed **unmodified** solely to represent the IcedLens application.
- May not be altered, recolored, cropped, or used to represent another product/company.
- No derivative logos or brand usage.
- SPDX reference: `LicenseRef-IcedLens-Icon`.
