# hexer

Super simple Windows-native color picker. Point at any pixel, get its HEX / RGB / HSL, one-click copy.

## Stack
- Rust 2024, `eframe`/`egui` 0.36 for the window, `windows` 0.62 for Win32 (GetPixel, GetCursorPos, RegisterHotKey, GetAsyncKeyState).
- No other deps. Clipboard goes through `egui::Context::copy_text`.

## Layout
- `src/main.rs` – the app: state, swatch card UI, copy rows, pick button. Note eframe 0.36 uses `App::ui(&mut self, ui, frame)`, not `update`.
- `src/win.rs` – all `unsafe` Win32 lives here. Hotkey loop runs on its own thread and pokes the UI via a channel + `request_repaint`.
- `src/color.rs` – `Rgb` newtype with hex/rgb/hsl formatting and tests.

## Behaviour
- Global hotkey `Ctrl+Shift+C` (or the Pick button) enters pick mode.
- In pick mode the swatch follows the cursor live. Left click locks the color and copies HEX. Esc cancels.
- Each row has a Copy button that flashes "Copied" for ~1.2s.

## Commands
- `cargo run` – dev build (console attached).
- `cargo build --release` – small stripped exe, no console.
- `cargo test` – color math tests.

## Design intent
Dark card, rounded corners, monospace codes, a clear accent. Meant to feel like a small polished utility
(the user cited "mrkdup" as the look they want). Keep it to one window and no settings.
