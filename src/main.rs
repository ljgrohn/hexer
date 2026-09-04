#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod color;
mod win;

use std::sync::mpsc::{Receiver, channel};
use std::time::{Duration, Instant};

use color::Rgb;
use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Stroke, Vec2};
use win::HotkeyEvent;

/// Behind the card.
const SHELL_BG: Color32 = Color32::from_rgb(0x14, 0x14, 0x17);
/// The card itself.
const CARD_BG: Color32 = Color32::from_rgb(0x1C, 0x1C, 0x21);
/// One code row.
const ROW_BG: Color32 = Color32::from_rgb(0x23, 0x23, 0x2A);
const BTN_BG: Color32 = Color32::from_rgb(0x2E, 0x2E, 0x37);
const BTN_HOVER: Color32 = Color32::from_rgb(0x3C, 0x3C, 0x48);
const CARD_FG: Color32 = Color32::from_rgb(0xEC, 0xEC, 0xF0);
const CARD_MUTED: Color32 = Color32::from_rgb(0x86, 0x86, 0x92);
const CARD_LINE: Color32 = Color32::from_rgb(0x2A, 0x2A, 0x31);
const ACCENT: Color32 = Color32::from_rgb(0x5B, 0x9C, 0xFF);
const ACCENT_HOVER: Color32 = Color32::from_rgb(0x74, 0xAD, 0xFF);
const ACCENT_ACTIVE: Color32 = Color32::from_rgb(0x46, 0x88, 0xEB);

const WINDOW: [f32; 2] = [340.0, 344.0];
const SWATCH_H: f32 = 104.0;
const ROW_H: f32 = 36.0;
const LABEL_W: f32 = 34.0;
const COPY_W: f32 = 58.0;

fn main() -> eframe::Result {
    // Must happen before any window (or any GetCursorPos/GetPixel) exists.
    win::set_dpi_aware();

    let viewport = egui::ViewportBuilder::default()
        .with_title("hexer")
        .with_inner_size(WINDOW)
        .with_min_inner_size(WINDOW)
        .with_resizable(false)
        .with_always_on_top();
    let options = eframe::NativeOptions { viewport, ..Default::default() };
    eframe::run_native("hexer", options, Box::new(|cc| Ok(Box::new(Hexer::new(cc)))))
}

struct Hexer {
    color: Rgb,
    picking: bool,
    /// Pick mode ignores the mouse until it has seen the button *up* once, so a
    /// click that is still in flight when the hotkey fires cannot lock a color.
    armed: bool,
    /// Which row was just copied, and when, for the "Copied" flash.
    copied: Option<(usize, Instant)>,
    hotkeys: Receiver<HotkeyEvent>,
}

impl Hexer {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (tx, rx) = channel();
        let ctx = cc.egui_ctx.clone();
        std::thread::spawn(move || win::hotkey_loop(tx, move || ctx.request_repaint()));

        cc.egui_ctx.set_visuals(visuals());
        cc.egui_ctx.all_styles_mut(|s| {
            s.spacing.item_spacing = Vec2::new(8.0, 8.0);
            s.spacing.button_padding = Vec2::new(10.0, 4.0);
        });

        Self {
            color: Rgb([0x5B, 0x9C, 0xFF]),
            picking: false,
            armed: false,
            copied: None,
            hotkeys: rx,
        }
    }

    fn start_pick(&mut self, ctx: &egui::Context) {
        self.picking = true;
        self.armed = false;
        // The hotkey is global, so the card may well be minimised when it fires.
        // Bring it back: a picker that silently does nothing is worse than useless.
        // (We deliberately do not steal keyboard focus - picking reads the mouse
        // and Esc through GetAsyncKeyState, so focus buys us nothing.)
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
    }

    /// Drive pick mode: sample the pixel under the cursor, and decide whether the
    /// mouse button state should lock it in.
    fn poll_picker(&mut self, ctx: &egui::Context) {
        while let Ok(HotkeyEvent::Pick) = self.hotkeys.try_recv() {
            self.start_pick(ctx);
        }
        if !self.picking {
            return;
        }
        if win::escape_down() {
            self.picking = false;
            return;
        }

        let (x, y) = win::cursor_pos();
        // Our card is always-on-top, so it is regularly the thing under the
        // cursor. Sampling it would just show our own chrome, and clicking it
        // should hit our buttons rather than lock a color.
        let over_self = win::point_over_own_window(x, y);
        if !over_self && let Some(px) = win::pixel_at(x, y) {
            self.color = Rgb(px);
        }

        if !win::left_button_down() {
            self.armed = true;
        } else if self.armed {
            self.armed = false;
            if !over_self {
                self.picking = false;
                ctx.copy_text(self.color.hex());
                self.copied = Some((0, Instant::now()));
            }
        }
        ctx.request_repaint_after(Duration::from_millis(16));
    }

    fn swatch(&self, ui: &mut egui::Ui) {
        let (rect, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), SWATCH_H), egui::Sense::hover());
        let [r, g, b] = self.color.0;
        let light = self.color.is_light();
        // A 1px inset edge so the swatch reads as a inlaid tile. It has to flip
        // polarity or it vanishes against very dark / very light colors.
        let edge =
            if light { Color32::from_black_alpha(38) } else { Color32::from_white_alpha(30) };
        ui.painter().rect(
            rect,
            CornerRadius::same(12),
            Color32::from_rgb(r, g, b),
            Stroke::new(1.0, edge),
            egui::StrokeKind::Inside,
        );

        let ink = if light { Color32::from_black_alpha(200) } else { Color32::from_white_alpha(225) };
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            self.color.hex(),
            egui::FontId::monospace(21.0),
            ink,
        );
        if self.picking {
            ui.painter().text(
                rect.center_bottom() - egui::vec2(0.0, 14.0),
                egui::Align2::CENTER_CENTER,
                "click to lock  ·  Esc to cancel",
                egui::FontId::proportional(10.0),
                if light { Color32::from_black_alpha(120) } else { Color32::from_white_alpha(150) },
            );
        }
    }

    fn copy_row(&mut self, ui: &mut egui::Ui, idx: usize, label: &str, value: String) {
        let flash =
            matches!(self.copied, Some((i, t)) if i == idx && t.elapsed() < Duration::from_millis(1200));

        egui::Frame::new()
            .fill(ROW_BG)
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::symmetric(12, 0))
            .show(ui, |ui| {
                // `horizontal` (not `horizontal_centered`) — the latter claims all
                // remaining vertical space, which stretches the row to the window.
                ui.horizontal(|ui| {
                    ui.set_min_height(ROW_H);
                    ui.allocate_ui_with_layout(
                        Vec2::new(LABEL_W, ROW_H),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            ui.label(
                                RichText::new(label).monospace().size(10.0).color(CARD_MUTED),
                            );
                        },
                    );
                    ui.label(RichText::new(&value).monospace().size(13.0).color(CARD_FG));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let text = if flash { "Copied" } else { "Copy" };
                        let btn = egui::Button::new(
                            RichText::new(text)
                                .size(11.0)
                                .color(if flash { SHELL_BG } else { CARD_FG }),
                        )
                        .corner_radius(CornerRadius::same(7))
                        .min_size(Vec2::new(COPY_W, 24.0));
                        let btn = if flash { btn.fill(ACCENT) } else { btn };
                        let r = ui.add(btn).on_hover_cursor(egui::CursorIcon::PointingHand);
                        if r.clicked() {
                            ui.ctx().copy_text(value.clone());
                            self.copied = Some((idx, Instant::now()));
                        }
                    });
                });
            });

        if flash {
            ui.ctx().request_repaint_after(Duration::from_millis(120));
        }
    }

    fn footer(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.set_min_height(30.0);
            let label = if self.picking { "Picking…" } else { "Pick color" };
            let pick = ui
                .scope(|ui| {
                    let w = &mut ui.visuals_mut().widgets;
                    w.inactive.weak_bg_fill = ACCENT;
                    w.hovered.weak_bg_fill = ACCENT_HOVER;
                    w.active.weak_bg_fill = ACCENT_ACTIVE;
                    ui.add(
                        egui::Button::new(
                            RichText::new(label).size(12.5).strong().color(SHELL_BG),
                        )
                        .corner_radius(CornerRadius::same(8))
                        .min_size(Vec2::new(112.0, 30.0)),
                    )
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                })
                .inner;
            if pick.clicked() {
                let ctx = ui.ctx().clone();
                self.start_pick(&ctx);
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(win::HOTKEY_LABEL).size(11.0).color(CARD_MUTED));
            });
        });
    }
}

impl eframe::App for Hexer {
    fn ui(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = root.ctx().clone();
        self.poll_picker(&ctx);

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(SHELL_BG).inner_margin(Margin::same(12)))
            .show(root, |ui| {
                egui::Frame::new()
                    .fill(CARD_BG)
                    .corner_radius(CornerRadius::same(14))
                    .stroke(Stroke::new(1.0, CARD_LINE))
                    .inner_margin(Margin::same(14))
                    .show(ui, |ui| {
                        self.swatch(ui);
                        ui.add_space(6.0);
                        self.copy_row(ui, 0, "HEX", self.color.hex());
                        self.copy_row(ui, 1, "RGB", self.color.rgb());
                        self.copy_row(ui, 2, "HSL", self.color.hsl());
                        ui.add_space(6.0);
                        self.footer(ui);
                    });
            });
    }
}

fn visuals() -> egui::Visuals {
    let mut v = egui::Visuals::dark();
    v.panel_fill = SHELL_BG;
    v.window_fill = SHELL_BG;
    v.override_text_color = Some(CARD_FG);
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, CARD_LINE);

    for w in [
        &mut v.widgets.inactive,
        &mut v.widgets.hovered,
        &mut v.widgets.active,
        &mut v.widgets.open,
    ] {
        // No grow-on-hover: these are small boxes and the wobble reads as a bug.
        w.expansion = 0.0;
        w.bg_stroke = Stroke::NONE;
        w.corner_radius = CornerRadius::same(7);
    }
    v.widgets.inactive.weak_bg_fill = BTN_BG;
    v.widgets.inactive.bg_fill = BTN_BG;
    v.widgets.hovered.weak_bg_fill = BTN_HOVER;
    v.widgets.hovered.bg_fill = BTN_HOVER;
    v.widgets.active.weak_bg_fill = ACCENT;
    v.widgets.active.bg_fill = ACCENT;
    v
}
