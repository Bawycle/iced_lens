#!/usr/bin/env bash
# Convert all SVG icons to PNG format for consistent cross-platform rendering.
# Uses rsvg-convert (preferred) or falls back to Inkscape.
set -euo pipefail

ICONS_DIR="assets/icons"
PNG_DIR="assets/icons/png"
SIZE=32  # Standard size for UI icons

# Create output directory
mkdir -p "$PNG_DIR"

# Check for conversion tool
if command -v rsvg-convert >/dev/null 2>&1; then
    CONVERTER="rsvg"
elif command -v inkscape >/dev/null 2>&1; then
    CONVERTER="inkscape"
else
    echo "Error: Neither rsvg-convert nor inkscape found." >&2
    echo "Install librsvg2-bin (Debian/Ubuntu) or inkscape to convert SVGs." >&2
    exit 1
fi

echo "Using converter: $CONVERTER"
echo "Converting SVG icons to ${SIZE}x${SIZE} PNG..."

# Convert each SVG (except the app icon which has its own sizes)
for svg in "$ICONS_DIR"/*.svg; do
    filename=$(basename "$svg" .svg)

    # Skip the main app icon (it has dedicated sizes already)
    if [[ "$filename" == "iced_lens" ]]; then
        continue
    fi

    output="$PNG_DIR/${filename}.png"

    if [[ "$CONVERTER" == "rsvg" ]]; then
        rsvg-convert -w "$SIZE" -h "$SIZE" "$svg" -o "$output"
    else
        inkscape "$svg" --export-type=png -w "$SIZE" -h "$SIZE" -o "$output"
    fi

    echo "  Converted: $filename.svg -> $filename.png"
done

echo ""
echo "Done! PNG icons saved to $PNG_DIR/"
echo "Total icons converted: $(ls -1 "$PNG_DIR"/*.png 2>/dev/null | wc -l)"
