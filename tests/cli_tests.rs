//! Integration tests for msi-mysticlight binary.

use std::process::Command;

fn get_bin_path() -> std::path::PathBuf {
    let mut p = std::env::current_exe().unwrap();
    p.pop(); // drop test binary name
    if p.ends_with("deps") {
        p.pop(); // drop deps
    }
    p.push("msi-mysticlight");
    p
}

#[test]
fn test_cli_help() {
    let bin = get_bin_path();
    let out = Command::new(&bin).arg("--help").output().expect("failed to run bin");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("MSI Mystic Light Controller for Linux"));
    assert!(stdout.contains("-c, --color"));
    assert!(stdout.contains("-B, --brightness"));
    assert!(stdout.contains("-w, --wave"));
    assert!(stdout.contains("-b, --breathe"));
    assert!(stdout.contains("-s, --cycle"));
    assert!(stdout.contains("-r, --row"));
    assert!(stdout.contains("-z, --zone"));
    assert!(stdout.contains("--status"));
}

#[test]
fn test_cli_version() {
    let bin = get_bin_path();
    let out = Command::new(&bin).arg("--version").output().expect("failed to run bin");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(stdout.trim(), "msi-mysticlight 1.0.0");
}

#[test]
fn test_cli_status() {
    let bin = get_bin_path();
    let out = Command::new(&bin).arg("--status").output().expect("failed to run bin");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("MSI Mystic Light Controller for Linux"));
    assert!(stdout.contains("Total Zones     : 24 Zones"));
    assert!(stdout.contains("Total Rows      : 6 Physical Rows"));
}

#[test]
fn test_cli_invalid_color() {
    let bin = get_bin_path();
    let out = Command::new(&bin)
        .args(["-c", "invalid_color_xyz"])
        .output()
        .expect("failed to run bin");
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("Invalid color"));
}

#[test]
fn test_cli_invalid_brightness() {
    let bin = get_bin_path();
    let out = Command::new(&bin)
        .args(["-B", "99"])
        .output()
        .expect("failed to run bin");
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("out of range"));
}

#[test]
fn test_cli_invalid_row() {
    let bin = get_bin_path();
    let out = Command::new(&bin)
        .args(["-r", "7"])
        .output()
        .expect("failed to run bin");
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("out of range"));
}

#[test]
fn test_cli_invalid_zone() {
    let bin = get_bin_path();
    let out = Command::new(&bin)
        .args(["-z", "30"])
        .output()
        .expect("failed to run bin");
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("out of range"));
}
