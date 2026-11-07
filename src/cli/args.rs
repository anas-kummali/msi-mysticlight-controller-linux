//! Command-line argument parsing and validation.

use std::env;

/// Parsed command line arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliArgs {
    pub color: String,
    pub brightness: u8,
    pub off: bool,
    pub wave: bool,
    pub breathe: bool,
    pub cycle: bool,
    pub row: Option<u8>,
    pub zone: Option<u8>,
    pub status: bool,
    pub gui: bool,
    pub bridge: bool,
    pub show_version: bool,
    pub show_help: bool,
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
            gui: false,
            bridge: false,
            show_version: false,
            show_help: false,
        }
    }
}

/// Parse arguments from environment or custom iterator.
pub fn parse_args() -> Result<CliArgs, String> {
    parse_from(env::args().collect())
}

/// Parse arguments from a vector of strings.
pub fn parse_from(raw: Vec<String>) -> Result<CliArgs, String> {
    let mut args = CliArgs::default();
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
        } else if arg == "--bridge" || arg == "--daemon" || arg == "bridge" || arg == "daemon" {
            args.bridge = true;
        } else if arg == "--gui" || arg == "-g" {
            args.gui = true;
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


#[cfg(test)]
mod tests {
    use super::*;

    fn to_vec(args: &[&str]) -> Vec<String> {
        let mut v = vec!["msi-mysticlight".to_string()];
        v.extend(args.iter().map(|s| s.to_string()));
        v
    }

    #[test]
    fn test_parse_default() {
        let args = parse_from(to_vec(&[])).unwrap();
        assert_eq!(args.color, "white");
        assert_eq!(args.brightness, 10);
        assert!(!args.off);
        assert!(!args.wave);
        assert!(!args.breathe);
        assert!(!args.cycle);
        assert_eq!(args.row, None);
        assert_eq!(args.zone, None);
    }

    #[test]
    fn test_parse_flags() {
        assert!(parse_from(to_vec(&["-h"])).unwrap().show_help);
        assert!(parse_from(to_vec(&["--help"])).unwrap().show_help);
        assert!(parse_from(to_vec(&["-v"])).unwrap().show_version);
        assert!(parse_from(to_vec(&["--version"])).unwrap().show_version);
        assert!(parse_from(to_vec(&["--status"])).unwrap().status);
        assert!(parse_from(to_vec(&["-o"])).unwrap().off);
        assert!(parse_from(to_vec(&["-w"])).unwrap().wave);
        assert!(parse_from(to_vec(&["-b"])).unwrap().breathe);
        assert!(parse_from(to_vec(&["-s"])).unwrap().cycle);
        assert!(parse_from(to_vec(&["--bridge"])).unwrap().bridge);
        assert!(parse_from(to_vec(&["--daemon"])).unwrap().bridge);
        assert!(parse_from(to_vec(&["bridge"])).unwrap().bridge);
        assert!(parse_from(to_vec(&["daemon"])).unwrap().bridge);
    }

    #[test]
    fn test_parse_color_options() {
        let a1 = parse_from(to_vec(&["-c", "red"])).unwrap();
        assert_eq!(a1.color, "red");

        let a2 = parse_from(to_vec(&["--color=blue"])).unwrap();
        assert_eq!(a2.color, "blue");

        assert!(parse_from(to_vec(&["-c"])).is_err());
    }

    #[test]
    fn test_parse_brightness_options() {
        let a1 = parse_from(to_vec(&["-B", "5"])).unwrap();
        assert_eq!(a1.brightness, 5);

        let a2 = parse_from(to_vec(&["--brightness=8"])).unwrap();
        assert_eq!(a2.brightness, 8);

        assert!(parse_from(to_vec(&["-B"])).is_err());
        assert!(parse_from(to_vec(&["-B", "11"])).is_err());
        assert!(parse_from(to_vec(&["--brightness=foo"])).is_err());
    }

    #[test]
    fn test_parse_row_and_zone() {
        let a1 = parse_from(to_vec(&["-r", "2"])).unwrap();
        assert_eq!(a1.row, Some(2));

        let a2 = parse_from(to_vec(&["--row=6"])).unwrap();
        assert_eq!(a2.row, Some(6));

        assert!(parse_from(to_vec(&["-r", "0"])).is_err());
        assert!(parse_from(to_vec(&["-r", "7"])).is_err());

        let z1 = parse_from(to_vec(&["-z", "15"])).unwrap();
        assert_eq!(z1.zone, Some(15));

        let z2 = parse_from(to_vec(&["--zone=23"])).unwrap();
        assert_eq!(z2.zone, Some(23));

        assert!(parse_from(to_vec(&["-z", "24"])).is_err());
    }

    #[test]
    fn test_parse_unknown_argument() {
        assert!(parse_from(to_vec(&["--nonexistent"])).is_err());
    }
}
