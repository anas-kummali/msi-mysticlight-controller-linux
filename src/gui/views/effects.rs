//! Hardware & Software Animation Effects Gallery.

use eframe::egui::{self, RichText, Rounding, Stroke};
use crate::gui::style::{
    ACCENT_CYAN, ACCENT_GREEN, ACCENT_ORANGE, BORDER_ACTIVE, BORDER_SUBTLE, CARD_BG, CARD_HOVER,
    CARD_SELECTED, TEXT_MUTED, TEXT_PRIMARY,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveEffect {
    None,
    Wave,
    Cycle,
    Breathe,
    Off,
}

pub struct EffectItem {
    pub id: ActiveEffect,
    pub title: &'static str,
    pub icon: &'static str,
    pub is_hardware: bool,
    pub description: &'static str,
    pub default_speed: u16,
    pub speed_unit: &'static str,
}

pub const EFFECTS_LIST: &[EffectItem] = &[
    EffectItem {
        id: ActiveEffect::Wave,
        title: "RAINBOW WAVE",
        icon: "🌊",
        is_hardware: true,
        description: "Smooth chromatic wave sweeping horizontally across all 6 physical rows.",
        default_speed: 200,
        speed_unit: "ms/step",
    },
    EffectItem {
        id: ActiveEffect::Cycle,
        title: "SPECTRUM CYCLE",
        icon: "🔄",
        is_hardware: true,
        description: "Simultaneously cycles all 24 zones through the 360° RGB color spectrum.",
        default_speed: 150,
        speed_unit: "ms/step",
    },
    EffectItem {
        id: ActiveEffect::Breathe,
        title: "BREATHING PULSE",
        icon: "🫁",
        is_hardware: true,
        description: "Gently pulses the active color in and out in a calm rhythmic cadence.",
        default_speed: 120,
        speed_unit: "ms/fade",
    },
    EffectItem {
        id: ActiveEffect::Off,
        title: "POWER OFF",
        icon: "⏹",
        is_hardware: true,
        description: "Completely cuts microcontroller LED power across all 24 zones (0.0% brightness).",
        default_speed: 0,
        speed_unit: "",
    },
];

pub fn render_effects_view(
    ui: &mut egui::Ui,
    current_effect: ActiveEffect,
    wave_speed: &mut u16,
    cycle_speed: &mut u16,
    breathe_speed: &mut u16,
    trigger_effect: &mut Option<ActiveEffect>,
) {
    ui.horizontal(|ui| {
        ui.heading(RichText::new("Hardware Effects Gallery").size(16.0).color(TEXT_PRIMARY));
        ui.label(RichText::new("·  Zero CPU on-chip microcontroller execution").size(12.0).color(TEXT_MUTED));
    });

    ui.add_space(8.0);

    ui.columns(2, |cols| {
        for (idx, item) in EFFECTS_LIST.iter().enumerate() {
            let col = &mut cols[idx % 2];
            let is_active = current_effect == item.id;
            let bg = if is_active { CARD_SELECTED } else { CARD_BG };
            let border_stroke = if is_active {
                Stroke::new(2.0_f32, BORDER_ACTIVE)
            } else {
                Stroke::new(1.0_f32, BORDER_SUBTLE)
            };

            egui::Frame::none()
                .fill(bg)
                .stroke(border_stroke)
                .rounding(Rounding::same(8.0))
                .inner_margin(12.0)
                .show(col, |ui| {
                    // Card Header: Icon + Title + HW Badge
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(item.icon).size(16.0));
                        ui.label(RichText::new(item.title).strong().size(13.5).color(if is_active { ACCENT_CYAN } else { TEXT_PRIMARY }));

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let badge_text = if item.is_hardware { "HW · 0% CPU" } else { "SW" };
                            let badge_color = if item.is_hardware { ACCENT_GREEN } else { ACCENT_ORANGE };
                            ui.colored_label(badge_color, badge_text);
                        });
                    });

                    ui.add_space(4.0);
                    ui.add(egui::Label::new(RichText::new(item.description).size(11.5).color(TEXT_MUTED)).wrap());
                    ui.add_space(8.0);

                    // Speed Slider & Action Button
                    ui.horizontal(|ui| {
                        match item.id {
                            ActiveEffect::Wave => {
                                ui.add(egui::Slider::new(wave_speed, 50..=500).text(item.speed_unit));
                            }
                            ActiveEffect::Cycle => {
                                ui.add(egui::Slider::new(cycle_speed, 50..=400).text(item.speed_unit));
                            }
                            ActiveEffect::Breathe => {
                                ui.add(egui::Slider::new(breathe_speed, 50..=300).text(item.speed_unit));
                            }
                            ActiveEffect::Off => {
                                ui.label(RichText::new("Cuts power to all rows").size(11.0).color(TEXT_MUTED));
                            }
                            _ => {}
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let btn_label = if is_active { "Active ●" } else { "Activate" };
                            let btn = egui::Button::new(RichText::new(btn_label).strong())
                                .fill(if is_active { ACCENT_CYAN } else { CARD_HOVER })
                                .stroke(if is_active { Stroke::NONE } else { Stroke::new(1.0_f32, BORDER_SUBTLE) });

                            if ui.add(btn).clicked() {
                                *trigger_effect = Some(item.id);
                            }
                        });
                    });
                });

            col.add_space(10.0);
        }
    });
}
