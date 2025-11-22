#!/usr/bin/env bash
# Generate raster PNG icons from the master SVG.
# Falls back to Inkscape if rsvg-convert not available.
set -euo pipefail
SRC="assets/icons/iced_lens.svg"
OUT="assets/icons"
SIZES=(16 24 32 48 64 128 256 512)
if [[ ! -f "$SRC" ]]; then
  echo "Source SVG not found: $SRC" >&2
  exit 1
fi
for s in "${SIZES[@]}"; do
  if command -v rsvg-convert >/dev/null 2>&1; then
    rsvg-convert -w "$s" -h "$s" "$SRC" -o "$OUT/iced_lens_${s}.png"
  elif command -v inkscape >/dev/null 2>&1; then
    inkscape "$SRC" --export-type=png -w "$s" -h "$s" -o "$OUT/iced_lens_${s}.png"
  else
    echo "Neither rsvg-convert nor inkscape found. Install one to export PNGs." >&2
    exit 2
  fi
  echo "Generated $OUT/iced_lens_${s}.png"
done
