//! Lighting & Color Studio view.

use eframe::egui::{self, Color32, RichText, Stroke, Vec2};
use crate::color::{Rgb, COLORS};
use crate::gui::style::{ACCENT_CYAN, BORDER_ACTIVE, BORDER_SUBTLE, CARD_SELECTED, TEXT_MUTED};

pub fn render_controls_view(
    ui: &mut egui::Ui,
    color_rgb: &mut [f32; 3],
    hex_input: &mut String,
    _brightness: &mut u8,
    selected_zone: Option<usize>,
    selected_row: Option<u8>,
    on_apply: &mut bool,
) {
    ui.horizontal(|ui| {
        let target_title = if let Some(z) = selected_zone {
            format!("Target: Individual Zone Z{:02}", z)
        } else if let Some(r) = selected_row {
            format!("Target: Physical Row {}", r)
        } else {
            "Target: Full Keyboard (All 24 Zones)".to_string()
        };
        ui.heading(RichText::new(target_title).size(15.0).color(ACCENT_CYAN));
    });

    ui.add_space(6.0);

    ui.columns(2, |cols| {
        // Left Column: Interactive Color Wheel, RGB Sliders & Hex Input
        cols[0].group(|ui| {
            ui.label(RichText::new("Color Palette & Channels").strong());
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("Pick Color:");
                if egui::color_picker::color_edit_button_rgb(ui, color_rgb).changed() {
                    let r = (color_rgb[0] * 255.0).round().clamp(0.0, 255.0) as u8;
                    let g = (color_rgb[1] * 255.0).round().clamp(0.0, 255.0) as u8;
                    let b = (color_rgb[2] * 255.0).round().clamp(0.0, 255.0) as u8;
                    *hex_input = format!("#{:02X}{:02X}{:02X}", r, g, b);
                }
            });

            ui.add_space(4.0);

            // Hex Code
            ui.horizontal(|ui| {
                ui.label("Hex Code:");
                if ui.text_edit_singleline(hex_input).changed() {
                    if let Ok(c) = Rgb::from_hex(hex_input) {
                        *color_rgb = [
                            c.r as f32 / 255.0,
                            c.g as f32 / 255.0,
                            c.b as f32 / 255.0,
                        ];
                    }
                }
            });

            ui.add_space(4.0);

            // Sliders for R, G, B
            let mut r = (color_rgb[0] * 255.0) as u8;
            let mut g = (color_rgb[1] * 255.0) as u8;
            let mut b = (color_rgb[2] * 255.0) as u8;

            let mut changed = false;
            ui.horizontal(|ui| {
                ui.label(RichText::new("R").color(Color32::from_rgb(255, 90, 90)));
                changed |= ui.add(egui::Slider::new(&mut r, 0..=255)).changed();
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new("G").color(Color32::from_rgb(90, 255, 90)));
                changed |= ui.add(egui::Slider::new(&mut g, 0..=255)).changed();
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new("B").color(Color32::from_rgb(90, 160, 255)));
                changed |= ui.add(egui::Slider::new(&mut b, 0..=255)).changed();
            });

            if changed {
                *color_rgb = [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0];
                *hex_input = format!("#{:02X}{:02X}{:02X}", r, g, b);
            }

            ui.add_space(8.0);
            let apply_btn = egui::Button::new(
                RichText::new("✔ Apply Color to Selected Target")
                    .strong()
                    .size(13.5),
            )
            .fill(CARD_SELECTED)
            .stroke(Stroke::new(1.5_f32, BORDER_ACTIVE))
            .min_size(Vec2::new(ui.available_width(), 32.0));

            if ui.add(apply_btn).clicked() {
                *on_apply = true;
            }
        });

        // Right Column: Preset Palette Swatches
        cols[1].group(|ui| {
            ui.label(RichText::new("Preset Gaming Swatches").strong());
            ui.label(RichText::new("Click any swatch to apply instantly.").size(11.0).color(TEXT_MUTED));
            ui.add_space(6.0);

            ui.horizontal_wrapped(|ui| {
                for &(name, (pr, pg, pb)) in COLORS {
                    let fill_color = Color32::from_rgb(pr, pg, pb);
                    let label_color = if (pr as u32 + pg as u32 + pb as u32) > 380 {
                        Color32::BLACK
                    } else {
                        Color32::WHITE
                    };

                    let btn = egui::Button::new(
                        RichText::new(name)
                            .size(11.0)
                            .color(label_color)
                            .strong(),
                    )
                    .fill(fill_color)
                    .stroke(Stroke::new(1.0_f32, BORDER_SUBTLE))
                    .min_size(Vec2::new(74.0, 26.0));

                    if ui.add(btn).clicked() {
                        *color_rgb = [pr as f32 / 255.0, pg as f32 / 255.0, pb as f32 / 255.0];
                        *hex_input = format!("#{:02X}{:02X}{:02X}", pr, pg, pb);
                        *on_apply = true;
                    }
                }
            });
        });
    });
}
