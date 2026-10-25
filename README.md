# Lotpix

**Bulk image optimizer**: a desktop application to convert, compress and resize images in batches, written in Rust with [Iced](https://iced.rs). Available in English, French, Spanish and German.

[![Build and Release](https://github.com/yoanmarchal/lotpix/actions/workflows/release.yml/badge.svg)](https://github.com/yoanmarchal/lotpix/actions/workflows/release.yml)

> **Were you using Image Converter (version 1, Electron)?** Lotpix is its complete rewrite, under a new name: it does not update from version 1. Uninstall the old version, then install the new one from the [GitHub releases](https://github.com/yoanmarchal/lotpix/releases). The conversion history is not carried over.

## Features

- 🖼️ **Multi-format image conversion**
  - Output formats: WebP (best quality/size ratio), JPEG (compatible everywhere), PNG (lossless, with transparency), AVIF (modern high-performance format)
  - "Original format" to only resize or recompress (GIF and TIFF are then saved as PNG)
  - Input formats: JPEG, PNG, GIF, TIFF, WebP
  - EXIF orientation applied: photos taken in portrait stay in portrait
  - Animated GIFs kept as animated WebP
  - Transparency replaced by a white background in JPEG

- 📐 **Batch resizing** (optional)
  - By width, height, longest side or percentage, always keeping the proportions
  - "Do not enlarge smaller images" option, checked by default
  - Final dimensions of each image shown before the conversion

- ⚙️ **Conversion options**
  - Quality control (1 to 100%, except PNG which is lossless)
  - Converted files next to the originals, or in an output folder of your choice
  - No file is overwritten: if `photo.webp` already exists, the output becomes `photo-1.webp`

- 🎯 **Interface**
  - Drag and drop images or folders, or open them from the file manager ("Open with")
  - Thumbnails, progress of each conversion, size saved
  - Dark/light mode
  - In English, French, Spanish or German: the system language by default, or the one chosen in the header. Sizes, percentages and dates follow the conventions of the language

- 🖱️ **Context menu** (Windows Explorer, Dolphin, Nemo)
  - On images or folders (subfolders included), without opening a window, with the recommended quality for each format (WebP 80, JPEG 82, AVIF 65, lossless PNG). Files are written next to the originals, and a notification sums up the saving
  - "Compress with Lotpix": WebP, JPEG, PNG, AVIF or original format
  - "Resize with Lotpix": standard sizes (4K, Full HD, HD, Web, Thumbnail, 50%), keeping the original format, or social network formats (Instagram square and portrait, Story/Reels/TikTok, Facebook/LinkedIn, X, YouTube thumbnail, Pinterest), resized then cropped to the center at the exact size and saved as JPEG. An image smaller than the frame is cropped without being enlarged
  - "Open in Lotpix" to choose other settings before converting
  - On Windows 11, the entries are under "Show more options" (or Shift + right-click). The labels use the language chosen at installation. Nemo has no submenus, so its "Compress…" and "Resize…" entries let you pick the format or the size in a list (they need zenity, installed by default on Cinnamon)
  - From the command line: `lotpix compress --format <format>`, `lotpix resize --size <size>`, or `lotpix compress` alone for the interface settings (`lotpix --help` lists the sizes)

- 📊 **Conversion history**
  - The last 100 conversions, with size and dimensions before and after
  - Quick access to the converted files

- 🔔 **New versions announced**
  - At startup, the application checks the GitHub releases in the background and shows a banner if a newer version exists
  - The "Download" button opens the release page: installation stays manual
  - Can be turned off with the "Check for updates" checkbox at the bottom of the window; it is the only network access of the application

## Built with

- Rust
- [Iced](https://iced.rs) for the interface, without an embedded browser
- [`image`](https://crates.io/crates/image), [`webp`](https://crates.io/crates/webp), [`jpeg-encoder`](https://crates.io/crates/jpeg-encoder) and [`ravif`](https://crates.io/crates/ravif) for the codecs
- [`fast_image_resize`](https://crates.io/crates/fast_image_resize) for resizing
- [Fluent](https://projectfluent.org) ([`i18n-embed`](https://crates.io/crates/i18n-embed)) for the translations
- [`cargo-packager`](https://crates.io/crates/cargo-packager) for the installers

## Development

Requirements: stable [Rust](https://rustup.rs) (edition 2024, so Rust 1.85 or later). On Windows, the Visual Studio C++ build tools are needed to compile libwebp.

```bash
git clone https://github.com/yoanmarchal/lotpix.git
cd lotpix
cargo run --release
```

## Available commands

- `cargo run --release` - Runs the application
- `cargo test` - Runs the tests; the rendering of each screen is also written to `target/captures/`, in light and dark and in each language
- `cargo fmt` and `cargo clippy --all-targets` - Formats and checks the code
- `cargo packager --release --formats nsis` - Builds the Windows installer in `release/`
- `cargo packager --release --formats deb,appimage` - Builds the Linux packages in `release/`
- `cargo run --release --example compare -- <output folder> <images…>` - Measures the time and output size of each format
- `LOTPIX_UPDATE_MENUS=1 cargo test` - Regenerates the context menus (NSIS block of `Cargo.toml`, Dolphin and Nemo files) after a change to the translations or the presets; without the variable, `cargo test` fails if they are out of date

Both packaging commands need `cargo install cargo-packager --locked`.

## Code layout

- `src/conversion.rs` - Conversion and resizing, independent of the interface
- `src/batch.rs`, `src/queue.rs` - Batches of files and folders, and grouping of the processes started together by Windows Explorer
- `src/settings.rs`, `src/history.rs`, `src/preferences.rs`, `src/storage.rs` - Data saved in the user's folder
- `src/i18n.rs`, `i18n/<language>/lotpix.ftl` - Application language and [Fluent](https://projectfluent.org) translations; English is the reference, and the Spanish and German translations still need a review by native speakers
- `src/format.rs` - Formatting of sizes, percentages and dates in each language
- `src/presets.rs`, `src/cli.rs`, `src/context_menu.rs` - Context menu presets and command line, and generation of the context menu files
- `src/update.rs` - Check for a newer version in the GitHub releases
- `src/ui/` - Iced interface: state and messages (`mod.rs`), theme (`theme.rs`), screens (`convert.rs`, `history.rs`)
- `tests/` - Conversion and batch tests on real images

## Translations

Each language is a [Fluent](https://projectfluent.org) file in `i18n/<language>/lotpix.ftl`. To fix a translation, edit the file of the language, then run `LOTPIX_UPDATE_MENUS=1 cargo test` to update the context menus. To add a language, copy `i18n/en/lotpix.ftl` and add the language to `src/i18n.rs`: a test checks that every language has exactly the keys of the English file.

## Publishing a version

Change `version` in `Cargo.toml` and merge into `main`: the workflow creates the `vX.Y.Z` tag and the GitHub release, and uploads the Windows and Linux installers to it.

On Windows, the workflow waits for two approvals in SignPath (the application, then the installer), for at most one hour each. This needs the `SIGNPATH_API_TOKEN` secret and the `SIGNPATH_ORGANIZATION_ID`, `SIGNPATH_PROJECT_SLUG` and `SIGNPATH_SIGNING_POLICY_SLUG` variables in the repository settings; without the secret, the installer is published unsigned. The artifact configurations to create in SignPath are in [`.signpath/artifact-configurations/`](.signpath/artifact-configurations/), named `executable` and `installer`.

## System requirements

### Windows
- Windows 10 or later
- x64 architecture

### Linux
- Ubuntu 22.04 or later, or an equivalent distribution
- x64 or arm64 architecture

## Code signing policy

Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by [SignPath Foundation](https://signpath.org).

The Windows executables (`lotpix.exe` and the installer) are built and signed by the [`release.yml`](.github/workflows/release.yml) GitHub Actions workflow, from the code of this repository. Each signature is approved by hand before it is applied.

Team:
- Committers and reviewers: [Yoan Marchal](https://github.com/yoanmarchal)
- Approvers: [Yoan Marchal](https://github.com/yoanmarchal)

### Privacy

Lotpix processes images on your computer: no image, file name or usage data is sent. The only network access is the check for new versions at startup, which reads the address of the latest release page on GitHub; it can be turned off with the "Check for updates" checkbox at the bottom of the window. This request is subject to the [GitHub privacy statement](https://docs.github.com/site-policy/privacy-policies/github-general-privacy-statement).

## License

[MIT](LICENSE) © Yoan Marchal. The [Inter](https://rsms.me/inter/) font is distributed under the SIL Open Font License (`assets/fonts/Inter-LICENSE.txt`).
