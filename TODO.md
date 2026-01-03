# TODO - Next Release

> This file tracks planned work for the next release. It lives only in the `dev` branch and is not included in releases.

## Bugs to Fix


## Changed

- [] Pour évitér les validations inutiles des modèles AI des outils d'édition d'image, les validation ne doivent plus se faire au lancement de l'pplication, mais la premuière fois (depuis le lancement de l'application) que l'utilisateur se rend dans l'éditeur d'image. Ca évitera une utilisation inetensive inutile du CPU surtout quand l'utiisateur ferme et réouvre l'application fréquemment.


## Planned Features

### Viewer

#### Media Filters for Navigation
- [ ] Add auto-focus between segmented date input fields (blocked, requires `text_input::focus(id)` Task API, expected in future iced versions)

#### Metadata Sidebar
- [ ] Allow text selection and copying in the metadata sidebar (blocked, pending native support in Iced 0.15.0)
- [ ] Add video metadata editing support

### Video Player



### Help
- [ ] Allow text selection and copying in the help screen (blocked, pending native support in Iced 0.15.0)

### Video Editor
- [ ] Create a simple video editor allowing users to trim videos by removing segments. The editor should let users play the video, seek to any position, step forward/backward frame by frame, and change the playback speed.

## Packaging / Distribution

### Flatpak
- [ ] Test Flatpak build locally with `flatpak-builder` (Waiting new Freedesktop SDK with support of Rust 1.92)
- [ ] Prepare Flathub submission:
  - [ ] Fork [flathub/flathub](https://github.com/flathub/flathub)
  - [ ] Create PR with manifest following [Flathub submission guidelines](https://docs.flathub.org/docs/for-app-authors/submission/)
  - [ ] Ensure app passes Flathub quality guidelines (no network at build time, proper permissions, etc.)

## Notes

- Test videos can be generated with `scripts/generate-test-videos.sh`
