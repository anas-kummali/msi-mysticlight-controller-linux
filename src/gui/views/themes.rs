//! Theme Studio & Profile Management.

use eframe::egui::{self, Color32, Pos2, Rect, RichText, Rounding, Stroke, Vec2};
use crate::gui::style::{ACCENT_CYAN, BORDER_ACTIVE, BORDER_SUBTLE, CARD_BG, CARD_HOVER, CARD_SELECTED, TEXT_MUTED, TEXT_PRIMARY};
use crate::theme::Theme;

pub fn render_themes_view(
    ui: &mut egui::Ui,
    themes: &[Theme],
    selected_theme: Option<&'static str>,
    on_select_theme: &mut Option<usize>,
) {
    ui.horizontal(|ui| {
        ui.heading(RichText::new("Theme Studio & Palettes").size(16.0).color(TEXT_PRIMARY));
        ui.label(RichText::new("·  Curated OpenRGB & LightShow cross-row profiles").size(12.0).color(TEXT_MUTED));
    });

    ui.add_space(8.0);

    ui.columns(2, |cols| {
        for (idx, theme) in themes.iter().enumerate() {
            let col = &mut cols[idx % 2];
            let is_active = selected_theme == Some(theme.id);
            let border_stroke = if is_active {
                Stroke::new(2.0_f32, BORDER_ACTIVE)
            } else {
                Stroke::new(1.0_f32, BORDER_SUBTLE)
            };
            let bg = if is_active { CARD_SELECTED } else { CARD_BG };

            egui::Frame::none()
                .fill(bg)
                .stroke(border_stroke)
                .rounding(Rounding::same(8.0))
                .inner_margin(12.0)
                .show(col, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(theme.name).strong().size(14.0).color(if is_active { ACCENT_CYAN } else { TEXT_PRIMARY }));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let btn_label = if is_active { "Applied ●" } else { "Apply" };
                            let btn = egui::Button::new(RichText::new(btn_label).strong())
                                .fill(if is_active { ACCENT_CYAN } else { CARD_HOVER });
                            if ui.add(btn).clicked() {
                                *on_select_theme = Some(idx);
                            }
                        });
                    });

                    ui.add_space(2.0);
                    ui.add(egui::Label::new(RichText::new(theme.description).size(11.5).color(TEXT_MUTED)).wrap());
                    ui.add_space(8.0);

                    // Full 6-row chromatic ribbon preview
                    let (response, painter) = ui.allocate_painter(Vec2::new(ui.available_width(), 16.0), egui::Sense::hover());
                    let rect = response.rect;
                    let row_w = rect.width() / 6.0;

                    for r in 0..6 {
                        let c = theme.zones[r * 4];
                        let color = Color32::from_rgb(c.r, c.g, c.b);
                        let segment = Rect::from_min_size(
                            Pos2::new(rect.min.x + (r as f32) * row_w, rect.min.y),
                            Vec2::new(row_w, 16.0),
                        );
                        painter.rect_filled(segment, Rounding::same(2.0), color);
                    }
                });

            col.add_space(10.0);
        }
    });
}
