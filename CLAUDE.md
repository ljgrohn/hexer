# hexer

Windows-native color picker and palette helper. Point at any pixel, get its HEX / RGB / HSL, one-click
copy, and generate coordinated theme colors.

## Stack

- Rust 2024, `eframe`/`egui` 0.36 for the window, `windows` 0.62 for Win32 (GetPixel, GetCursorPos, RegisterHotKey, GetAsyncKeyState).
- No other deps. Clipboard goes through `egui::Context::copy_text`.

## Layout

- `src/main.rs` - app state, picker/theme tabs, swatches, and copy actions. Note eframe 0.36 uses `App::ui(&mut self, ui, frame)`, not `update`.
- `src/win.rs` - all `unsafe` Win32 lives here. The hotkey loop runs on its own thread and wakes the UI through a channel.
- `src/color.rs` - `Rgb` formatting, HSL conversion, harmony generation, and tests.

## Behaviour

- Global hotkey `Ctrl+Shift+C` (or the Pick button) enters pick mode.
- In pick mode the swatch follows the cursor live. Left click locks the color and copies HEX. Esc cancels.
- Each format row and theme swatch copies directly and flashes "Copied" for about 1.2 seconds.
- The Theme tab derives analogous, complementary, and triadic options from the current pick.

## Commands

- `cargo run` - dev build (console attached).
- `cargo build --release` - small stripped exe, no console.
- `cargo test` - color math tests.

## Design intent

Dark layered cards, rounded corners, monospace codes, and a clear accent. It should feel like a small,
polished utility (the user cited "mrkdup" as the look they want). Keep it to one window and no settings.
