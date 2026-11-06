//! Graphical User Interface (GUI) implementation using egui / eframe.

use eframe::egui::{self, Color32, RichText, Stroke, Vec2};
use crate::color::{Rgb, COLORS};
use crate::constants::{MODE_BREATHING, MODE_CYCLE, MODE_WAVE};
use crate::controller::MSIKeyboardController;
use crate::device::DeviceInfo;
use crate::matrix::{ROW_ZONES, TOTAL_ZONES};
use crate::theme::{get_builtin_themes, Theme};
use crate::VERSION;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GuiTab {
    Controls,
    Themes,
    Effects,
    Status,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TargetMode {
    AllZones,
    Row(u8),
    Zone(u8),
}

/// Main GUI application state.
pub struct MysticLightApp {
    ctrl: MSIKeyboardController,
    device_info: DeviceInfo,
    active_tab: GuiTab,
    target_mode: TargetMode,
    color_rgb: [f32; 3], // RGB in 0.0 .. 1.0 range
    hex_input: String,
    brightness: u8,
    zone_colors: [Rgb; TOTAL_ZONES],
    selected_zone: Option<usize>,
    themes: Vec<Theme>,
    status_banner: String,
    wave_speed: u16,
    cycle_speed: u16,
    breathe_speed: u16,
}

impl MysticLightApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut ctrl = MSIKeyboardController::new();
        let _ = ctrl.open();
        let info = ctrl.get_device_info();

        let initial_color = Rgb::new(255, 255, 255);
        let zone_colors = [initial_color; TOTAL_ZONES];
        let themes = get_builtin_themes();

        Self {
            ctrl,
            device_info: info,
            active_tab: GuiTab::Controls,
            target_mode: TargetMode::AllZones,
            color_rgb: [1.0, 1.0, 1.0],
            hex_input: "#FFFFFF".to_string(),
            brightness: 10,
            zone_colors,
            selected_zone: None,
            themes,
            status_banner: "Hardware Controller initialized successfully.".to_string(),
            wave_speed: 200,
            cycle_speed: 150,
            breathe_speed: 120,
        }
    }

    fn current_rgb(&self) -> Rgb {
        Rgb::new(
            (self.color_rgb[0] * 255.0).round().clamp(0.0, 255.0) as u8,
            (self.color_rgb[1] * 255.0).round().clamp(0.0, 255.0) as u8,
            (self.color_rgb[2] * 255.0).round().clamp(0.0, 255.0) as u8,
        )
    }

    fn set_rgb(&mut self, rgb: Rgb) {
        self.color_rgb = [
            rgb.r as f32 / 255.0,
            rgb.g as f32 / 255.0,
            rgb.b as f32 / 255.0,
        ];
        self.hex_input = format!("{}", rgb);
    }

    fn apply_current_color(&mut self) {
        let rgb = self.current_rgb();
        let b = self.brightness;

        match self.target_mode {
            TargetMode::AllZones => {
                if let Err(e) = self.ctrl.set_color(rgb.r, rgb.g, rgb.b, b, None, None) {
                    self.status_banner = format!("Error setting color: {}", e);
                } else {
                    self.zone_colors = [rgb; TOTAL_ZONES];
                    self.status_banner = format!("Applied {} to All 24 Zones.", rgb);
                }
            }
            TargetMode::Row(r) => {
                if let Err(e) = self.ctrl.set_color(rgb.r, rgb.g, rgb.b, b, Some(r), None) {
                    self.status_banner = format!("Error setting row {}: {}", r, e);
                } else {
                    if let Some(row_entry) = ROW_ZONES.iter().find(|(row_num, _)| *row_num == r) {
                        for &z in &row_entry.1 {
                            self.zone_colors[z as usize] = rgb;
                        }
                    }
                    self.status_banner = format!("Applied {} to Physical Row {}.", rgb, r);
                }
            }
            TargetMode::Zone(z) => {
                if let Err(e) = self.ctrl.set_color(rgb.r, rgb.g, rgb.b, b, None, Some(z)) {
                    self.status_banner = format!("Error setting zone {}: {}", z, e);
                } else {
                    self.zone_colors[z as usize] = rgb;
                    self.status_banner = format!("Applied {} to Zone {}.", rgb, z);
                }
            }
        }
    }
}

impl eframe::App for MysticLightApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Header Bar
            ui.horizontal(|ui| {
                ui.heading(RichText::new("MSI Mystic Light Controller").strong().size(22.0));
                ui.label(RichText::new(format!("v{}", VERSION)).color(Color32::from_rgb(120, 160, 255)));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.device_info.present {
                        ui.colored_label(Color32::from_rgb(50, 205, 50), "● Hardware Connected");
                    } else {
                        ui.colored_label(Color32::from_rgb(255, 69, 0), "● Hardware Disconnected");
                    }
                });
            });

            ui.separator();

            // Navigation Tabs
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.active_tab, GuiTab::Controls, "🎨 Lighting Controls");
                ui.selectable_value(&mut self.active_tab, GuiTab::Themes, "✨ Themes & Profiles");
                ui.selectable_value(&mut self.active_tab, GuiTab::Effects, "🌊 Hardware Effects");
                ui.selectable_value(&mut self.active_tab, GuiTab::Status, "⚙ Hardware & Info");
            });

            ui.separator();

            // Tab Content
            match self.active_tab {
                GuiTab::Controls => self.render_controls_tab(ui),
                GuiTab::Themes => self.render_themes_tab(ui),
                GuiTab::Effects => self.render_effects_tab(ui),
                GuiTab::Status => self.render_status_tab(ui),
            }

            ui.add_space(8.0);
            ui.separator();

            // Bottom Status Banner
            ui.horizontal(|ui| {
                ui.label(RichText::new(&self.status_banner).italics().size(12.0));
            });
        });
    }
}

impl MysticLightApp {
    fn render_controls_tab(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("24-Zone Physical Keyboard Matrix:").strong());
        ui.add_space(4.0);

        // Render 24-Zone Matrix (6 rows x 4 columns)
        let row_labels = [
            ("Row 1 (ESC..F12)", [0, 1, 2, 3]),
            ("Row 2 (`..BS)", [4, 5, 6, 7]),
            ("Row 3 (TAB..\\)", [8, 9, 10, 11]),
            ("Row 4 (CAPS..ENT)", [12, 13, 14, 15]),
            ("Row 5 (SHIFT..RSH)", [16, 17, 18, 19]),
            ("Row 6 (CTRL..NAV)", [20, 21, 22, 23]),
        ];

        let mut clicked_zone: Option<usize> = None;

        egui::Grid::new("matrix_grid")
            .spacing([6.0, 6.0])
            .show(ui, |ui| {
                for (r_idx, (r_name, zones)) in row_labels.iter().enumerate() {
                    ui.label(RichText::new(*r_name).size(12.0).color(Color32::from_rgb(180, 180, 180)));
                    for &z in zones {
                        let c = self.zone_colors[z];
                        let fill = Color32::from_rgb(c.r, c.g, c.b);
                        let is_selected = match self.target_mode {
                            TargetMode::AllZones => true,
                            TargetMode::Row(r) => r == (r_idx as u8 + 1),
                            TargetMode::Zone(target_z) => target_z == (z as u8),
                        };

                        let stroke = if is_selected {
                            Stroke::new(2.5_f32, Color32::WHITE)
                        } else {
                            Stroke::new(1.0_f32, Color32::from_rgb(60, 60, 60))
                        };

                        let text_color = if (c.r as u32 + c.g as u32 + c.b as u32) > 380 {
                            Color32::BLACK
                        } else {
                            Color32::WHITE
                        };

                        let btn = egui::Button::new(
                            RichText::new(format!("Z{:02}", z))
                                .size(11.0)
                                .color(text_color)
                                .strong(),
                        )
                        .min_size(Vec2::new(72.0, 28.0))
                        .fill(fill)
                        .stroke(stroke);

                        if ui.add(btn).clicked() {
                            clicked_zone = Some(z);
                        }
                    }
                    ui.end_row();
                }
            });

        if let Some(z) = clicked_zone {
            self.target_mode = TargetMode::Zone(z as u8);
            self.selected_zone = Some(z);
            let z_color = self.zone_colors[z];
            self.set_rgb(z_color);
        }

        ui.add_space(8.0);

        // Target Selection Quick Filters
        ui.horizontal(|ui| {
            ui.label(RichText::new("Target:").strong());
            if ui.selectable_label(self.target_mode == TargetMode::AllZones, "All 24 Zones").clicked() {
                self.target_mode = TargetMode::AllZones;
                self.selected_zone = None;
            }
            for r in 1..=6 {
                if ui.selectable_label(self.target_mode == TargetMode::Row(r), format!("Row {}", r)).clicked() {
                    self.target_mode = TargetMode::Row(r);
                    self.selected_zone = None;
                }
            }
        });

        ui.add_space(8.0);
        ui.separator();

        // Color & Brightness Controls
        ui.columns(2, |cols| {
            // Left Column: Sliders and Color Picker
            cols[0].label(RichText::new("Color Configuration:").strong());
            cols[0].add_space(4.0);

            let mut rgb_arr = self.color_rgb;
            if egui::color_picker::color_edit_button_rgb(&mut cols[0], &mut rgb_arr).changed() {
                self.color_rgb = rgb_arr;
                self.hex_input = format!("{}", self.current_rgb());
            }

            cols[0].add_space(4.0);
            cols[0].horizontal(|ui| {
                ui.label("Hex Code:");
                if ui.text_edit_singleline(&mut self.hex_input).changed() {
                    if let Ok(c) = Rgb::from_hex(&self.hex_input) {
                        self.color_rgb = [
                            c.r as f32 / 255.0,
                            c.g as f32 / 255.0,
                            c.b as f32 / 255.0,
                        ];
                    }
                }
            });

            cols[0].add_space(4.0);
            cols[0].horizontal(|ui| {
                ui.label("Brightness:");
                ui.add(egui::Slider::new(&mut self.brightness, 0..=10).text("Level (0..10)"));
            });

            cols[0].add_space(8.0);
            if cols[0].button(RichText::new("✔ Apply Color to Keyboard").strong().size(14.0)).clicked() {
                self.apply_current_color();
            }

            // Right Column: Palette Swatches
            cols[1].label(RichText::new("Preset Palette Swatches:").strong());
            cols[1].add_space(4.0);

            cols[1].horizontal_wrapped(|ui| {
                for &(name, (r, g, b)) in COLORS {
                    let swatch_c = Color32::from_rgb(r, g, b);
                    let label_c = if (r as u32 + g as u32 + b as u32) > 380 {
                        Color32::BLACK
                    } else {
                        Color32::WHITE
                    };
                    let btn = egui::Button::new(RichText::new(name).size(11.0).color(label_c))
                        .min_size(Vec2::new(76.0, 24.0))
                        .fill(swatch_c);

                    if ui.add(btn).clicked() {
                        self.set_rgb(Rgb::new(r, g, b));
                        self.apply_current_color();
                    }
                }
            });

            cols[1].add_space(8.0);
            if cols[1].button(RichText::new("⏹ Power Off All LEDs").color(Color32::from_rgb(255, 100, 100))).clicked() {
                if let Err(e) = self.ctrl.turn_off() {
                    self.status_banner = format!("Error powering off: {}", e);
                } else {
                    self.zone_colors = [Rgb::new(0, 0, 0); TOTAL_ZONES];
                    self.status_banner = "Keyboard LEDs powered off.".to_string();
                }
            }
        });
    }

    fn render_themes_tab(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Curated Themes (LightShow & OpenRGB Integration):").strong());
        ui.label("Apply studio-designed color profiles tuned for the 24-zone physical matrix.");
        ui.add_space(8.0);

        let mut apply_idx: Option<usize> = None;

        egui::ScrollArea::vertical().show(ui, |ui| {
            for (idx, theme) in self.themes.iter().enumerate() {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(theme.name).strong().size(14.0));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Apply Theme").clicked() {
                                apply_idx = Some(idx);
                            }
                        });
                    });

                    ui.label(RichText::new(theme.description).size(12.0).color(Color32::GRAY));
                    ui.add_space(4.0);

                    // Row of mini swatches showing the theme's colors
                    ui.horizontal(|ui| {
                        for (_z, c) in theme.zones.iter().enumerate().step_by(4) {
                            let rect_color = Color32::from_rgb(c.r, c.g, c.b);
                            let (rect, _) = ui.allocate_exact_size(Vec2::new(40.0, 14.0), egui::Sense::hover());
                            ui.painter().rect_filled(rect, 2.0, rect_color);
                        }
                    });
                });
                ui.add_space(4.0);
            }
        });

        if let Some(idx) = apply_idx {
            let theme = &self.themes[idx];
            if let Err(e) = theme.apply(&self.ctrl, self.brightness) {
                self.status_banner = format!("Error applying theme {}: {}", theme.name, e);
            } else {
                self.zone_colors = theme.zones;
                self.status_banner = format!("Theme '{}' applied successfully across all 6 rows.", theme.name);
            }
        }
    }

    fn render_effects_tab(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Hardware Microcontroller Animation Modes:").strong());
        ui.label("Animations execute on-chip inside the keyboard controller at 0.0% CPU overhead.");
        ui.add_space(10.0);

        ui.group(|ui| {
            ui.label(RichText::new("🌊 Rainbow Wave Effect").strong().size(14.0));
            ui.label("Chases dynamic rainbow hues smoothly across all 6 physical rows and 24 zones.");
            ui.horizontal(|ui| {
                ui.label("Cycle Duration:");
                ui.add(egui::Slider::new(&mut self.wave_speed, 50..=500).text("ms / keyframe"));
                if ui.button("Activate Wave").clicked() {
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
                        self.status_banner = format!("Error activating wave: {}", e);
                    } else {
                        self.status_banner = "Rainbow Wave activated on hardware across all 6 rows.".to_string();
                    }
                }
            });
        });

        ui.add_space(8.0);

        ui.group(|ui| {
            ui.label(RichText::new("🔄 Chromatic Spectrum Cycle").strong().size(14.0));
            ui.label("Rotates all 24 zones through the full color spectrum simultaneously.");
            ui.horizontal(|ui| {
                ui.label("Cycle Duration:");
                ui.add(egui::Slider::new(&mut self.cycle_speed, 50..=400).text("ms / keyframe"));
                if ui.button("Activate Cycle").clicked() {
                    let cycle_cols = vec![
                        (255, 0, 0),
                        (0, 255, 0),
                        (0, 120, 255),
                        (255, 255, 0),
                    ];
                    if let Err(e) = self.ctrl.set_effect(MODE_CYCLE, &cycle_cols, self.cycle_speed) {
                        self.status_banner = format!("Error activating cycle: {}", e);
                    } else {
                        self.status_banner = "Color Cycle activated across all 24 zones.".to_string();
                    }
                }
            });
        });

        ui.add_space(8.0);

        ui.group(|ui| {
            ui.label(RichText::new("🫁 Breathing Illumination").strong().size(14.0));
            ui.label("Rhythmically fades the active color in and out.");
            ui.horizontal(|ui| {
                ui.label("Speed:");
                ui.add(egui::Slider::new(&mut self.breathe_speed, 50..=300).text("ms"));
                if ui.button("Activate Breathing").clicked() {
                    let rgb = self.current_rgb();
                    let seq = vec![(0, 0, 0), (rgb.r, rgb.g, rgb.b), (0, 0, 0)];
                    if let Err(e) = self.ctrl.set_effect(MODE_BREATHING, &seq, self.breathe_speed) {
                        self.status_banner = format!("Error activating breathing: {}", e);
                    } else {
                        self.status_banner = format!("Breathing effect activated with {}.", rgb);
                    }
                }
            });
        });
    }

    fn render_status_tab(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Hardware & System Diagnostics:").strong());
        ui.add_space(6.0);

        ui.group(|ui| {
            ui.label(format!("Device Node: {}", self.device_info.node.as_deref().unwrap_or("None detected")));
            ui.label(format!("Hardware State: {}", if self.device_info.present { "Online (/dev/hidraw)" } else { "Not Found" }));
            ui.label(format!("Permissions: {}", if self.device_info.accessible { "Read/Write Access (OK)" } else { "Permission Denied (Install udev rule)" }));
            ui.label(format!("Hardware Topology: {} Physical Rows, {} Discrete Zones", self.device_info.total_rows, self.device_info.total_zones));
            ui.label("Driver Architecture: Linux HID raw feature report (HIDIOCSFEATURE_64)");
            ui.label(format!("Software Version: {}", VERSION));
        });

        ui.add_space(10.0);
        ui.label(RichText::new("Troubleshooting & Non-Root Access:").strong());
        ui.label("If permissions show 'Permission Denied', ensure the udev rule is installed:");
        ui.monospace("sudo cp packaging/99-msi-mysticlight.rules /etc/udev/rules.d/");
        ui.monospace("sudo udevadm control --reload-rules && sudo udevadm trigger");
    }
}

/// Run native GUI application.
pub fn run_gui() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([780.0, 620.0])
            .with_min_inner_size([650.0, 520.0])
            .with_title("MSI Mystic Light Controller"),
        ..Default::default()
    };

    eframe::run_native(
        "MSI Mystic Light",
        options,
        Box::new(|cc| Ok(Box::new(MysticLightApp::new(cc)))),
    )
}
