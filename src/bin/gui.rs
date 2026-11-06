//! Dedicated GUI binary entrypoint for msi-mysticlight-gui.

fn main() {
    #[cfg(feature = "gui")]
    {
        if let Err(e) = msi_mysticlight::gui::run_gui() {
            eprintln!("Error running GUI: {}", e);
            std::process::exit(1);
        }
    }
    #[cfg(not(feature = "gui"))]
    {
        eprintln!("GUI feature was not enabled at compile time. Recompile with --features gui.");
        std::process::exit(1);
    }
}
