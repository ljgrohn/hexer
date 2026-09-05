#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod color;
mod win;

use std::sync::mpsc::{Receiver, channel};
use std::time::{Duration, Instant};

use color::Rgb;
use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Stroke, Vec2};
use win::HotkeyEvent;

const SHELL_BG: Color32 = Color32::from_rgb(0x10, 0x11, 0x15);
const CARD_BG: Color32 = Color32::from_rgb(0x19, 0x1B, 0x21);
const SURFACE_BG: Color32 = Color32::from_rgb(0x20, 0x22, 0x2A);
const SURFACE_HOVER: Color32 = Color32::from_rgb(0x2A, 0x2D, 0x37);
const BTN_BG: Color32 = Color32::from_rgb(0x2B, 0x2E, 0x38);
const CARD_FG: Color32 = Color32::from_rgb(0xF4, 0xF4, 0xF6);
const CARD_SUBTLE: Color32 = Color32::from_rgb(0xAF, 0xB0, 0xB9);
const CARD_MUTED: Color32 = Color32::from_rgb(0x7D, 0x80, 0x8C);
const CARD_LINE: Color32 = Color32::from_rgb(0x2C, 0x2F, 0x39);
const ACCENT: Color32 = Color32::from_rgb(0x6C, 0xA6, 0xFF);
const ACCENT_HOVER: Color32 = Color32::from_rgb(0x82, 0xB4, 0xFF);
const ACCENT_ACTIVE: Color32 = Color32::from_rgb(0x54, 0x92, 0xEE);
const SUCCESS: Color32 = Color32::from_rgb(0x75, 0xDB, 0xA4);

const WINDOW_WIDTH: f32 = 392.0;
const PICKER_HEIGHT: f32 = 492.0;
const THEME_HEIGHT: f32 = 566.0;
const SWATCH_H: f32 = 132.0;
const ROW_H: f32 = 40.0;
const LABEL_W: f32 = 38.0;
const COPY_W: f32 = 62.0;
const FLASH_TIME: Duration = Duration::from_millis(1200);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Picker,
    Theme,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CopyTarget {
    Format(u8),
    Palette(u8, u8),
}

fn main() -> eframe::Result {
    // This must happen before eframe creates a window. It keeps cursor and pixel
    // coordinates aligned on scaled and mixed-DPI displays.
    win::set_dpi_aware();

    let viewport = egui::ViewportBuilder::default()
        .with_title("hexer")
        .with_inner_size([WINDOW_WIDTH, PICKER_HEIGHT])
        .with_min_inner_size([WINDOW_WIDTH, PICKER_HEIGHT])
        .with_resizable(false)
        .with_always_on_top();
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "hexer",
        options,
        Box::new(|cc| Ok(Box::new(Hexer::new(cc)))),
    )
}

struct Hexer {
    color: Rgb,
    tab: Tab,
    picking: bool,
    /// Do not accept a locking click until pick mode has observed button-up.
    armed: bool,
    copied: Option<(CopyTarget, Instant)>,
    hotkeys: Receiver<HotkeyEvent>,
}

impl Hexer {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (tx, rx) = channel();
        let ctx = cc.egui_ctx.clone();
        std::thread::spawn(move || win::hotkey_loop(tx, move || ctx.request_repaint()));

        cc.egui_ctx.set_visuals(visuals());
        cc.egui_ctx.all_styles_mut(|style| {
            style.spacing.item_spacing = Vec2::new(8.0, 8.0);
            style.spacing.button_padding = Vec2::new(10.0, 5.0);
        });

        Self {
            color: Rgb([0x6C, 0xA6, 0xFF]),
            tab: Tab::Picker,
            picking: false,
            armed: false,
            copied: None,
            hotkeys: rx,
        }
    }

    fn start_pick(&mut self, ctx: &egui::Context) {
        self.set_tab(Tab::Picker, ctx);
        self.picking = true;
        self.armed = false;
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
    }

    fn set_tab(&mut self, tab: Tab, ctx: &egui::Context) {
        self.tab = tab;
        let height = match tab {
            Tab::Picker => PICKER_HEIGHT,
            Tab::Theme => THEME_HEIGHT,
        };
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(Vec2::new(
            WINDOW_WIDTH,
            height,
        )));
    }

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
        let over_self = win::point_over_own_window(x, y);
        if !over_self && let Some(pixel) = win::pixel_at(x, y) {
            self.color = Rgb(pixel);
        }

        if !win::left_button_down() {
            self.armed = true;
        } else if self.armed {
            self.armed = false;
            if !over_self {
                self.picking = false;
                ctx.copy_text(self.color.hex());
                self.copied = Some((CopyTarget::Format(0), Instant::now()));
            }
        }
        ctx.request_repaint_after(Duration::from_millis(16));
    }

    fn just_copied(&self, target: CopyTarget) -> bool {
        matches!(self.copied, Some((current, at)) if current == target && at.elapsed() < FLASH_TIME)
    }

    fn copy(&mut self, ctx: &egui::Context, target: CopyTarget, value: String) {
        ctx.copy_text(value);
        self.copied = Some((target, Instant::now()));
    }

    fn header(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let (mark, _) = ui.allocate_exact_size(Vec2::splat(30.0), egui::Sense::hover());
            ui.painter()
                .rect_filled(mark, CornerRadius::same(9), ACCENT);
            ui.painter().text(
                mark.center(),
                egui::Align2::CENTER_CENTER,
                "H",
                egui::FontId::monospace(15.0),
                SHELL_BG,
            );

            ui.vertical(|ui| {
                ui.add_space(1.0);
                ui.label(RichText::new("hexer").size(15.0).strong().color(CARD_FG));
                ui.label(
                    RichText::new("PICK  /  PAIR  /  COPY")
                        .size(8.5)
                        .monospace()
                        .color(CARD_MUTED),
                );
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (dot, text) = if self.picking {
                    (ACCENT, "SAMPLING")
                } else {
                    (SUCCESS, "READY")
                };
                egui::Frame::new()
                    .fill(SURFACE_BG)
                    .corner_radius(CornerRadius::same(10))
                    .inner_margin(Margin::symmetric(9, 5))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let (rect, _) =
                                ui.allocate_exact_size(Vec2::splat(6.0), egui::Sense::hover());
                            ui.painter().circle_filled(rect.center(), 3.0, dot);
                            ui.label(RichText::new(text).monospace().size(9.0).color(CARD_SUBTLE));
                        });
                    });
            });
        });
    }

    fn tabs(&mut self, ui: &mut egui::Ui) {
        egui::Frame::new()
            .fill(CARD_BG)
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::same(4))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                ui.horizontal(|ui| {
                    let width = (ui.available_width() - 4.0) / 2.0;
                    for (tab, label) in [(Tab::Picker, "Picker"), (Tab::Theme, "Theme")] {
                        let selected = self.tab == tab;
                        let button = egui::Button::new(
                            RichText::new(label).size(11.5).strong().color(if selected {
                                CARD_FG
                            } else {
                                CARD_MUTED
                            }),
                        )
                        .fill(if selected {
                            SURFACE_BG
                        } else {
                            Color32::TRANSPARENT
                        })
                        .corner_radius(CornerRadius::same(7));
                        if ui
                            .add_sized([width, 29.0], button)
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                        {
                            self.set_tab(tab, ui.ctx());
                        }
                    }
                });
            });
    }

    fn swatch(&self, ui: &mut egui::Ui) {
        let (rect, _) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), SWATCH_H),
            egui::Sense::hover(),
        );
        let [r, g, b] = self.color.0;
        let light = self.color.is_light();
        let edge = if light {
            Color32::from_black_alpha(40)
        } else {
            Color32::from_white_alpha(34)
        };
        ui.painter().rect(
            rect,
            CornerRadius::same(13),
            Color32::from_rgb(r, g, b),
            Stroke::new(1.0, edge),
            egui::StrokeKind::Inside,
        );

        let ink = if light {
            Color32::from_black_alpha(210)
        } else {
            Color32::from_white_alpha(235)
        };
        ui.painter().text(
            rect.center() - egui::vec2(0.0, 6.0),
            egui::Align2::CENTER_CENTER,
            self.color.hex(),
            egui::FontId::monospace(23.0),
            ink,
        );
        ui.painter().text(
            rect.center_bottom() - egui::vec2(0.0, 18.0),
            egui::Align2::CENTER_CENTER,
            if self.picking {
                "CLICK TO LOCK  -  ESC TO CANCEL"
            } else {
                "CURRENT COLOR"
            },
            egui::FontId::monospace(8.5),
            if light {
                Color32::from_black_alpha(120)
            } else {
                Color32::from_white_alpha(145)
            },
        );
    }

    fn copy_row(&mut self, ui: &mut egui::Ui, idx: u8, label: &str, value: String) {
        let target = CopyTarget::Format(idx);
        let flash = self.just_copied(target);

        egui::Frame::new()
            .fill(SURFACE_BG)
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::symmetric(12, 0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.set_min_height(ROW_H);
                    ui.allocate_ui_with_layout(
                        Vec2::new(LABEL_W, ROW_H),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            ui.label(RichText::new(label).monospace().size(9.5).color(CARD_MUTED));
                        },
                    );
                    ui.label(RichText::new(&value).monospace().size(12.5).color(CARD_FG));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let button = egui::Button::new(
                            RichText::new(if flash { "Copied" } else { "Copy" })
                                .size(10.5)
                                .strong()
                                .color(if flash { SHELL_BG } else { CARD_FG }),
                        )
                        .fill(if flash { SUCCESS } else { BTN_BG })
                        .corner_radius(CornerRadius::same(7))
                        .min_size(Vec2::new(COPY_W, 26.0));
                        if ui
                            .add(button)
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                        {
                            self.copy(ui.ctx(), target, value.clone());
                        }
                    });
                });
            });

        if flash {
            ui.ctx().request_repaint_after(Duration::from_millis(100));
        }
    }

    fn picker_footer(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let label = if self.picking {
                "Sampling..."
            } else {
                "Pick color"
            };
            let pick = ui
                .scope(|ui| {
                    let widgets = &mut ui.visuals_mut().widgets;
                    widgets.inactive.weak_bg_fill = ACCENT;
                    widgets.hovered.weak_bg_fill = ACCENT_HOVER;
                    widgets.active.weak_bg_fill = ACCENT_ACTIVE;
                    ui.add(
                        egui::Button::new(RichText::new(label).size(12.0).strong().color(SHELL_BG))
                            .corner_radius(CornerRadius::same(8))
                            .min_size(Vec2::new(122.0, 32.0)),
                    )
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                })
                .inner;
            if pick.clicked() {
                self.start_pick(ui.ctx());
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new(win::HOTKEY_LABEL)
                        .monospace()
                        .size(9.5)
                        .color(CARD_MUTED),
                );
            });
        });
    }

    fn picker_tab(&mut self, ui: &mut egui::Ui) {
        self.swatch(ui);
        ui.add_space(4.0);
        self.copy_row(ui, 0, "HEX", self.color.hex());
        self.copy_row(ui, 1, "RGB", self.color.rgb());
        self.copy_row(ui, 2, "HSL", self.color.hsl());
        ui.add_space(6.0);
        self.picker_footer(ui);
    }

    fn theme_base(&mut self, ui: &mut egui::Ui) {
        egui::Frame::new()
            .fill(SURFACE_BG)
            .corner_radius(CornerRadius::same(11))
            .inner_margin(Margin::symmetric(10, 9))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let (swatch, _) =
                        ui.allocate_exact_size(Vec2::splat(34.0), egui::Sense::hover());
                    let [r, g, b] = self.color.0;
                    ui.painter().rect(
                        swatch,
                        CornerRadius::same(8),
                        Color32::from_rgb(r, g, b),
                        Stroke::new(1.0, Color32::from_white_alpha(28)),
                        egui::StrokeKind::Inside,
                    );
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new("BASE COLOR")
                                .monospace()
                                .size(8.5)
                                .color(CARD_MUTED),
                        );
                        ui.label(
                            RichText::new(self.color.hex())
                                .monospace()
                                .size(13.0)
                                .color(CARD_FG),
                        );
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("Pick new").size(10.5).color(CARD_FG),
                                )
                                .fill(BTN_BG)
                                .corner_radius(CornerRadius::same(7))
                                .min_size(Vec2::new(72.0, 27.0)),
                            )
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                        {
                            self.start_pick(ui.ctx());
                        }
                    });
                });
            });
    }

    fn palette_chip(&mut self, ui: &mut egui::Ui, target: CopyTarget, color: Rgb, width: f32) {
        let flash = self.just_copied(target);
        let [r, g, b] = color.0;
        let fill = Color32::from_rgb(r, g, b);
        let ink = if color.is_light() {
            Color32::from_black_alpha(210)
        } else {
            Color32::from_white_alpha(235)
        };
        let button = egui::Button::new(
            RichText::new(if flash {
                "Copied".to_owned()
            } else {
                color.hex()
            })
            .monospace()
            .size(10.0)
            .strong()
            .color(ink),
        )
        .fill(fill)
        .stroke(Stroke::new(1.0, Color32::from_white_alpha(30)))
        .corner_radius(CornerRadius::same(8));
        let response = ui
            .add_sized([width, 35.0], button)
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .on_hover_text("Copy color");
        if response.clicked() {
            self.copy(ui.ctx(), target, color.hex());
        }
        if flash {
            ui.ctx().request_repaint_after(Duration::from_millis(100));
        }
    }

    fn palette_card(
        &mut self,
        ui: &mut egui::Ui,
        scheme: u8,
        title: &str,
        description: &str,
        offsets: [f32; 2],
    ) {
        let colors = [
            self.color,
            self.color.harmonized(offsets[0]),
            self.color.harmonized(offsets[1]),
        ];
        egui::Frame::new()
            .fill(SURFACE_BG)
            .corner_radius(CornerRadius::same(11))
            .inner_margin(Margin::symmetric(10, 9))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(title).size(11.5).strong().color(CARD_FG));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(description).size(9.5).color(CARD_MUTED));
                    });
                });
                ui.add_space(4.0);
                ui.spacing_mut().item_spacing.x = 6.0;
                ui.horizontal(|ui| {
                    let width = (ui.available_width() - 12.0) / 3.0;
                    for (index, color) in colors.into_iter().enumerate() {
                        self.palette_chip(
                            ui,
                            CopyTarget::Palette(scheme, index as u8),
                            color,
                            width,
                        );
                    }
                });
            });
    }

    fn theme_tab(&mut self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new("Build a theme")
                .size(17.0)
                .strong()
                .color(CARD_FG),
        );
        ui.label(
            RichText::new("Color pairings generated from your current pick.")
                .size(10.5)
                .color(CARD_SUBTLE),
        );
        ui.add_space(5.0);
        self.theme_base(ui);
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("HARMONY OPTIONS")
                    .monospace()
                    .size(8.5)
                    .color(CARD_MUTED),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new("Click a swatch to copy")
                        .size(9.0)
                        .color(CARD_MUTED),
                );
            });
        });
        self.palette_card(ui, 0, "Analogous", "calm + cohesive", [-30.0, 30.0]);
        self.palette_card(ui, 1, "Complementary", "balanced contrast", [150.0, 180.0]);
        self.palette_card(ui, 2, "Triadic", "bright + playful", [120.0, 240.0]);
    }
}

impl eframe::App for Hexer {
    fn ui(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = root.ctx().clone();
        self.poll_picker(&ctx);

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(SHELL_BG)
                    .inner_margin(Margin::same(14)),
            )
            .show(root, |ui| {
                self.header(ui);
                ui.add_space(10.0);
                self.tabs(ui);
                ui.add_space(10.0);
                egui::Frame::new()
                    .fill(CARD_BG)
                    .corner_radius(CornerRadius::same(15))
                    .stroke(Stroke::new(1.0, CARD_LINE))
                    .inner_margin(Margin::same(14))
                    .show(ui, |ui| match self.tab {
                        Tab::Picker => self.picker_tab(ui),
                        Tab::Theme => self.theme_tab(ui),
                    });
            });
    }
}

fn visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = SHELL_BG;
    visuals.window_fill = SHELL_BG;
    visuals.override_text_color = Some(CARD_FG);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, CARD_LINE);

    for widget in [
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        widget.expansion = 0.0;
        widget.bg_stroke = Stroke::NONE;
        widget.corner_radius = CornerRadius::same(7);
    }
    visuals.widgets.inactive.weak_bg_fill = BTN_BG;
    visuals.widgets.inactive.bg_fill = BTN_BG;
    visuals.widgets.hovered.weak_bg_fill = SURFACE_HOVER;
    visuals.widgets.hovered.bg_fill = SURFACE_HOVER;
    visuals.widgets.active.weak_bg_fill = ACCENT_ACTIVE;
    visuals.widgets.active.bg_fill = ACCENT_ACTIVE;
    visuals
}
