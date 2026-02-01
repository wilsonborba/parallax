use std::path::{Path, PathBuf};
use std::sync::mpsc;

use host::core::logging as loggins;

mod daemon;
mod palette;
mod qr;
mod ui;
mod widgets;

const DEFAULT_SOCKET_PATH: &str = "~/.local/share/prlx/prlx.sock";

fn main() -> eframe::Result<()> {
    let socket_path = expand_path(DEFAULT_SOCKET_PATH);
    loggins::info(
        "ui",
        format!("Starting Parallax Host UI; socket_path={socket_path:?}"),
    );

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_inner_size([1020.0, 760.0]),
        ..Default::default()
    };

    // Keep ONLY this Ctrl-C handler (no second handler anywhere else).
    let (shutdown_tx, shutdown_rx) = mpsc::channel();
    let handler_tx = shutdown_tx.clone();
    if let Err(err) = ctrlc::set_handler(move || {
        let _ = handler_tx.send(());
    }) {
        loggins::error("ui", format!("Failed to install signal handler: {err}"));
    }

    eframe::run_native(
        "Parallax Host UI",
        native_options,
        Box::new(move |cc| Box::new(ui::HostUiApp::new(cc, socket_path, shutdown_rx))),
    )
}

fn expand_path(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return Path::new(&home).join(rest);
        }
    }
    PathBuf::from(path)
}
