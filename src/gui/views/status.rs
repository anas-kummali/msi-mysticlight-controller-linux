//! Hardware Diagnostics and Laptop Power Inspector.

use eframe::egui::{self, RichText, Vec2};
use crate::device::DeviceInfo;
use crate::gui::style::{ACCENT_CYAN, ACCENT_GREEN, ACCENT_ORANGE, TEXT_MUTED, TEXT_PRIMARY};
use crate::power::PowerInfo;
use crate::VERSION;

pub fn render_status_view(ui: &mut egui::Ui, info: &DeviceInfo, power: &PowerInfo) {
    ui.horizontal(|ui| {
        ui.heading(RichText::new("Hardware & System Diagnostics").size(16.0).color(TEXT_PRIMARY));
        ui.label(RichText::new("·  Kernel hidraw, permissions & laptop battery telemetry").size(12.0).color(TEXT_MUTED));
    });

    ui.add_space(8.0);

    // 1. Controller & Hidraw Hardware Card
    ui.group(|ui| {
        ui.set_min_size(Vec2::new(ui.available_width(), 160.0));
        ui.label(RichText::new("USB HID Controller & Topology").strong().color(ACCENT_CYAN));
        ui.add_space(4.0);

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

    // 2. Laptop Power & Battery Management Card
    ui.group(|ui| {
        ui.set_min_size(Vec2::new(ui.available_width(), 100.0));
        ui.label(RichText::new("Laptop Power Supply & Battery Telemetry").strong().color(ACCENT_GREEN));
        ui.add_space(4.0);

        egui::Grid::new("power_grid")
            .num_columns(2)
            .spacing([24.0, 8.0])
            .show(ui, |ui| {
                ui.label(RichText::new("AC Power Adapter:").strong());
                if power.ac_online {
                    ui.colored_label(ACCENT_GREEN, "● Plugged In (AC Online)");
                } else {
                    ui.colored_label(ACCENT_ORANGE, "● Unplugged (Running on Battery Power)");
                }
                ui.end_row();

                ui.label(RichText::new("Battery Charge & Status:").strong());
                if let Some(pct) = power.battery_percent {
                    ui.label(format!("{}% ({})", pct, power.battery_status));
                } else {
                    ui.label(&power.battery_status);
                }
                ui.end_row();

                ui.label(RichText::new("Auto-Dim Conservation:").strong());
                ui.label("Adjust master brightness slider to 3/10 or below when on battery to maximize laptop runtime.");
                ui.end_row();
            });
    });

    ui.add_space(10.0);

    // 3. Systemd & Udev Setup Card
    ui.group(|ui| {
        ui.label(RichText::new("Udev Rule & Suspend/Resume Service Guide").strong());
        ui.add_space(4.0);
        ui.label(RichText::new("If non-root permissions or sleep persistence need configuration, run:").size(11.5).color(TEXT_MUTED));
        ui.add_space(4.0);
        ui.monospace("sudo cp packaging/99-msi-mysticlight.rules /etc/udev/rules.d/");
        ui.monospace("sudo udevadm control --reload-rules && sudo udevadm trigger");
        ui.monospace("systemctl --user enable --now msi-keyboard-resume.service");
    });
}
