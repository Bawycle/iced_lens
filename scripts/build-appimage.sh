#!/usr/bin/env bash
# Build an AppImage for iced_lens so testers can run a single portable binary with the right assets.
# Default artifact path: target/release/iced_lens-<version>-<arch>.AppImage (override with --output-dir or APPIMAGE_OUTPUT_DIR).
set -euo pipefail

# Keep all intermediate artifacts under target/ to avoid dirtying the repo tree.
ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
TARGET_DIR="$ROOT_DIR/target"
BUILD_DIR="$TARGET_DIR/appimage"
APPDIR="$BUILD_DIR/AppDir"
BIN_NAME="iced_lens"
LINUXDEPLOY_BIN=${LINUXDEPLOY_BIN:-${LINUXDEPLOY:-linuxdeploy}}
TARGET_TRIPLE=${TARGET_TRIPLE:-}
APPIMAGE_ARCH=${APPIMAGE_ARCH:-}
# Bundle GTK dependencies (required by the rfd file-dialog crate) unless explicitly disabled.
APPIMAGE_BUNDLE_GTK=${APPIMAGE_BUNDLE_GTK:-1}
# Default GTK major version to deploy (can be overridden through DEPLOY_GTK_VERSION or APPIMAGE_GTK_VERSION)
if [[ -z "${DEPLOY_GTK_VERSION:-}" ]]; then
  DEPLOY_GTK_VERSION=${APPIMAGE_GTK_VERSION:-3}
fi
export DEPLOY_GTK_VERSION
# Default AppImage output goes under target/release so CI artifacts stay with cargo builds.
OUTPUT_DIR=${APPIMAGE_OUTPUT_DIR:-$TARGET_DIR/release}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --target)
      TARGET_TRIPLE="$2"
      shift 2
      ;;
    --linuxdeploy|--linuxdeploy-bin)
      LINUXDEPLOY_BIN="$2"
      shift 2
      ;;
    --output-dir)
      OUTPUT_DIR="$2"
      shift 2
      ;;
    --help|-h)
      cat <<'USAGE'
Usage: scripts/build-appimage.sh [--target <triple>] [--linuxdeploy <path>] [--output-dir <dir>]

Environment overrides:
  LINUXDEPLOY_BIN Path to linuxdeploy binary (preferred)
  LINUXDEPLOY     Legacy env alias for linuxdeploy path
  LINUXDEPLOY_PLUGIN_GTK Path to linuxdeploy GTK plugin (auto-detected when available)
  APPIMAGE_BUNDLE_GTK Set to 0 to skip invoking the GTK plugin (default 1)
  APPIMAGE_GTK_VERSION Default GTK major version to bundle (fallback 3, forwarded to DEPLOY_GTK_VERSION)
  DEPLOY_GTK_VERSION GTK major version understood by linuxdeploy-plugin-gtk (overrides APPIMAGE_GTK_VERSION)
  TARGET_TRIPLE Rust target triple to cross-compile
  APPIMAGE_ARCH Architecture label for output filename/AppImage metadata
  APPIMAGE_OUTPUT_DIR Destination directory for final AppImage (default target/release)
USAGE
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo is required but not found in PATH" >&2
  exit 1
fi

if ! command -v "$LINUXDEPLOY_BIN" >/dev/null 2>&1; then
  echo "linuxdeploy not found. Set the LINUXDEPLOY env var to its path." >&2
  exit 1
fi

# Use the Cargo version to guarantee the AppImage filename matches published builds.
VERSION=$(awk -F '"' '/^version = "[0-9]+\./ {print $2; exit}' "$ROOT_DIR/Cargo.toml")
if [[ -z "$VERSION" ]]; then
  VERSION="dev"
fi

if [[ -z "$APPIMAGE_ARCH" ]]; then
  if [[ -n "$TARGET_TRIPLE" ]]; then
    APPIMAGE_ARCH="${TARGET_TRIPLE%%-*}"
  else
    APPIMAGE_ARCH="$(uname -m)"
  fi
fi

OUTPUT_NAME="${BIN_NAME}-${VERSION}-${APPIMAGE_ARCH}.AppImage"
OUTPUT_PATH="$OUTPUT_DIR/$OUTPUT_NAME"

CARGO_BUILD_ARGS=(--release)
if [[ -n "$TARGET_TRIPLE" ]]; then
  CARGO_BUILD_ARGS+=(--target "$TARGET_TRIPLE")
fi

# Release build ensures optimized binary is packaged just like production releases.
cargo build "${CARGO_BUILD_ARGS[@]}"

if [[ -n "$TARGET_TRIPLE" ]]; then
  BIN_PATH="$TARGET_DIR/$TARGET_TRIPLE/release/$BIN_NAME"
else
  BIN_PATH="$TARGET_DIR/release/$BIN_NAME"
fi

if [[ ! -x "$BIN_PATH" ]]; then
  echo "Built binary not found at $BIN_PATH" >&2
  exit 1
fi

rm -rf "$BUILD_DIR"
mkdir -p "$APPDIR/usr/bin" \
         "$APPDIR/usr/share/$BIN_NAME/assets" \
         "$APPDIR/usr/share/icons/hicolor/scalable/apps" \
         "$APPDIR/usr/share/applications"

install -m 755 "$BIN_PATH" "$APPDIR/usr/bin/$BIN_NAME"

# Ship translations with the bundle because users may launch the AppImage offline.
I18N_SRC="$ROOT_DIR/assets/i18n"
I18N_DEST="$APPDIR/usr/share/$BIN_NAME/assets/i18n"
if [[ ! -d "$I18N_SRC" ]]; then
  echo "Missing translations directory: $I18N_SRC" >&2
  exit 1
fi
mkdir -p "$I18N_DEST"
cp -a "$I18N_SRC/." "$I18N_DEST/"

# Desktop environments discover the app icon through the standard hicolor path.
ICON_SRC="$ROOT_DIR/assets/icons/iced_lens.svg"
ICON_DEST="$APPDIR/usr/share/icons/hicolor/scalable/apps/iced_lens.svg"
if [[ ! -f "$ICON_SRC" ]]; then
  echo "Missing icon: $ICON_SRC" >&2
  exit 1
fi
install -m 644 "$ICON_SRC" "$ICON_DEST"

# Copy custom icon license where linuxdeploy expects dpkg-style copyright info.
DOC_DIR="$APPDIR/usr/share/doc/$BIN_NAME"
mkdir -p "$DOC_DIR"
if [[ -f "$ROOT_DIR/ICON_LICENSE.md" ]]; then
  cp "$ROOT_DIR/ICON_LICENSE.md" "$DOC_DIR/copyright"
fi

DESKTOP_FILE="$APPDIR/usr/share/applications/iced_lens.desktop"
# Provide a .desktop file so the AppImage integrates with menus when registered.
cat >"$DESKTOP_FILE" <<'EOF'
[Desktop Entry]
Type=Application
Name=Iced Lens
Comment=Minimal image viewer built with Iced
Exec=iced_lens %F
Icon=iced_lens
Categories=Graphics;Viewer;
Terminal=false
EOF

# Custom AppRun injects --i18n-dir so the binary loads bundled translations instead of host files.
APPRUN="$APPDIR/AppRun"
cat >"$APPRUN" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
APPDIR=$(cd "$(dirname "$0")" && pwd)
I18N_DIR=${ICED_LENS_I18N_DIR:-"$APPDIR/usr/share/iced_lens/assets/i18n"}
exec "$APPDIR/usr/bin/iced_lens" --i18n-dir "$I18N_DIR" "$@"
EOF
chmod +x "$APPRUN"

# Run linuxdeploy inside the build dir so its output lands next to AppDir for easy cleanup.
pushd "$BUILD_DIR" >/dev/null
export ARCH="$APPIMAGE_ARCH" # AppImage tooling reads ARCH to label metadata correctly.
GTK_PLUGIN_ARGS=()
if [[ "$APPIMAGE_BUNDLE_GTK" -ne 0 ]]; then
  if [[ -z "${LINUXDEPLOY_PLUGIN_GTK:-}" ]]; then
    if command -v linuxdeploy-plugin-gtk.sh >/dev/null 2>&1; then
      export LINUXDEPLOY_PLUGIN_GTK="$(command -v linuxdeploy-plugin-gtk.sh)"
    elif command -v linuxdeploy-plugin-gtk >/dev/null 2>&1; then
      export LINUXDEPLOY_PLUGIN_GTK="$(command -v linuxdeploy-plugin-gtk)"
    fi
  elif [[ ! -x "${LINUXDEPLOY_PLUGIN_GTK}" ]]; then
    echo "linuxdeploy GTK plugin specified in LINUXDEPLOY_PLUGIN_GTK is not executable" >&2
    exit 1
  fi

  if [[ -n "${LINUXDEPLOY_PLUGIN_GTK:-}" ]]; then
    GTK_PLUGIN_ARGS+=(--plugin gtk)
  else
    echo "Warning: linuxdeploy GTK plugin not found; GTK deps required by rfd dialogs may be missing" >&2
  fi
fi

"$LINUXDEPLOY_BIN" --appdir "$APPDIR" \
  --desktop-file "$DESKTOP_FILE" \
  --icon-file "$ICON_DEST" \
  "${GTK_PLUGIN_ARGS[@]}" \
  --output appimage
NEW_APPIMAGE=$(find "$BUILD_DIR" -maxdepth 1 -type f -name "*.AppImage" -print -quit)
popd >/dev/null

if [[ -z "$NEW_APPIMAGE" ]]; then
  echo "linuxdeploy did not produce an AppImage" >&2
  exit 1
fi

mkdir -p "$OUTPUT_DIR"
mv "$NEW_APPIMAGE" "$OUTPUT_PATH"
echo "AppImage created at $OUTPUT_PATH"

# Generate SHA256 checksum for integrity verification
CHECKSUM_FILE="${OUTPUT_PATH}.sha256"
if command -v sha256sum >/dev/null 2>&1; then
  (cd "$OUTPUT_DIR" && sha256sum "$(basename "$OUTPUT_PATH")") > "$CHECKSUM_FILE"
  echo "SHA256 checksum: $CHECKSUM_FILE"
elif command -v shasum >/dev/null 2>&1; then
  # macOS fallback
  (cd "$OUTPUT_DIR" && shasum -a 256 "$(basename "$OUTPUT_PATH")") > "$CHECKSUM_FILE"
  echo "SHA256 checksum: $CHECKSUM_FILE"
else
  echo "Warning: sha256sum not found, checksum file not generated" >&2
fi
