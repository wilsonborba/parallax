use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use host::cli::CliConfig;
use host::control;

fn main() {
    let config = match CliConfig::from_env() {
        Ok(config) => config,
        Err(message) => {
            println!("{message}");
            return;
        }
    };

    println!("Starting prlx-hostd with config: {config:?}");

    let running = Arc::new(AtomicBool::new(true));
    let handler_flag = Arc::clone(&running);
    if let Err(err) = ctrlc::set_handler(move || {
        handler_flag.store(false, Ordering::Relaxed);
    }) {
        eprintln!("Failed to install signal handler: {err}");
    }

    let control_config = control::server::ControlConfig {
        control_bind: config.control_bind.clone(),
        pairing_token: config.pairing_token.clone(),
        stream: control::server::StreamConfig {
            display: config.display.clone(),
            bind_addr: config.bind_addr.clone(),
            target_addr: config.target_addr.clone(),
            prefer_vaapi: config.prefer_vaapi,
        },
    };

    if let Err(error) = control::server::run_with_shutdown(control_config, running) {
        eprintln!("Control server error: {error}");
    }
}
