//! Terminal rendering, help texts, and status displays.

use crate::constants::COLORS;
use crate::device::DeviceInfo;
use crate::VERSION;

/// Print formatted CLI help documentation.
pub fn print_help(prog_name: &str) {
    let color_names: Vec<&str> = COLORS.iter().map(|(n, _)| *n).collect();
    println!(
        r##"MSI Mystic Light Controller for Linux — Keyboard Backlight Driver (Vector 16 HX AI, Crosshair, Pulse, Katana, etc.)

Usage: {0} [OPTIONS]

Options:
  -c, --color <COLOR>        Color name ({1}) or hex (#RRGGBB) [default: white]
  -B, --brightness <0..10>   Brightness level from 0 (off) to 10 (max, default: 10) [default: 10]
  -o, --off                  Turn off keyboard backlight completely across all 6 rows
  -w, --wave                 Run rainbow wave animation across all 6 rows
  -b, --breathe              Run breathing effect using selected color
  -s, --cycle                Run spectrum color cycle animation
  -r, --row <1..6>           Target a specific physical row (1: Function, 2: Number, 3: Tab/QWERTY, 4: Caps/ASDF, 5: Shift/ZXCV, 6: Space/Ctrl)
  -z, --zone <0..23>         Target a specific individual zone (0..23)
      --status               Display detected hardware node, permission state, and zone map
      --bridge, --daemon     Run userspace sysfs bridge daemon (enables OS tray slider & Fn keys)
  -g, --gui                  Launch native graphical control interface
  -v, --version              Show version information
  -h, --help                 Print this help message

Examples:
  {0} -c white -B 10       # Set all 6 rows to bright white
  {0} -c "#FF8000"         # Set all rows to custom orange
  {0} -c red -r 1          # Set only Row 1 (ESC to DEL) to red
  {0} -w                   # Activate rainbow wave effect across all 6 rows
  {0} -b -c cyan           # Activate cyan breathing effect
  {0} -s                   # Activate color cycle effect
  {0} -o                   # Power off backlight completely
  {0} --status             # Show controller & device status"##,
        prog_name,
        color_names.join(", ")
    );
}

/// Print formatted hardware and controller diagnostic status.
pub fn print_status(info: &DeviceInfo) {
    println!("==================================================");
    println!(" MSI Mystic Light Controller for Linux");
    println!(" Version : {}", VERSION);
    println!("==================================================");
    println!(
        "Device Node     : {}",
        info.node.as_deref().unwrap_or("None")
    );
    println!(
        "Hardware Status : {}",
        if info.present { "Online" } else { "Not Detected" }
    );
    println!(
        "Permissions     : {}",
        if info.accessible {
            "Read/Write (OK)"
        } else {
            "Permission Denied (Check udev)"
        }
    );
    println!("Total Zones     : {} Zones", info.total_zones);
    println!("Total Rows      : {} Physical Rows", info.total_rows);
    println!("\nPhysical Row Layout:");
    println!("  - Row 1 (ESC to DEL)       : Zones 0..3");
    println!("  - Row 2 (` to BACKSPACE)   : Zones 4..7");
    println!("  - Row 3 (TAB to \\)         : Zones 8..11");
    println!("  - Row 4 (CAPS to ENTER)    : Zones 12..15");
    println!("  - Row 5 (SHIFT to SHIFT)   : Zones 16..19");
    println!("  - Row 6 (CTRL to CTRL/Nav) : Zones 20..23");
    println!("==================================================");
}

