#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
#
# Build a Flatpak for IcedLens.
#
# This script automates the Flatpak build process:
# 1. Generates cargo-sources.json from Cargo.lock (vendored dependencies)
# 2. Builds the Flatpak using flatpak-builder
# 3. Optionally installs it locally for testing
#
# Prerequisites:
#   - flatpak and flatpak-builder installed
#   - Python 3 with aiohttp and toml packages (for cargo generator)
#   - Flathub repository added: flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo
#   - Runtime and SDK installed (script will prompt if missing)
#
# Usage:
#   ./scripts/build-flatpak.sh [OPTIONS]
#
# Options:
#   --install       Install the Flatpak locally after building
#   --run           Run the Flatpak after building (implies --install)
#   --skip-sources  Skip regenerating cargo-sources.json (use existing)
#   --clean         Force clean build (remove previous build directory)
#   --help          Show this help message

set -euo pipefail

# Configuration
ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
FLATPAK_DIR="$ROOT_DIR/flatpak"
BUILD_DIR="$ROOT_DIR/target/flatpak-build"
MANIFEST="$FLATPAK_DIR/page.codeberg.Bawycle.IcedLens.yml"
APP_ID="page.codeberg.Bawycle.IcedLens"
CARGO_SOURCES="$FLATPAK_DIR/cargo-sources.json"

# Runtime requirements
RUNTIME_VERSION="24.08"
RUNTIME="org.freedesktop.Platform//${RUNTIME_VERSION}"
SDK="org.freedesktop.Sdk//${RUNTIME_VERSION}"
RUST_EXT="org.freedesktop.Sdk.Extension.rust-stable//${RUNTIME_VERSION}"

# Default options
DO_INSTALL=0
DO_RUN=0
SKIP_SOURCES=0
FORCE_CLEAN=0

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

show_help() {
    head -30 "$0" | grep -E "^#" | sed 's/^# \?//'
    exit 0
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --install)
            DO_INSTALL=1
            shift
            ;;
        --run)
            DO_INSTALL=1
            DO_RUN=1
            shift
            ;;
        --skip-sources)
            SKIP_SOURCES=1
            shift
            ;;
        --clean)
            FORCE_CLEAN=1
            shift
            ;;
        --help|-h)
            show_help
            ;;
        *)
            log_error "Unknown option: $1"
            echo "Use --help for usage information."
            exit 1
            ;;
    esac
done

# Check prerequisites
log_info "Checking prerequisites..."

if ! command -v flatpak >/dev/null 2>&1; then
    log_error "flatpak is not installed. Please install it first."
    echo "  Ubuntu/Debian: sudo apt install flatpak"
    echo "  Fedora: sudo dnf install flatpak"
    echo "  Arch: sudo pacman -S flatpak"
    exit 1
fi

if ! command -v flatpak-builder >/dev/null 2>&1; then
    log_error "flatpak-builder is not installed. Please install it first."
    echo "  Ubuntu/Debian: sudo apt install flatpak-builder"
    echo "  Fedora: sudo dnf install flatpak-builder"
    echo "  Arch: sudo pacman -S flatpak-builder"
    exit 1
fi

# Check if Flathub remote is configured
if ! flatpak remote-list | grep -q flathub; then
    log_warn "Flathub remote not found. Adding it..."
    flatpak remote-add --if-not-exists --user flathub https://flathub.org/repo/flathub.flatpakrepo
    log_success "Flathub remote added."
fi

# Check and install runtime/SDK if needed
check_and_install_flatpak() {
    local ref="$1"
    local name="$2"
    if ! flatpak info "$ref" >/dev/null 2>&1; then
        log_warn "$name not found. Installing..."
        flatpak install --user -y flathub "$ref"
        log_success "$name installed."
    else
        log_success "$name is available."
    fi
}

check_and_install_flatpak "$RUNTIME" "Runtime (org.freedesktop.Platform $RUNTIME_VERSION)"
check_and_install_flatpak "$SDK" "SDK (org.freedesktop.Sdk $RUNTIME_VERSION)"
check_and_install_flatpak "$RUST_EXT" "Rust extension"

# Generate cargo-sources.json if needed
if [[ $SKIP_SOURCES -eq 0 ]] || [[ ! -f "$CARGO_SOURCES" ]]; then
    log_info "Generating cargo-sources.json from Cargo.lock..."

    # Check Python dependencies
    if ! python3 -c "import aiohttp, toml" 2>/dev/null; then
        log_warn "Python dependencies missing. Installing aiohttp and toml..."
        if ! pip3 install --user aiohttp toml; then
            log_error "Failed to install Python dependencies. Please install manually:"
            echo "  pip3 install --user aiohttp toml"
            exit 1
        fi
        # Verify installation succeeded
        if ! python3 -c "import aiohttp, toml" 2>/dev/null; then
            log_error "Python dependencies installed but cannot be imported."
            echo "  Try: pip3 install --user aiohttp toml"
            exit 1
        fi
    fi

    # Download flatpak-cargo-generator if not present
    CARGO_GENERATOR="$ROOT_DIR/target/flatpak-cargo-generator.py"
    if [[ ! -f "$CARGO_GENERATOR" ]]; then
        log_info "Downloading flatpak-cargo-generator.py..."
        mkdir -p "$ROOT_DIR/target"
        curl -sL "https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/master/cargo/flatpak-cargo-generator.py" \
            -o "$CARGO_GENERATOR"
        chmod +x "$CARGO_GENERATOR"
    fi

    # Generate the sources file
    python3 "$CARGO_GENERATOR" "$ROOT_DIR/Cargo.lock" -o "$CARGO_SOURCES"
    log_success "cargo-sources.json generated ($(wc -l < "$CARGO_SOURCES") lines)."
else
    log_info "Using existing cargo-sources.json (--skip-sources)"
fi

# Verify manifest exists
if [[ ! -f "$MANIFEST" ]]; then
    log_error "Manifest not found: $MANIFEST"
    exit 1
fi

# Clean build directory if requested
if [[ $FORCE_CLEAN -eq 1 ]] && [[ -d "$BUILD_DIR" ]]; then
    log_info "Cleaning previous build directory..."
    rm -rf "$BUILD_DIR"
fi

# Build the Flatpak
log_info "Building Flatpak..."
log_info "This may take a while on first build (downloading dependencies, compiling Rust)..."

BUILDER_ARGS=(
    --user
    --force-clean
    --install-deps-from=flathub
    --state-dir="$ROOT_DIR/target/flatpak-state"
    --repo="$ROOT_DIR/target/flatpak-repo"
)

if [[ $DO_INSTALL -eq 1 ]]; then
    BUILDER_ARGS+=(--install)
fi

flatpak-builder "${BUILDER_ARGS[@]}" "$BUILD_DIR" "$MANIFEST"

log_success "Flatpak build completed!"

# Show build artifacts
echo ""
log_info "Build artifacts:"
echo "  Build directory: $BUILD_DIR"
echo "  Repository: $ROOT_DIR/target/flatpak-repo"

if [[ $DO_INSTALL -eq 1 ]]; then
    log_success "Flatpak installed locally."
    echo ""
    echo "Run with: flatpak run $APP_ID"
    echo "Uninstall with: flatpak uninstall $APP_ID"
fi

# Run if requested
if [[ $DO_RUN -eq 1 ]]; then
    echo ""
    log_info "Starting $APP_ID..."
    flatpak run "$APP_ID"
fi

echo ""
log_info "Next steps for Flathub submission:"
echo "  1. Add screenshots to flatpak/page.codeberg.Bawycle.IcedLens.metainfo.xml"
echo "  2. Validate metadata: flatpak run org.freedesktop.appstream-glib validate flatpak/*.metainfo.xml"
echo "  3. Test all features in the Flatpak sandbox"
echo "  4. Read docs/FLATHUB_PUBLICATION.md for submission instructions"
