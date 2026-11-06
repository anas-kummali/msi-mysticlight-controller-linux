//! Main GUI application coordinator and lifecycle engine.

use std::time::Duration;
use eframe::egui::{self, Color32, RichText, Stroke};

use crate::color::Rgb;
use crate::constants::{MODE_BREATHING, MODE_CYCLE, MODE_WAVE};
use crate::controller::MSIKeyboardController;
use crate::device::DeviceInfo;
use crate::gui::keyboard::render_keyboard_canvas;
use crate::gui::style::{
    configure_visuals, ACCENT_CYAN, ACCENT_GREEN, ACCENT_RED, BORDER_ACTIVE,
    BORDER_SUBTLE, CARD_BG, CARD_SELECTED, PANEL_BG, TEXT_MUTED, TEXT_PRIMARY,
};
use crate::gui::views::controls::render_controls_view;
use crate::gui::views::effects::{render_effects_view, ActiveEffect};
use crate::gui::views::status::render_status_view;
use crate::gui::views::themes::render_themes_view;
use crate::matrix::{ROW_ZONES, TOTAL_ZONES};
use crate::theme::{get_builtin_themes, Theme};
use crate::VERSION;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationTab {
    Studio,
    Effects,
    Themes,
    Diagnostics,
}

pub struct MysticLightApp {
    ctrl: MSIKeyboardController,
    device_info: DeviceInfo,
    active_tab: NavigationTab,
    selected_zone: Option<usize>,
    selected_row: Option<u8>,
    zone_colors: [Rgb; TOTAL_ZONES],
    color_rgb: [f32; 3],
    hex_input: String,
    brightness: u8,
    active_effect: ActiveEffect,
    wave_speed: u16,
    cycle_speed: u16,
    breathe_speed: u16,
    themes: Vec<Theme>,
    selected_theme: Option<&'static str>,
    status_toast: String,
}

impl MysticLightApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_visuals(&cc.egui_ctx);

        let mut ctrl = MSIKeyboardController::new();
        let _ = ctrl.open();
        let info = ctrl.get_device_info();

        let initial_rgb = Rgb::new(255, 255, 255);
        let zone_colors = [initial_rgb; TOTAL_ZONES];
        let themes = get_builtin_themes();

        let active_tab = match std::env::var("MYSTICLIGHT_TAB").as_deref() {
            Ok("effects") => NavigationTab::Effects,
            Ok("themes") => NavigationTab::Themes,
            Ok("diagnostics") => NavigationTab::Diagnostics,
            _ => NavigationTab::Studio,
        };

        Self {
            ctrl,
            device_info: info,
            active_tab,
            selected_zone: None,
            selected_row: None,
            zone_colors,
            color_rgb: [1.0, 1.0, 1.0],
            hex_input: "#FFFFFF".to_string(),
            brightness: 10,
            active_effect: ActiveEffect::None,
            wave_speed: 200,
            cycle_speed: 150,
            breathe_speed: 120,
            themes,
            selected_theme: Some("sovereign-white"),
            status_toast: "Driver online · Hardware communication ready.".to_string(),
        }
    }

    fn current_rgb(&self) -> Rgb {
        Rgb::new(
            (self.color_rgb[0] * 255.0).round().clamp(0.0, 255.0) as u8,
            (self.color_rgb[1] * 255.0).round().clamp(0.0, 255.0) as u8,
            (self.color_rgb[2] * 255.0).round().clamp(0.0, 255.0) as u8,
        )
    }

    fn apply_selected_color(&mut self) {
        let rgb = self.current_rgb();
        let b = self.brightness;

        if let Some(z) = self.selected_zone {
            if let Err(e) = self.ctrl.set_color(rgb.r, rgb.g, rgb.b, b, None, Some(z as u8)) {
                self.status_toast = format!("Error setting zone {}: {}", z, e);
            } else {
                self.zone_colors[z] = rgb;
                self.status_toast = format!("Applied {} to Zone Z{:02}.", rgb, z);
                self.active_effect = ActiveEffect::None;
            }
        } else if let Some(r) = self.selected_row {
            if let Err(e) = self.ctrl.set_color(rgb.r, rgb.g, rgb.b, b, Some(r), None) {
                self.status_toast = format!("Error setting row {}: {}", r, e);
            } else {
                if let Some(row_entry) = ROW_ZONES.iter().find(|(r_num, _)| *r_num == r) {
                    for &z in &row_entry.1 {
                        self.zone_colors[z as usize] = rgb;
                    }
                }
                self.status_toast = format!("Applied {} across Physical Row {}.", rgb, r);
                self.active_effect = ActiveEffect::None;
            }
        } else {
            if let Err(e) = self.ctrl.set_color(rgb.r, rgb.g, rgb.b, b, None, None) {
                self.status_toast = format!("Error setting color: {}", e);
            } else {
                self.zone_colors = [rgb; TOTAL_ZONES];
                self.status_toast = format!("Applied {} across all 24 zones.", rgb);
                self.active_effect = ActiveEffect::None;
            }
        }
    }

    fn execute_effect(&mut self, effect: ActiveEffect) {
        self.active_effect = effect;
        match effect {
            ActiveEffect::Wave => {
                let rainbow = vec![
                    (255, 0, 0),
                    (255, 128, 0),
                    (255, 255, 0),
                    (0, 255, 0),
                    (0, 255, 255),
                    (0, 0, 255),
                    (255, 0, 255),
                ];
                if let Err(e) = self.ctrl.set_effect(MODE_WAVE, &rainbow, self.wave_speed) {
                    self.status_toast = format!("Error activating wave: {}", e);
                } else {
                    self.status_toast = "Hardware Rainbow Wave active across all 6 rows.".to_string();
                }
            }
            ActiveEffect::Cycle => {
                let cycle_cols = vec![
                    (255, 0, 0),
                    (0, 255, 0),
                    (0, 120, 255),
                    (255, 255, 0),
                ];
                if let Err(e) = self.ctrl.set_effect(MODE_CYCLE, &cycle_cols, self.cycle_speed) {
                    self.status_toast = format!("Error activating cycle: {}", e);
                } else {
                    self.status_toast = "Hardware Color Cycle active across all 24 zones.".to_string();
                }
            }
            ActiveEffect::Breathe => {
                let rgb = self.current_rgb();
                let seq = vec![(0, 0, 0), (rgb.r, rgb.g, rgb.b), (0, 0, 0)];
                if let Err(e) = self.ctrl.set_effect(MODE_BREATHING, &seq, self.breathe_speed) {
                    self.status_toast = format!("Error activating breathing: {}", e);
                } else {
                    self.status_toast = format!("Hardware Breathing active with {}.", rgb);
                }
            }
            ActiveEffect::Off => {
                if let Err(e) = self.ctrl.turn_off() {
                    self.status_toast = format!("Error powering off: {}", e);
                } else {
                    self.zone_colors = [Rgb::new(0, 0, 0); TOTAL_ZONES];
                    self.status_toast = "Keyboard LEDs powered off completely.".to_string();
                }
            }
            ActiveEffect::None => {}
        }
    }

    /// Animate on-screen keyboard representation to match live hardware mode.
    fn step_animation_frame(&mut self, time: f64) {
        match self.active_effect {
            ActiveEffect::Wave => {
                for z in 0..TOTAL_ZONES {
                    let col = (z % 4) as f64;
                    let hue = ((time * 0.8 + col * 0.25) % 1.0) as f32;
                    let rgb = hsv_to_rgb(hue, 1.0, 1.0);
                    self.zone_colors[z] = rgb;
                }
            }
            ActiveEffect::Cycle => {
                let hue = ((time * 0.4) % 1.0) as f32;
                let rgb = hsv_to_rgb(hue, 1.0, 1.0);
                self.zone_colors = [rgb; TOTAL_ZONES];
            }
            ActiveEffect::Breathe => {
                let factor = ((time * 1.5).sin().abs()) as f32;
                let base = self.current_rgb();
                let rgb = Rgb::new(
                    (base.r as f32 * factor) as u8,
                    (base.g as f32 * factor) as u8,
                    (base.b as f32 * factor) as u8,
                );
                self.zone_colors = [rgb; TOTAL_ZONES];
            }
            _ => {}
        }
    }
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Rgb {
    let i = (h * 6.0).floor() as i32;
    let f = h * 6.0 - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    let (r, g, b) = match i % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    Rgb::new((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

impl eframe::App for MysticLightApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.active_effect != ActiveEffect::None && self.active_effect != ActiveEffect::Off {
            let t = ctx.input(|i| i.time);
            self.step_animation_frame(t);
            ctx.request_repaint_after(Duration::from_millis(16));
        }

        // 1. Top Panel (Header & Navigation Tabs)
        egui::TopBottomPanel::top("top_header")
            .frame(egui::Frame::none().fill(PANEL_BG).inner_margin(12.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("⚡").size(22.0).color(ACCENT_CYAN));
                    ui.heading(RichText::new("MSI MYSTIC LIGHT").strong().size(18.0));
                    ui.label(RichText::new(format!("v{}", VERSION)).size(11.0).color(TEXT_MUTED));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if self.device_info.present {
                            let node_str = self.device_info.node.as_deref().unwrap_or("/dev/hidraw");
                            ui.colored_label(ACCENT_GREEN, format!("● Online ({})", node_str));
                        } else {
                            ui.colored_label(ACCENT_RED, "● Disconnected");
                        }
                    });
                });

                ui.add_space(8.0);

                // Navigation Pills
                ui.horizontal(|ui| {
                    let tabs = [
                        (NavigationTab::Studio, "🎨 Studio Controls"),
                        (NavigationTab::Effects, "🌊 Effects Gallery"),
                        (NavigationTab::Themes, "✨ Theme Studio"),
                        (NavigationTab::Diagnostics, "⚙ Diagnostics"),
                    ];

                    for (tab, label) in tabs {
                        let is_active = self.active_tab == tab;
                        let btn = egui::Button::new(
                            RichText::new(label)
                                .size(12.5)
                                .strong()
                                .color(if is_active { ACCENT_CYAN } else { TEXT_PRIMARY }),
                        )
                        .fill(if is_active { CARD_SELECTED } else { CARD_BG })
                        .stroke(if is_active { Stroke::new(1.5_f32, BORDER_ACTIVE) } else { Stroke::new(1.0_f32, BORDER_SUBTLE) });

                        if ui.add(btn).clicked() {
                            self.active_tab = tab;
                        }
                    }
                });
            });

        // 2. Bottom Persistent Master Action Deck
        egui::TopBottomPanel::bottom("bottom_deck")
            .frame(
                egui::Frame::none()
                    .fill(PANEL_BG)
                    .inner_margin(12.0)
                    .stroke(Stroke::new(1.0_f32, BORDER_SUBTLE)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("☀ Brightness:").strong().size(12.0));
                    let mut b = self.brightness;
                    if ui.add(egui::Slider::new(&mut b, 0..=10).text("Level")).changed() {
                        self.brightness = b;
                        let rgb = self.current_rgb();
                        let _ = self.ctrl.set_color(rgb.r, rgb.g, rgb.b, b, None, None);
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(RichText::new("⏹ Power Off").color(ACCENT_RED)).clicked() {
                            self.execute_effect(ActiveEffect::Off);
                        }
                        if ui.button(RichText::new("🌊 Rainbow Wave").color(ACCENT_CYAN)).clicked() {
                            self.execute_effect(ActiveEffect::Wave);
                        }
                        if ui.button(RichText::new("⚪ Solid White").color(Color32::WHITE)).clicked() {
                            self.color_rgb = [1.0, 1.0, 1.0];
                            self.hex_input = "#FFFFFF".to_string();
                            self.selected_zone = None;
                            self.selected_row = None;
                            self.apply_selected_color();
                        }
                    });
                });

                ui.add_space(4.0);

                // Status message toast bar
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&self.status_toast).italics().size(11.5).color(TEXT_MUTED));
                });
            });

        // 3. Central Canvas and Scrollable Tab View
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Interactive Keyboard Matrix Canvas
                let clicked_zone = render_keyboard_canvas(
                    ui,
                    &self.zone_colors,
                    self.selected_zone,
                    self.selected_row,
                );

                if let Some(z) = clicked_zone {
                    self.selected_zone = Some(z);
                    self.selected_row = None;
                    let c = self.zone_colors[z];
                    self.color_rgb = [c.r as f32 / 255.0, c.g as f32 / 255.0, c.b as f32 / 255.0];
                    self.hex_input = format!("{}", c);
                }

                ui.add_space(6.0);

                // Quick Cluster Filters
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Quick Target:").size(11.5).strong().color(TEXT_MUTED));

                    let all_btn = egui::Button::new(RichText::new("Full Board (24 Zones)").size(11.0))
                        .fill(if self.selected_zone.is_none() && self.selected_row.is_none() { CARD_SELECTED } else { CARD_BG });
                    if ui.add(all_btn).clicked() {
                        self.selected_zone = None;
                        self.selected_row = None;
                    }

                    for r in 1..=6 {
                        let row_btn = egui::Button::new(RichText::new(format!("Row {}", r)).size(11.0))
                            .fill(if self.selected_row == Some(r) { CARD_SELECTED } else { CARD_BG });
                        if ui.add(row_btn).clicked() {
                            self.selected_row = Some(r);
                            self.selected_zone = None;
                        }
                    }
                });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(6.0);

                // Tab View Content
                match self.active_tab {
                    NavigationTab::Studio => {
                        let mut apply_requested = false;
                        render_controls_view(
                            ui,
                            &mut self.color_rgb,
                            &mut self.hex_input,
                            &mut self.brightness,
                            self.selected_zone,
                            self.selected_row,
                            &mut apply_requested,
                        );
                        if apply_requested {
                            self.apply_selected_color();
                        }
                    }
                    NavigationTab::Effects => {
                        let mut triggered = None;
                        render_effects_view(
                            ui,
                            self.active_effect,
                            &mut self.wave_speed,
                            &mut self.cycle_speed,
                            &mut self.breathe_speed,
                            &mut triggered,
                        );
                        if let Some(fx) = triggered {
                            self.execute_effect(fx);
                        }
                    }
                    NavigationTab::Themes => {
                        let mut chosen_idx = None;
                        render_themes_view(
                            ui,
                            &self.themes,
                            self.selected_theme,
                            &mut chosen_idx,
                        );
                        if let Some(idx) = chosen_idx {
                            let t = &self.themes[idx];
                            self.selected_theme = Some(t.id);
                            if let Err(e) = t.apply(&self.ctrl, self.brightness) {
                                self.status_toast = format!("Error applying theme {}: {}", t.name, e);
                            } else {
                                self.zone_colors = t.zones;
                                self.active_effect = ActiveEffect::None;
                                self.status_toast = format!("Theme '{}' applied across all 6 rows.", t.name);
                            }
                        }
                    }
                    NavigationTab::Diagnostics => {
                        render_status_view(ui, &self.device_info);
                    }
                }
            });
        });
    }
}

/// Run native GUI application.
pub fn run_gui() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([940.0, 720.0])
            .with_min_inner_size([800.0, 580.0])
            .with_title("MSI Mystic Light Controller"),
        ..Default::default()
    };

    eframe::run_native(
        "MSI Mystic Light",
        options,
        Box::new(|cc| Ok(Box::new(MysticLightApp::new(cc)))),
    )
}
