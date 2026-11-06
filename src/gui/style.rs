//! Custom Design System and Visual Theme (Tokyo Night / Adwaita Dark).

use eframe::egui::{self, Color32, Margin, Rounding, Stroke, Visuals};

pub const BG_DARK: Color32 = Color32::from_rgb(19, 20, 31);
pub const PANEL_BG: Color32 = Color32::from_rgb(26, 29, 45);
pub const CARD_BG: Color32 = Color32::from_rgb(34, 38, 59);
pub const CARD_HOVER: Color32 = Color32::from_rgb(44, 49, 76);
pub const CARD_SELECTED: Color32 = Color32::from_rgb(38, 48, 80);

pub const ACCENT_CYAN: Color32 = Color32::from_rgb(0, 240, 255);
pub const ACCENT_BLUE: Color32 = Color32::from_rgb(122, 162, 247);
pub const ACCENT_PURPLE: Color32 = Color32::from_rgb(187, 154, 247);
pub const ACCENT_GREEN: Color32 = Color32::from_rgb(158, 206, 106);
pub const ACCENT_ORANGE: Color32 = Color32::from_rgb(255, 158, 100);
pub const ACCENT_RED: Color32 = Color32::from_rgb(247, 118, 142);

pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(240, 243, 255);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(140, 150, 185);
pub const BORDER_SUBTLE: Color32 = Color32::from_rgb(46, 52, 78);
pub const BORDER_ACTIVE: Color32 = Color32::from_rgb(0, 220, 255);

/// Configure high-contrast, modern dark visuals for the egui Context.
pub fn configure_visuals(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();

    visuals.override_text_color = Some(TEXT_PRIMARY);
    visuals.panel_fill = BG_DARK;
    visuals.window_fill = PANEL_BG;
    visuals.faint_bg_color = PANEL_BG;
    visuals.extreme_bg_color = BG_DARK;

    visuals.widgets.noninteractive.bg_fill = PANEL_BG;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, BORDER_SUBTLE);
    visuals.widgets.noninteractive.rounding = Rounding::same(8.0);

    visuals.widgets.inactive.bg_fill = CARD_BG;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, BORDER_SUBTLE);
    visuals.widgets.inactive.rounding = Rounding::same(6.0);

    visuals.widgets.hovered.bg_fill = CARD_HOVER;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.5_f32, ACCENT_BLUE);
    visuals.widgets.hovered.rounding = Rounding::same(6.0);

    visuals.widgets.active.bg_fill = CARD_SELECTED;
    visuals.widgets.active.bg_stroke = Stroke::new(2.0_f32, ACCENT_CYAN);
    visuals.widgets.active.rounding = Rounding::same(6.0);

    visuals.selection.bg_fill = Color32::from_rgba_premultiplied(0, 240, 255, 45);
    visuals.selection.stroke = Stroke::new(1.5_f32, ACCENT_CYAN);

    visuals.window_rounding = Rounding::same(12.0);
    visuals.menu_rounding = Rounding::same(8.0);

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 7.0);
    style.spacing.window_margin = Margin::same(12.0);
    ctx.set_style(style);
}
