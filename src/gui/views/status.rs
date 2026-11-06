//! Hardware Diagnostics and System Status.

use eframe::egui::{self, RichText, Vec2};
use crate::device::DeviceInfo;
use crate::gui::style::{ACCENT_GREEN, ACCENT_ORANGE, TEXT_MUTED, TEXT_PRIMARY};
use crate::VERSION;

pub fn render_status_view(ui: &mut egui::Ui, info: &DeviceInfo) {
    ui.horizontal(|ui| {
        ui.heading(RichText::new("Hardware & System Diagnostics").size(16.0).color(TEXT_PRIMARY));
        ui.label(RichText::new("·  Kernel hidraw and permissions inspector").size(12.0).color(TEXT_MUTED));
    });

    ui.add_space(8.0);

    ui.group(|ui| {
        ui.set_min_size(Vec2::new(ui.available_width(), 160.0));

        egui::Grid::new("diag_grid")
            .num_columns(2)
            .spacing([24.0, 8.0])
            .show(ui, |ui| {
                ui.label(RichText::new("Detected Device Node:").strong());
                ui.label(info.node.as_deref().unwrap_or("None detected (/dev/hidraw*)"));
                ui.end_row();

                ui.label(RichText::new("Hardware Status:").strong());
                if info.present {
                    ui.colored_label(ACCENT_GREEN, "● Online (Ready for Feature Reports)");
                } else {
                    ui.colored_label(ACCENT_ORANGE, "● Disconnected / Not Found");
                }
                ui.end_row();

                ui.label(RichText::new("Udev Permissions:").strong());
                if info.accessible {
                    ui.colored_label(ACCENT_GREEN, "● Read/Write Non-Root Access Verified (0660 + uaccess)");
                } else {
                    ui.colored_label(ACCENT_ORANGE, "● Permission Denied (Udev rule required)");
                }
                ui.end_row();

                ui.label(RichText::new("Lighting Matrix Topology:").strong());
                ui.label(format!("{} Physical Rows  ×  4 Zones = {} Total Addressable Zones", info.total_rows, info.total_zones));
                ui.end_row();

                ui.label(RichText::new("Kernel Driver Mechanism:").strong());
                ui.label("Linux POSIX ioctl (HIDIOCSFEATURE_64 / 0xC0404806)");
                ui.end_row();

                ui.label(RichText::new("Driver Suite Version:").strong());
                ui.label(format!("v{}", VERSION));
                ui.end_row();
            });
    });

    ui.add_space(10.0);

    ui.group(|ui| {
        ui.label(RichText::new("Udev Rule & Non-Root Setup Guide").strong());
        ui.add_space(4.0);
        ui.label(RichText::new("If permissions show 'Permission Denied', run the following commands:").size(11.5).color(TEXT_MUTED));
        ui.add_space(4.0);
        ui.monospace("sudo cp packaging/99-msi-mysticlight.rules /etc/udev/rules.d/");
        ui.monospace("sudo udevadm control --reload-rules && sudo udevadm trigger");
    });
}
