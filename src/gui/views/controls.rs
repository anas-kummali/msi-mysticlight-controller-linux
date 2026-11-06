//! Lighting & Color Studio view with gaming clusters and multi-zone targeting.

use std::collections::BTreeSet;
use eframe::egui::{self, Color32, RichText, Stroke, Vec2};
use crate::color::{Rgb, COLORS};
use crate::gui::style::{
    ACCENT_CYAN, ACCENT_GREEN, ACCENT_ORANGE, ACCENT_RED, BORDER_ACTIVE, BORDER_SUBTLE, CARD_BG,
    CARD_SELECTED, TEXT_MUTED, TEXT_PRIMARY,
};

pub fn render_controls_view(
    ui: &mut egui::Ui,
    color_rgb: &mut [f32; 3],
    hex_input: &mut String,
    selected_zones: &mut BTreeSet<usize>,
    on_apply: &mut bool,
) {
    // 1. Target Header
    ui.horizontal(|ui| {
        let (title, color) = if selected_zones.is_empty() {
            ("Target: Full Board (All 24 Zones)".to_string(), ACCENT_CYAN)
        } else if selected_zones.len() == 1 {
            let z = selected_zones.iter().next().unwrap();
            (format!("Target: Individual Zone Z{:02}", z), ACCENT_GREEN)
        } else {
            let list_str: Vec<String> = selected_zones.iter().map(|z| format!("Z{:02}", z)).collect();
            (format!("Target: {} Zones Selected ({})", selected_zones.len(), list_str.join(", ")), ACCENT_ORANGE)
        };
        ui.heading(RichText::new(title).size(15.0).color(color));

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if !selected_zones.is_empty()
                && ui.button(RichText::new("Clear Selection").size(11.0).color(ACCENT_RED)).clicked()
            {
                selected_zones.clear();
            }
        });
    });

    ui.add_space(6.0);

    // 2. Gaming Cluster Selection Toolbar
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new("Clusters:").size(11.5).strong().color(TEXT_MUTED));

        let clusters: &[(&str, &[usize])] = &[
            ("All (24)", &[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23]),
            ("WASD", &[8, 12]),
            ("Arrows", &[19, 23]),
            ("Numpad", &[11, 15, 19, 23]),
            ("QWERTY Alphas", &[8, 9, 10, 12, 13, 14, 16, 17, 18]),
            ("F1-F12", &[0, 1, 2, 3]),
            ("Number Row", &[4, 5, 6, 7]),
            ("Space & Mods", &[20, 21, 22]),
        ];

        for &(name, cluster_zones) in clusters {
            let is_matched = !selected_zones.is_empty()
                && cluster_zones.iter().all(|z| selected_zones.contains(z))
                && selected_zones.len() == cluster_zones.len();

            let btn = egui::Button::new(RichText::new(name).size(11.0).strong().color(if is_matched { ACCENT_CYAN } else { TEXT_PRIMARY }))
                .fill(if is_matched { CARD_SELECTED } else { CARD_BG })
                .stroke(if is_matched { Stroke::new(1.0_f32, BORDER_ACTIVE) } else { Stroke::new(1.0_f32, BORDER_SUBTLE) });

            if ui.add(btn).clicked() {
                selected_zones.clear();
                selected_zones.extend(cluster_zones.iter().copied());
            }
        }

        // Invert Button
        let invert_btn = egui::Button::new(RichText::new("Invert").size(11.0).color(TEXT_MUTED))
            .fill(CARD_BG)
            .stroke(Stroke::new(1.0_f32, BORDER_SUBTLE));
        if ui.add(invert_btn).clicked() {
            let mut inverted = BTreeSet::new();
            for z in 0..24 {
                if !selected_zones.contains(&z) {
                    inverted.insert(z);
                }
            }
            *selected_zones = inverted;
        }
    });

    ui.add_space(8.0);

    // 3. Two Columns: Color Channels & Gaming Swatches
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
                        *color_rgb = [c.r as f32 / 255.0, c.g as f32 / 255.0, c.b as f32 / 255.0];
                    }
                }
            });

            ui.add_space(4.0);

            // RGB Sliders
            let mut r = (color_rgb[0] * 255.0).round().clamp(0.0, 255.0) as u8;
            let mut g = (color_rgb[1] * 255.0).round().clamp(0.0, 255.0) as u8;
            let mut b = (color_rgb[2] * 255.0).round().clamp(0.0, 255.0) as u8;

            let mut changed = false;
            ui.horizontal(|ui| {
                ui.colored_label(Color32::from_rgb(255, 100, 100), "R");
                if ui.add(egui::Slider::new(&mut r, 0..=255)).changed() {
                    changed = true;
                }
            });
            ui.horizontal(|ui| {
                ui.colored_label(Color32::from_rgb(100, 255, 100), "G");
                if ui.add(egui::Slider::new(&mut g, 0..=255)).changed() {
                    changed = true;
                }
            });
            ui.horizontal(|ui| {
                ui.colored_label(Color32::from_rgb(100, 150, 255), "B");
                if ui.add(egui::Slider::new(&mut b, 0..=255)).changed() {
                    changed = true;
                }
            });

            if changed {
                *color_rgb = [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0];
                *hex_input = format!("#{:02X}{:02X}{:02X}", r, g, b);
            }

            ui.add_space(8.0);
            let apply_label = if selected_zones.is_empty() {
                "✔ Apply Color to Full Keyboard (24 Zones)".to_string()
            } else {
                format!("✔ Apply Color to {} Selected Zones", selected_zones.len())
            };

            let apply_btn = egui::Button::new(RichText::new(apply_label).strong().size(13.0))
                .fill(CARD_SELECTED)
                .stroke(Stroke::new(1.5_f32, BORDER_ACTIVE))
                .min_size(Vec2::new(ui.available_width(), 32.0));

            if ui.add(apply_btn).clicked() {
                *on_apply = true;
            }
        });

        // Right Column: Preset Gaming Swatches
        cols[1].group(|ui| {
            ui.label(RichText::new("Preset Gaming Swatches").strong());
            ui.label(RichText::new("Click any swatch to apply instantly to selected targets.").size(11.0).color(TEXT_MUTED));
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
