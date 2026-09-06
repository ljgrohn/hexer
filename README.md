# hexer

<p align="center">
  <img src="assets/hexer.png" width="96" height="96" alt="Hexer logo">
</p>

Polished Windows-native color picker and palette helper. Point at any pixel, get its HEX / RGB / HSL,
then turn it into a coordinated color theme.

Built in Rust with `eframe`/`egui` for the window and the `windows` crate for Win32.

## Download

Download the latest `hexer-*-windows-x86_64.zip` from [GitHub Releases](https://github.com/ljgrohn/hexer/releases/latest),
extract it, and run `hexer.exe`. Hexer is portable and does not need to be installed.

Windows may show a SmartScreen warning because release builds are not yet code-signed. Only download
Hexer from this repository's Releases page.

## Usage

```
cargo run              # dev build
cargo build --release  # small stripped exe, no console window
cargo test             # color math tests
```

## Releasing

1. Update `version` in `Cargo.toml` and commit the change.
2. Create and push a matching tag, such as `v0.1.0`.
3. GitHub Actions tests and builds Hexer, then creates a GitHub Release containing the portable ZIP
   and its SHA-256 checksum.

The release workflow can also be run manually from the Actions tab. Manual runs create a downloadable
workflow artifact for testing, but do not publish a GitHub Release.

Press **Ctrl + Shift + C** (or the Pick button) to enter pick mode. The swatch follows your cursor live.
Left click locks the color and copies the HEX. Esc cancels. Each row has its own Copy box.

Open the **Theme** tab to see analogous, complementary, and triadic palettes generated from the current
color. Click any palette swatch to copy its HEX value, or use **Pick new** to sample another base color.

## Features

- Global `Ctrl + Shift + C` picker with mixed-DPI support
- Live pixel preview with click-to-lock and Esc-to-cancel
- One-click HEX, RGB, and HSL copy actions
- Analogous, complementary, and triadic theme suggestions
- Useful palette output even for very dark, light, or desaturated source colors
