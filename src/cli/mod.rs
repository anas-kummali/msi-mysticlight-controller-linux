//! Command line interface execution engine.

pub mod args;
pub mod view;

use std::env;
use std::process;

pub use args::{parse_args, CliArgs};
pub use view::{print_help, print_status};

use crate::constants::{MODE_BREATHING, MODE_CYCLE, MODE_WAVE};
use crate::controller::{parse_color, MSIKeyboardController};
use crate::VERSION;

/// Main CLI execution entrypoint.
pub fn run() {
    let exit_code = execute();
    if exit_code != 0 {
        process::exit(exit_code);
    }
}

/// Execute CLI action and return exit status code.
pub fn execute() -> i32 {
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
            return 1;
        }
    };

    if args.show_help {
        print_help(&prog_name);
        return 0;
    }

    if args.show_version {
        println!("{} {}", prog_name, VERSION);
        return 0;
    }

    if args.gui {
        #[cfg(feature = "gui")]
        {
            if let Err(e) = crate::gui::run_gui() {
                eprintln!("Error launching GUI: {}", e);
                return 1;
            }
            return 0;
        }
        #[cfg(not(feature = "gui"))]
        {
            eprintln!("GUI feature was disabled at compile time. Recompile with --features gui.");
            return 1;
        }
    }

    let mut ctrl = MSIKeyboardController::new();

    if args.status {
        let info = ctrl.get_device_info();
        print_status(&info);
        return 0;
    }

    // Validate color argument before opening hardware device
    let raw_rgb = match parse_color(&args.color, 10) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {}", e);
            return 1;
        }
    };

    if let Err(e) = ctrl.open() {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            eprintln!("Error: {}", e);
            return 2;
        } else {
            eprintln!("Error opening keyboard device: {}", e);
            return 1;
        }
    }

    if args.off || args.brightness == 0 {
        if let Err(e) = ctrl.turn_off() {
            eprintln!("Error powering off keyboard: {}", e);
            return 1;
        }
        println!("Keyboard backlight turned OFF across all 6 rows.");
        return 0;
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
            return 1;
        }
        println!("Applied Rainbow Wave effect across all 6 rows (all 24 zones).");
        return 0;
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
            return 1;
        }
        println!("Applied Color Cycle effect across all 6 rows (all 24 zones).");
        return 0;
    }

    if args.breathe {
        let breathe_seq = vec![(0, 0, 0), raw_rgb, (0, 0, 0)];
        if let Err(e) = ctrl.set_effect(MODE_BREATHING, &breathe_seq, 120) {
            eprintln!("Error activating breathing effect: {}", e);
            return 1;
        }
        println!("Applied Breathing effect with {} across all 6 rows.", args.color);
        return 0;
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
        return 1;
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

    0
}

