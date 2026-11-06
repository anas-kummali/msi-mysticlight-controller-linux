//! Gradient Studio view for generating smooth multi-zone color gradients.

use eframe::egui::{self, Color32, Pos2, Rect, RichText, Rounding, Stroke, Vec2};
use crate::color::{Rgb, COLORS};
use crate::gradient::{generate_gradient, GradientDirection};
use crate::gui::style::{
    ACCENT_CYAN, ACCENT_GREEN, BORDER_ACTIVE, BORDER_SUBTLE, CARD_BG, CARD_SELECTED,
    TEXT_MUTED, TEXT_PRIMARY,
};

pub fn render_gradient_view(
    ui: &mut egui::Ui,
    start_rgb: &mut [f32; 3],
    end_rgb: &mut [f32; 3],
    direction: &mut GradientDirection,
    on_apply_gradient: &mut bool,
) {
    ui.horizontal(|ui| {
        ui.heading(RichText::new("Gradient Studio").size(16.0).color(TEXT_PRIMARY));
        ui.label(RichText::new("·  Smooth mathematical spatial color interpolation across all 24 zones").size(12.0).color(TEXT_MUTED));
    });

    ui.add_space(8.0);

    ui.columns(2, |cols| {
        // Left Column: Start & End Color Pickers
        cols[0].group(|ui| {
            ui.label(RichText::new("Gradient Color Endpoints").strong());
            ui.add_space(4.0);

            // Start Color
            ui.horizontal(|ui| {
                ui.label(RichText::new("Start Color:").strong().color(ACCENT_CYAN));
                egui::color_picker::color_edit_button_rgb(ui, start_rgb);
                let sr = (start_rgb[0] * 255.0).round() as u8;
                let sg = (start_rgb[1] * 255.0).round() as u8;
                let sb = (start_rgb[2] * 255.0).round() as u8;
                ui.monospace(format!("#{:02X}{:02X}{:02X}", sr, sg, sb));
            });

            // Quick Start Swatches
            ui.horizontal_wrapped(|ui| {
                for &(name, (r, g, b)) in &COLORS[..6] {
                    let btn = egui::Button::new(RichText::new(name).size(10.0))
                        .fill(Color32::from_rgb(r, g, b))
                        .min_size(Vec2::new(48.0, 18.0));
                    if ui.add(btn).clicked() {
                        *start_rgb = [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0];
                    }
                }
            });

            ui.add_space(8.0);

            // End Color
            ui.horizontal(|ui| {
                ui.label(RichText::new("End Color:  ").strong().color(ACCENT_GREEN));
                egui::color_picker::color_edit_button_rgb(ui, end_rgb);
                let er = (end_rgb[0] * 255.0).round() as u8;
                let eg = (end_rgb[1] * 255.0).round() as u8;
                let eb = (end_rgb[2] * 255.0).round() as u8;
                ui.monospace(format!("#{:02X}{:02X}{:02X}", er, eg, eb));
            });

            // Quick End Swatches
            ui.horizontal_wrapped(|ui| {
                for &(name, (r, g, b)) in &COLORS[6..] {
                    let btn = egui::Button::new(RichText::new(name).size(10.0))
                        .fill(Color32::from_rgb(r, g, b))
                        .min_size(Vec2::new(48.0, 18.0));
                    if ui.add(btn).clicked() {
                        *end_rgb = [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0];
                    }
                }
            });

            ui.add_space(8.0);
            if ui.button(RichText::new("⇄ Swap Start & End Colors").size(11.5)).clicked() {
                std::mem::swap(start_rgb, end_rgb);
            }
        });

        // Right Column: Spatial Direction & Live Preview
        cols[1].group(|ui| {
            ui.label(RichText::new("Spatial Direction & Interpolation").strong());
            ui.add_space(4.0);

            for dir in GradientDirection::ALL {
                let is_selected = *direction == *dir;
                let btn = egui::Button::new(
                    RichText::new(dir.label())
                        .size(12.0)
                        .color(if is_selected { ACCENT_CYAN } else { TEXT_PRIMARY }),
                )
                .fill(if is_selected { CARD_SELECTED } else { CARD_BG })
                .stroke(if is_selected { Stroke::new(1.5_f32, BORDER_ACTIVE) } else { Stroke::new(1.0_f32, BORDER_SUBTLE) })
                .min_size(Vec2::new(ui.available_width(), 26.0));

                if ui.add(btn).clicked() {
                    *direction = *dir;
                }
            }

            ui.add_space(8.0);

            // Live Computed Matrix Preview Ribbon
            ui.label(RichText::new("Matrix Preview (6 Rows × 4 Columns):").size(11.5).color(TEXT_MUTED));
            let s_rgb = Rgb::new(
                (start_rgb[0] * 255.0).round() as u8,
                (start_rgb[1] * 255.0).round() as u8,
                (start_rgb[2] * 255.0).round() as u8,
            );
            let e_rgb = Rgb::new(
                (end_rgb[0] * 255.0).round() as u8,
                (end_rgb[1] * 255.0).round() as u8,
                (end_rgb[2] * 255.0).round() as u8,
            );
            let preview_zones = generate_gradient(s_rgb, e_rgb, *direction);

            let (response, painter) = ui.allocate_painter(Vec2::new(ui.available_width(), 24.0), egui::Sense::hover());
            let rect = response.rect;
            let col_w = rect.width() / 4.0;

            for r in 0..6 {
                for c in 0..4 {
                    let z = r * 4 + c;
                    let color = Color32::from_rgb(preview_zones[z].r, preview_zones[z].g, preview_zones[z].b);
                    let seg = Rect::from_min_size(
                        Pos2::new(rect.min.x + (c as f32) * col_w, rect.min.y + (r as f32) * 4.0),
                        Vec2::new(col_w, 3.5),
                    );
                    painter.rect_filled(seg, Rounding::ZERO, color);
                }
            }

            ui.add_space(8.0);

            let apply_btn = egui::Button::new(
                RichText::new("🌈 Apply Gradient to All 24 Zones")
                    .strong()
                    .size(13.5),
            )
            .fill(CARD_SELECTED)
            .stroke(Stroke::new(1.5_f32, BORDER_ACTIVE))
            .min_size(Vec2::new(ui.available_width(), 32.0));

            if ui.add(apply_btn).clicked() {
                *on_apply_gradient = true;
            }
        });
    });
}
