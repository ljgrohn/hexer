# hexer

Super simple Windows-native color picker. Point at any pixel, get its HEX / RGB / HSL, one-click copy.

Built in Rust with `eframe`/`egui` for the window and the `windows` crate for Win32.

## Usage

```
cargo run              # dev build
cargo build --release  # small stripped exe, no console window
cargo test             # color math tests
```

Press **Ctrl + Shift + C** (or the Pick button) to enter pick mode. The swatch follows your cursor live.
Left click locks the color and copies the HEX. Esc cancels. Each row has its own Copy box.

## Status

Scaffolded and building. The Win32 side is done (per-monitor DPI awareness, off-screen pixel detection,
ignoring clicks that land on hexer's own always-on-top window). The UI polish and a real end-to-end
pick test are still to do. See the prompt below to pick up where things left off.

## Polish prompt

Paste this into Claude Code (or your agent of choice) from the repo root:

> Read CLAUDE.md, then src/main.rs, src/win.rs, src/color.rs. This is a super simple Windows color picker
> in Rust (eframe/egui 0.36 + windows 0.62). It builds and tests pass. Finish and polish it:
>
> 1. Run it with `cargo run` and actually exercise it: press Ctrl+Shift+C, move the mouse, click, press
>    each Copy button. Verify the picked pixel matches a known solid color on screen and that the
>    clipboard holds the right string after each copy. Fix anything broken.
> 2. Test picking on a scaled monitor (125% / 150%) and on a second monitor if available. The cursor
>    position and the sampled pixel must agree. `win::set_dpi_aware` exists; make sure it is called before
>    the window is created.
> 3. Handle the pick-mode edge cases: require a button-up before accepting the locking click so a click
>    that was already held when the hotkey fired does not lock immediately, and ignore clicks that land on
>    our own window (`win::point_over_own_window` exists for this).
> 4. Make the UI look polished and intentional: a clean dark card with rounded corners, a large swatch
>    with a subtle 1px inset stroke, monospace codes, compact Copy boxes aligned to the right of each code
>    with a soft hover state and a short "Copied" flash. One window, no settings, no extra features.
>    No magnifier or history unless it is trivial and does not clutter.
> 5. Confirm `cargo build --release` produces a working exe with no console window and `cargo test`
>    still passes.
>
> Report what you changed file by file, what you verified by running it, and anything you could not verify.
