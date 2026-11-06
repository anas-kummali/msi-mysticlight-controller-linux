//! Interactive 24-Zone Physical Keyboard Canvas Renderer.

use std::collections::BTreeSet;
use eframe::egui::{self, Color32, Pos2, Rect, RichText, Rounding, Sense, Stroke, Vec2};
use crate::color::Rgb;
use crate::gui::style::{ACCENT_CYAN, BORDER_SUBTLE, CARD_BG, PANEL_BG, TEXT_MUTED};

/// Key representation in the physical matrix.
pub struct MatrixKey {
    pub label: &'static str,
    pub sub_label: &'static str,
    pub zone: usize,
    pub width_weight: f32, // Proportional width for natural laptop layout
}

/// Physical keyboard layout definition with authentic laptop stagger proportions.
pub const KEYBOARD_ROWS: [&[MatrixKey]; 6] = [
    // Row 1: Function Keys (ESC to DEL)
    &[
        MatrixKey { label: "ESC · F1-F4", sub_label: "Z00", zone: 0, width_weight: 1.0 },
        MatrixKey { label: "F5 - F8", sub_label: "Z01", zone: 1, width_weight: 1.0 },
        MatrixKey { label: "F9 - F12", sub_label: "Z02", zone: 2, width_weight: 1.0 },
        MatrixKey { label: "PRTSCR · DEL", sub_label: "Z03", zone: 3, width_weight: 1.0 },
    ],
    // Row 2: Number Row (` to BACKSPACE)
    &[
        MatrixKey { label: "` · 1 - 4", sub_label: "Z04", zone: 4, width_weight: 1.0 },
        MatrixKey { label: "5 - 8", sub_label: "Z05", zone: 5, width_weight: 1.0 },
        MatrixKey { label: "9 - 0 · - =", sub_label: "Z06", zone: 6, width_weight: 1.0 },
        MatrixKey { label: "BACKSPACE", sub_label: "Z07", zone: 7, width_weight: 1.0 },
    ],
    // Row 3: Tab & Upper Letters (TAB to \)
    &[
        MatrixKey { label: "TAB · Q - R", sub_label: "Z08", zone: 8, width_weight: 1.08 },
        MatrixKey { label: "T - U · I", sub_label: "Z09", zone: 9, width_weight: 0.92 },
        MatrixKey { label: "O - P · [ ]", sub_label: "Z10", zone: 10, width_weight: 0.92 },
        MatrixKey { label: "\\ | · NUM 7-9", sub_label: "Z11", zone: 11, width_weight: 1.08 },
    ],
    // Row 4: Caps & Home Row (CAPS to ENTER)
    &[
        MatrixKey { label: "CAPS · A - F", sub_label: "Z12", zone: 12, width_weight: 1.14 },
        MatrixKey { label: "G - J · K", sub_label: "Z13", zone: 13, width_weight: 0.86 },
        MatrixKey { label: "L · ; '", sub_label: "Z14", zone: 14, width_weight: 0.86 },
        MatrixKey { label: "ENTER · NUM 4-6", sub_label: "Z15", zone: 15, width_weight: 1.14 },
    ],
    // Row 5: Shifts & Lower Row (SHIFT to SHIFT)
    &[
        MatrixKey { label: "LSHIFT · Z - V", sub_label: "Z16", zone: 16, width_weight: 1.20 },
        MatrixKey { label: "B - M", sub_label: "Z17", zone: 17, width_weight: 0.80 },
        MatrixKey { label: ", . / · RSHIFT", sub_label: "Z18", zone: 18, width_weight: 0.90 },
        MatrixKey { label: "UP · NUM 1-3", sub_label: "Z19", zone: 19, width_weight: 1.10 },
    ],
    // Row 6: Spacebar & Modifiers (CTRL to Nav)
    &[
        MatrixKey { label: "CTRL · FN · ALT", sub_label: "Z20", zone: 20, width_weight: 1.05 },
        MatrixKey { label: "SPACEBAR", sub_label: "Z21", zone: 21, width_weight: 1.30 },
        MatrixKey { label: "ALT · MENU · CTRL", sub_label: "Z22", zone: 22, width_weight: 0.80 },
        MatrixKey { label: "ARROWS · NUM 0", sub_label: "Z23", zone: 23, width_weight: 0.85 },
    ],
];

/// Render the interactive physical keyboard graphic on screen.
/// Returns `Some((clicked_zone, is_multi_select))` if a key was clicked.
pub fn render_keyboard_canvas(
    ui: &mut egui::Ui,
    zone_colors: &[Rgb; 24],
    selected_zones: &BTreeSet<usize>,
) -> Option<(usize, bool)> {
    let mut click_event = None;
    let available_width = ui.available_width().max(600.0);
    let key_height = 36.0;
    let spacing = 5.0;
    let canvas_height = (key_height + spacing) * 6.0 + 20.0;

    let (response, painter) = ui.allocate_painter(
        Vec2::new(available_width, canvas_height),
        Sense::click(),
    );

    let canvas_rect = response.rect;

    // Outer Laptop Chassis
    painter.rect_filled(canvas_rect, Rounding::same(10.0), PANEL_BG);
    painter.rect_stroke(
        canvas_rect,
        Rounding::same(10.0),
        Stroke::new(1.5_f32, BORDER_SUBTLE),
    );

    let content_x = canvas_rect.min.x + 10.0;
    let content_w = canvas_rect.width() - 20.0;
    let start_y = canvas_rect.min.y + 10.0;

    for (r_idx, row_keys) in KEYBOARD_ROWS.iter().enumerate() {
        let y = start_y + (r_idx as f32) * (key_height + spacing);
        let total_weight: f32 = row_keys.iter().map(|k| k.width_weight).sum();
        let gap_total = spacing * (row_keys.len() as f32 - 1.0);
        let usable_w = content_w - gap_total;

        let mut current_x = content_x;

        for key in *row_keys {
            let key_w = usable_w * (key.width_weight / total_weight);
            let key_rect = Rect::from_min_size(Pos2::new(current_x, y), Vec2::new(key_w, key_height));

            let rgb = zone_colors[key.zone];
            let is_selected = selected_zones.contains(&key.zone);

            let key_response = ui.allocate_rect(key_rect, Sense::click());
            let is_hovered = key_response.hovered();

            // Rich interactive tooltip on hover
            key_response.clone().on_hover_ui(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(key.label).strong().color(Color32::WHITE));
                    ui.colored_label(ACCENT_CYAN, format!("Zone Z{:02}", key.zone));
                });
                ui.label(RichText::new(format!("Physical Row {}", r_idx + 1)).size(11.0).color(TEXT_MUTED));
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Color:").size(11.0));
                    ui.colored_label(Color32::from_rgb(rgb.r, rgb.g, rgb.b), format!("{}", rgb));
                });
            });

            if key_response.clicked() {
                let is_multi = ui.input(|i| i.modifiers.shift || i.modifiers.command || i.modifiers.ctrl);
                click_event = Some((key.zone, is_multi));
            }

            // 1. LED Underglow Halo (Soft chromatic ambient light around keycap)
            let glow_rect = key_rect.expand(if is_selected { 3.0 } else { 2.0 });
            let glow_alpha = if is_selected { 160 } else if is_hovered { 120 } else { 80 };
            let glow_color = Color32::from_rgba_premultiplied(rgb.r, rgb.g, rgb.b, glow_alpha);
            painter.rect_filled(glow_rect, Rounding::same(8.0), glow_color);

            // 2. Keycap Body (Sleek dark finish)
            let keycap_color = if is_selected {
                Color32::from_rgb(40, 52, 85)
            } else if is_hovered {
                Color32::from_rgb(45, 50, 75)
            } else {
                CARD_BG
            };
            painter.rect_filled(key_rect, Rounding::same(6.0), keycap_color);

            // 3. Selection / Status Border
            let stroke = if is_selected {
                Stroke::new(2.5_f32, ACCENT_CYAN)
            } else if is_hovered {
                Stroke::new(1.5_f32, Color32::WHITE)
            } else {
                Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(rgb.r, rgb.g, rgb.b, 100))
            };
            painter.rect_stroke(key_rect, Rounding::same(6.0), stroke);

            // 4. Live RGB Color Bar on Keycap Top Edge
            let rgb_bar = Rect::from_min_size(
                Pos2::new(key_rect.min.x + 4.0, key_rect.min.y + 3.0),
                Vec2::new(key_rect.width() - 8.0, 3.5),
            );
            painter.rect_filled(
                rgb_bar,
                Rounding::same(2.0),
                Color32::from_rgb(rgb.r, rgb.g, rgb.b),
            );

            // 5. Key Label Text
            let text_color = if is_selected { Color32::WHITE } else { TEXT_MUTED };
            let label_pos = Pos2::new(key_rect.min.x + 6.0, key_rect.min.y + 8.0);
            painter.text(
                label_pos,
                egui::Align2::LEFT_TOP,
                key.label,
                egui::FontId::proportional(10.0),
                text_color,
            );

            // 6. Zone Identifier Badge (Bottom Right)
            let badge_pos = Pos2::new(key_rect.max.x - 6.0, key_rect.max.y - 6.0);
            painter.text(
                badge_pos,
                egui::Align2::RIGHT_BOTTOM,
                key.sub_label,
                egui::FontId::monospace(9.0),
                Color32::from_rgba_premultiplied(rgb.r, rgb.g, rgb.b, 230),
            );

            current_x += key_w + spacing;
        }
    }

    click_event
}
