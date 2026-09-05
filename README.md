# hexer

Polished Windows-native color picker and palette helper. Point at any pixel, get its HEX / RGB / HSL,
then turn it into a coordinated color theme.

Built in Rust with `eframe`/`egui` for the window and the `windows` crate for Win32.

## Usage

```
cargo run              # dev build
cargo build --release  # small stripped exe, no console window
cargo test             # color math tests
```

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
