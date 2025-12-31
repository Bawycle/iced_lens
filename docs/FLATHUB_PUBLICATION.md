# Publishing IcedLens on Flathub

This guide explains the complete procedure for publishing IcedLens as a Flatpak on [Flathub](https://flathub.org), from local testing to submission and maintenance.

## Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Local Development](#local-development)
4. [Testing the Flatpak](#testing-the-flatpak)
5. [Preparing for Submission](#preparing-for-submission)
6. [Flathub Submission Process](#flathub-submission-process)
7. [Post-Submission Maintenance](#post-submission-maintenance)
8. [Troubleshooting](#troubleshooting)

---

## Overview

### What is Flathub?

Flathub is the central repository for Flatpak applications. Publishing on Flathub makes IcedLens available to millions of Linux users through a single, sandboxed installation.

### IcedLens Flatpak Details

| Property | Value |
|----------|-------|
| App ID | `page.codeberg.Bawycle.IcedLens` |
| Runtime | `org.freedesktop.Platform 24.08` |
| SDK | `org.freedesktop.Sdk 24.08` |
| Extensions | Rust stable, FFmpeg full |

### File Structure

```
flatpak/
├── page.codeberg.Bawycle.IcedLens.yml        # Main manifest
├── page.codeberg.Bawycle.IcedLens.desktop    # Desktop entry
├── page.codeberg.Bawycle.IcedLens.metainfo.xml  # AppStream metadata
└── cargo-sources.json                         # Generated vendored deps

scripts/
└── build-flatpak.sh                          # Build automation script
```

---

## Prerequisites

### System Requirements

Install these packages on your Linux system:

```bash
# Ubuntu/Debian
sudo apt install flatpak flatpak-builder python3-pip

# Fedora
sudo dnf install flatpak flatpak-builder python3-pip

# Arch Linux
sudo pacman -S flatpak flatpak-builder python-pip
```

### Python Dependencies

The cargo source generator requires:

```bash
pip3 install --user aiohttp toml
```

### Flathub Repository

Add the Flathub remote if not already configured:

```bash
flatpak remote-add --if-not-exists --user flathub https://flathub.org/repo/flathub.flatpakrepo
```

### Runtime and SDK

Install the required runtime, SDK, and extensions:

```bash
# Base runtime and SDK
flatpak install --user flathub org.freedesktop.Platform//24.08
flatpak install --user flathub org.freedesktop.Sdk//24.08

# Rust extension for building
flatpak install --user flathub org.freedesktop.Sdk.Extension.rust-stable//24.08

# FFmpeg extension (installed automatically, but can be pre-installed)
flatpak install --user flathub org.freedesktop.Platform.ffmpeg-full//24.08
```

---

## Local Development

### Quick Build

Use the provided script for a streamlined build process:

```bash
# Build only
./scripts/build-flatpak.sh

# Build and install locally
./scripts/build-flatpak.sh --install

# Build, install, and run immediately
./scripts/build-flatpak.sh --run

# Force clean build
./scripts/build-flatpak.sh --clean --install
```

### Manual Build Steps

If you prefer manual control:

#### 1. Generate Cargo Sources

The manifest requires `cargo-sources.json` with vendored Cargo dependencies (Flathub has no network access during builds):

```bash
# Download the generator (one-time)
curl -sL "https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/master/cargo/flatpak-cargo-generator.py" \
    -o target/flatpak-cargo-generator.py

# Generate sources from Cargo.lock
python3 target/flatpak-cargo-generator.py Cargo.lock -o flatpak/cargo-sources.json
```

**Important**: Regenerate `cargo-sources.json` whenever `Cargo.lock` changes.

#### 2. Build the Flatpak

```bash
flatpak-builder --user --force-clean \
    --install-deps-from=flathub \
    --state-dir=target/flatpak-state \
    --repo=target/flatpak-repo \
    target/flatpak-build \
    flatpak/page.codeberg.Bawycle.IcedLens.yml
```

#### 3. Install Locally

```bash
flatpak-builder --user --install --force-clean \
    --state-dir=target/flatpak-state \
    --repo=target/flatpak-repo \
    target/flatpak-build \
    flatpak/page.codeberg.Bawycle.IcedLens.yml
```

#### 4. Run the Flatpak

```bash
flatpak run page.codeberg.Bawycle.IcedLens

# With a specific file
flatpak run page.codeberg.Bawycle.IcedLens ~/Pictures/photo.jpg
```

---

## Testing the Flatpak

### Essential Tests

Before submission, verify all features work in the sandbox:

| Feature | Test Command/Action |
|---------|---------------------|
| **Image viewing** | Open JPEG, PNG, GIF, WebP, SVG files |
| **Video playback** | Open MP4, MKV, WebM files |
| **Audio** | Verify video sound works |
| **File dialogs** | Use Open File (Ctrl+O) and Save As (Ctrl+Shift+S) |
| **Image editing** | Rotate, crop, resize, adjust brightness |
| **Undo/Redo** | Verify history works (Ctrl+Z, Ctrl+Y) |
| **AI features** | Test deblur and upscaling (downloads models on first use) |
| **Metadata** | View and edit EXIF/XMP metadata |
| **Fullscreen** | Toggle fullscreen (F11) |
| **Language** | Switch languages in Settings |

### Debugging

Access a shell inside the sandbox:

```bash
flatpak run --command=sh page.codeberg.Bawycle.IcedLens
```

Inside the shell:

```bash
# Check ONNX Runtime is installed
ls -la /app/lib/libonnxruntime.so*

# Check FFmpeg extension
ls -la /app/lib/ffmpeg/

# Verify binary linkage
ldd /app/bin/iced_lens | grep -E "(onnx|av)"

# Check environment
env | grep -E "(XDG|HOME|FLATPAK)"
```

### Verify Permissions

```bash
flatpak info --show-permissions page.codeberg.Bawycle.IcedLens
```

Expected permissions:
- `--socket=wayland`, `--socket=fallback-x11`, `--share=ipc`
- `--device=dri` (GPU)
- `--socket=pulseaudio` (audio)
- `--share=network` (AI model downloads)
- `--filesystem=xdg-pictures`, `--filesystem=xdg-videos`, etc.
- Portal access for file dialogs and settings

### Validate Metadata

```bash
# Validate AppStream metainfo
flatpak run org.freedesktop.appstream-glib validate \
    flatpak/page.codeberg.Bawycle.IcedLens.metainfo.xml

# Validate desktop file
desktop-file-validate flatpak/page.codeberg.Bawycle.IcedLens.desktop
```

Fix any errors before submission.

---

## Preparing for Submission

### 1. Add Screenshots

Screenshots are **required** for Flathub. They must be:
- Hosted at stable, publicly accessible URLs
- At least 1248×702 pixels (16:9 recommended)
- PNG or JPEG format

Create a `docs/screenshots/` directory and add images:

```bash
mkdir -p docs/screenshots
# Add screenshot files: viewer.png, editor.png, video.png, etc.
```

Host them on Codeberg (raw URLs) or another stable host, then update the metainfo.xml:

```xml
<screenshots>
  <screenshot type="default">
    <image>https://codeberg.org/Bawycle/iced_lens/raw/branch/master/docs/screenshots/viewer.png</image>
    <caption>Image viewer with zoom and navigation controls</caption>
  </screenshot>
  <!-- Add more screenshots -->
</screenshots>
```

### 2. Update Release Information

Ensure `metainfo.xml` has the correct release version and date:

```xml
<releases>
  <release version="0.5.0" date="2025-12-22">
    <description>
      <p>Release description...</p>
    </description>
  </release>
</releases>
```

### 3. Verify App ID Consistency

The App ID must be identical across all files:

- Manifest filename: `page.codeberg.Bawycle.IcedLens.yml`
- Manifest `app-id:` field
- Desktop file: `page.codeberg.Bawycle.IcedLens.desktop`
- Metainfo `<id>`: `page.codeberg.Bawycle.IcedLens`
- Icon filenames: `page.codeberg.Bawycle.IcedLens.svg`

### 4. Final Validation Checklist

- [ ] `cargo-sources.json` is up to date with current `Cargo.lock`
- [ ] All screenshots added and URLs work
- [ ] Release version matches `Cargo.toml` version
- [ ] Metadata validates without errors
- [ ] Desktop file validates without errors
- [ ] All features tested in sandbox
- [ ] GitHub/Codeberg 2FA enabled on your account

---

## Flathub Submission Process

### Step 1: Fork the Flathub Repository

1. Go to https://github.com/flathub/flathub
2. Click **Fork** (keep all branches, don't copy only master)
3. Clone your fork:

```bash
git clone --branch new-pr https://github.com/YOUR_USERNAME/flathub.git
cd flathub
```

**Important**: Work from the `new-pr` branch, not `master`.

### Step 2: Create Your App Branch

```bash
git checkout -b page.codeberg.Bawycle.IcedLens
```

### Step 3: Add Your Files

Copy the manifest and metadata files:

```bash
# From the IcedLens repository
cp /path/to/iced_lens/flatpak/page.codeberg.Bawycle.IcedLens.yml .
cp /path/to/iced_lens/flatpak/page.codeberg.Bawycle.IcedLens.desktop .
cp /path/to/iced_lens/flatpak/page.codeberg.Bawycle.IcedLens.metainfo.xml .
cp /path/to/iced_lens/flatpak/cargo-sources.json .
```

**Note**: For Flathub submission, the manifest should reference the Git repository URL, not local files. Update the manifest:

```yaml
sources:
  - type: git
    url: https://codeberg.org/Bawycle/iced_lens.git
    tag: v0.5.0
    commit: <full-commit-hash>  # Required for reproducibility
```

### Step 4: Commit and Push

```bash
git add .
git commit -m "Add page.codeberg.Bawycle.IcedLens"
git push -u origin page.codeberg.Bawycle.IcedLens
```

### Step 5: Create Pull Request

1. Go to https://github.com/flathub/flathub
2. Click **New Pull Request**
3. Select:
   - Base repository: `flathub/flathub`
   - Base branch: `new-pr` (NOT master!)
   - Head repository: `YOUR_USERNAME/flathub`
   - Compare branch: `page.codeberg.Bawycle.IcedLens`
4. Title: `Add page.codeberg.Bawycle.IcedLens`
5. Fill in the PR template completely
6. Submit the PR

### Step 6: Review Process

- **Reviewers** are volunteers; be patient and responsive
- **Address feedback** promptly and push fixes to your branch
- When ready for a test build, a reviewer will comment `bot, build`
- **Build logs** appear in the PR; fix any failures

### Step 7: Acceptance

Once approved:
1. A new repository is created: `github.com/flathub/page.codeberg.Bawycle.IcedLens`
2. You receive an **invitation** to become a maintainer
3. **Accept within one week** (requires GitHub 2FA)
4. The app appears on Flathub within hours of merge

---

## Post-Submission Maintenance

### Updating the App

For new releases:

1. Update `Cargo.toml` version
2. Update `CHANGELOG.md`
3. Create a Git tag: `git tag v0.6.0`
4. Push the tag: `git push origin v0.6.0`
5. In the Flathub repository:
   - Update the manifest with new tag and commit hash
   - Regenerate `cargo-sources.json`
   - Update `metainfo.xml` with new release entry
   - Push to master (auto-builds and publishes)

### Updating ONNX Runtime

When `ort` crate updates its ONNX Runtime requirement:

1. Check the new version at https://github.com/pykeio/ort
2. Download new releases and calculate checksums:

```bash
# For x86_64
curl -sL "https://github.com/microsoft/onnxruntime/releases/download/vX.Y.Z/onnxruntime-linux-x64-X.Y.Z.tgz" | sha256sum

# For aarch64
curl -sL "https://github.com/microsoft/onnxruntime/releases/download/vX.Y.Z/onnxruntime-linux-aarch64-X.Y.Z.tgz" | sha256sum
```

3. Update the manifest with new URLs and checksums

### Updating Runtime Version

When Freedesktop releases a new runtime (e.g., 25.08):

1. Update `runtime-version` in manifest
2. Update extension versions
3. Test thoroughly before pushing

---

## Troubleshooting

### Build Fails: Missing Cargo Dependencies

**Symptom**: `cargo build` fails with unresolved dependencies.

**Solution**: Regenerate `cargo-sources.json`:

```bash
python3 target/flatpak-cargo-generator.py Cargo.lock -o flatpak/cargo-sources.json
```

### Build Fails: ONNX Runtime Not Found

**Symptom**: `ort` crate fails to link.

**Solution**: Verify the ONNX Runtime module installs correctly:

```bash
flatpak run --command=sh page.codeberg.Bawycle.IcedLens
ls -la /app/lib/libonnxruntime*
```

The library should be in `/app/lib/`. Check the manifest's ONNX Runtime module.

### Runtime Error: FFmpeg Not Found

**Symptom**: Video playback fails, FFmpeg libraries missing.

**Solution**: Ensure FFmpeg extension is configured in manifest:

```yaml
add-extensions:
  org.freedesktop.Platform.ffmpeg-full:
    version: '24.08'
    directory: lib/ffmpeg
    add-ld-path: .
```

And the mount point is created:

```yaml
build-commands:
  - install -d /app/lib/ffmpeg
```

### File Dialogs Don't Work

**Symptom**: Open/Save dialogs don't appear or crash.

**Solution**: Verify portal permissions:

```yaml
finish-args:
  - --talk-name=org.freedesktop.portal.FileChooser
```

Check that `xdg-desktop-portal` and a backend (GTK, GNOME, KDE) are installed on the host.

### AI Models Fail to Download

**Symptom**: Upscaling/deblurring fails with network error.

**Solution**: Ensure network permission is granted:

```yaml
finish-args:
  - --share=network
```

Check connectivity:

```bash
flatpak run --command=sh page.codeberg.Bawycle.IcedLens
curl -I https://huggingface.co
```

### Metadata Validation Errors

**Symptom**: `appstream-glib validate` reports errors.

**Common fixes**:
- Ensure all required tags are present (`id`, `name`, `summary`, `description`, `launchable`, `releases`)
- Screenshot URLs must be HTTPS and accessible
- Release dates must be in YYYY-MM-DD format
- Description must have at least one `<p>` tag

---

## Resources

- [Flathub Submission Guidelines](https://docs.flathub.org/docs/for-app-authors/submission)
- [Flathub App Requirements](https://docs.flathub.org/docs/for-app-authors/requirements)
- [Flatpak Documentation](https://docs.flatpak.org/)
- [AppStream Metadata Specification](https://freedesktop.org/software/appstream/docs/)
- [flatpak-cargo-generator](https://github.com/flatpak/flatpak-builder-tools/tree/master/cargo)
- [ort crate documentation](https://ort.pyke.io/)

---

## Quick Reference

### Common Commands

```bash
# Build and install
./scripts/build-flatpak.sh --install

# Run
flatpak run page.codeberg.Bawycle.IcedLens

# Debug shell
flatpak run --command=sh page.codeberg.Bawycle.IcedLens

# Uninstall
flatpak uninstall page.codeberg.Bawycle.IcedLens

# View permissions
flatpak info --show-permissions page.codeberg.Bawycle.IcedLens

# Validate metadata
flatpak run org.freedesktop.appstream-glib validate flatpak/*.metainfo.xml

# Regenerate cargo sources
python3 target/flatpak-cargo-generator.py Cargo.lock -o flatpak/cargo-sources.json
```

### Flathub PR Checklist

- [ ] Manifest validates: `flatpak-builder --show-manifest`
- [ ] App builds locally
- [ ] All features work in sandbox
- [ ] Desktop file validates
- [ ] AppStream metadata validates
- [ ] Screenshots included and accessible
- [ ] PR targets `new-pr` branch
- [ ] PR title: `Add page.codeberg.Bawycle.IcedLens`
- [ ] GitHub 2FA enabled
