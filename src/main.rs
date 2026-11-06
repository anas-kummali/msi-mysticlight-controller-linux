//! CLI binary for msi-mysticlight.

use std::env;
use std::process;

use msi_mysticlight::constants::{
    COLORS, MODE_BREATHING, MODE_CYCLE, MODE_WAVE,
};
use msi_mysticlight::controller::{parse_color, DeviceInfo, MSIKeyboardController};
use msi_mysticlight::VERSION;

struct CliArgs {
    color: String,
    brightness: u8,
    off: bool,
    wave: bool,
    breathe: bool,
    cycle: bool,
    row: Option<u8>,
    zone: Option<u8>,
    status: bool,
    show_version: bool,
    show_help: bool,
}

impl Default for CliArgs {
    fn default() -> Self {
        Self {
            color: "white".to_string(),
            brightness: 10,
            off: false,
            wave: false,
            breathe: false,
            cycle: false,
            row: None,
            zone: None,
            status: false,
            show_version: false,
            show_help: false,
        }
    }
}

fn print_help(prog_name: &str) {
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

fn print_status(info: &DeviceInfo) {
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

fn parse_args() -> Result<CliArgs, String> {
    let mut args = CliArgs::default();
    let raw: Vec<String> = env::args().collect();
    let mut i = 1;

    while i < raw.len() {
        let arg = &raw[i];
        if arg == "-h" || arg == "--help" {
            args.show_help = true;
            return Ok(args);
        } else if arg == "-v" || arg == "--version" {
            args.show_version = true;
            return Ok(args);
        } else if arg == "--status" {
            args.status = true;
        } else if arg == "-o" || arg == "--off" {
            args.off = true;
        } else if arg == "-w" || arg == "--wave" {
            args.wave = true;
        } else if arg == "-b" || arg == "--breathe" {
            args.breathe = true;
        } else if arg == "-s" || arg == "--cycle" {
            args.cycle = true;
        } else if arg == "-c" || arg == "--color" {
            i += 1;
            if i >= raw.len() {
                return Err("Option '-c/--color' requires an argument.".into());
            }
            args.color = raw[i].clone();
        } else if let Some(val) = arg.strip_prefix("--color=") {
            args.color = val.to_string();
        } else if arg == "-B" || arg == "--brightness" {
            i += 1;
            if i >= raw.len() {
                return Err("Option '-B/--brightness' requires an argument.".into());
            }
            let b: u8 = raw[i]
                .parse()
                .map_err(|_| format!("Invalid brightness '{}'. Must be between 0 and 10.", raw[i]))?;
            if b > 10 {
                return Err(format!("Brightness {} out of range [0..10].", b));
            }
            args.brightness = b;
        } else if let Some(val) = arg.strip_prefix("--brightness=") {
            let b: u8 = val
                .parse()
                .map_err(|_| format!("Invalid brightness '{}'. Must be between 0 and 10.", val))?;
            if b > 10 {
                return Err(format!("Brightness {} out of range [0..10].", b));
            }
            args.brightness = b;
        } else if arg == "-r" || arg == "--row" {
            i += 1;
            if i >= raw.len() {
                return Err("Option '-r/--row' requires an argument.".into());
            }
            let r: u8 = raw[i]
                .parse()
                .map_err(|_| format!("Invalid row '{}'. Must be between 1 and 6.", raw[i]))?;
            if !(1..=6).contains(&r) {
                return Err(format!("Row {} out of range [1..6].", r));
            }
            args.row = Some(r);
        } else if let Some(val) = arg.strip_prefix("--row=") {
            let r: u8 = val
                .parse()
                .map_err(|_| format!("Invalid row '{}'. Must be between 1 and 6.", val))?;
            if !(1..=6).contains(&r) {
                return Err(format!("Row {} out of range [1..6].", r));
            }
            args.row = Some(r);
        } else if arg == "-z" || arg == "--zone" {
            i += 1;
            if i >= raw.len() {
                return Err("Option '-z/--zone' requires an argument.".into());
            }
            let z: u8 = raw[i]
                .parse()
                .map_err(|_| format!("Invalid zone '{}'. Must be between 0 and 23.", raw[i]))?;
            if z >= 24 {
                return Err(format!("Zone {} out of range [0..23].", z));
            }
            args.zone = Some(z);
        } else if let Some(val) = arg.strip_prefix("--zone=") {
            let z: u8 = val
                .parse()
                .map_err(|_| format!("Invalid zone '{}'. Must be between 0 and 23.", val))?;
            if z >= 24 {
                return Err(format!("Zone {} out of range [0..23].", z));
            }
            args.zone = Some(z);
        } else {
            return Err(format!("Unknown argument: '{}'", arg));
        }
        i += 1;
    }

    Ok(args)
}

fn main() {
    let prog_name = env::args()
        .next()
        .map(|p| {
            std::path::Path::new(&p)
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or(p)
        })
        .unwrap_or_else(|| "msi-mysticlight".to_string());

    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error: {}", e);
            eprintln!("Run '{} --help' for usage instructions.", prog_name);
            process::exit(1);
        }
    };

    if args.show_help {
        print_help(&prog_name);
        return;
    }

    if args.show_version {
        println!("{} {}", prog_name, VERSION);
        return;
    }

    let mut ctrl = MSIKeyboardController::new();

    if args.status {
        let info = ctrl.get_device_info();
        print_status(&info);
        return;
    }

    // Validate color argument before opening hardware device
    let raw_rgb = match parse_color(&args.color, 10) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    if let Err(e) = ctrl.open() {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            eprintln!("Error: {}", e);
            process::exit(2);
        } else {
            eprintln!("Error opening keyboard device: {}", e);
            process::exit(1);
        }
    }

    if args.off || args.brightness == 0 {
        if let Err(e) = ctrl.turn_off() {
            eprintln!("Error powering off keyboard: {}", e);
            process::exit(1);
        }
        println!("Keyboard backlight turned OFF across all 6 rows.");
        return;
    }

    if args.wave {
        let rainbow = vec![
            (255, 0, 0),
            (255, 128, 0),
            (255, 255, 0),
            (0, 255, 0),
            (0, 255, 255),
            (0, 0, 255),
            (255, 0, 255),
        ];
        if let Err(e) = ctrl.set_effect(MODE_WAVE, &rainbow, 200) {
            eprintln!("Error activating wave effect: {}", e);
            process::exit(1);
        }
        println!("Applied Rainbow Wave effect across all 6 rows (all 24 zones).");
        return;
    }

    if args.cycle {
        let cycle_colors = vec![
            (255, 0, 0),
            (0, 255, 0),
            (0, 120, 255),
            (255, 255, 0),
        ];
        if let Err(e) = ctrl.set_effect(MODE_CYCLE, &cycle_colors, 150) {
            eprintln!("Error activating color cycle effect: {}", e);
            process::exit(1);
        }
        println!("Applied Color Cycle effect across all 6 rows (all 24 zones).");
        return;
    }

    if args.breathe {
        let breathe_seq = vec![(0, 0, 0), raw_rgb, (0, 0, 0)];
        if let Err(e) = ctrl.set_effect(MODE_BREATHING, &breathe_seq, 120) {
            eprintln!("Error activating breathing effect: {}", e);
            process::exit(1);
        }
        println!("Applied Breathing effect with {} across all 6 rows.", args.color);
        return;
    }

    if let Err(e) = ctrl.set_color(
        raw_rgb.0,
        raw_rgb.1,
        raw_rgb.2,
        args.brightness,
        args.row,
        args.zone,
    ) {
        eprintln!("Error setting keyboard color: {}", e);
        process::exit(1);
    }

    let target = if let Some(r) = args.row {
        format!("Row {}", r)
    } else if let Some(z) = args.zone {
        format!("Zone {}", z)
    } else {
        "All 6 Rows (24 Zones)".to_string()
    };

    println!(
        "Set static color to {} (RGB: ({}, {}, {}), Brightness: {}/10) on {}.",
        args.color, raw_rgb.0, raw_rgb.1, raw_rgb.2, args.brightness, target
    );
}
