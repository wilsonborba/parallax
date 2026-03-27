use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use host::cli::{CliAction, CliConfig};
use host::control;
use host::display;

fn main() {
    let action = match CliConfig::from_env() {
        Ok(action) => action,
        Err(message) => {
            println!("{message}");
            return;
        }
    };

    match action {
        CliAction::ListDisplays => match display::list_displays() {
            Ok(displays) => println!("{}", display::format_displays(&displays)),
            Err(err) => eprintln!("Failed to list displays: {err}"),
        },
        CliAction::VirtualBackendStatus => {
            let status = display::virtual_backend_status();
            println!("{}", display::format_virtual_backend_status(&status));
        },
        CliAction::ListVirtualDisplays => match display::list_virtual_displays() {
            Ok(displays) => println!("{}", display::format_virtual_displays(&displays)),
            Err(err) => eprintln!("Failed to list virtual displays: {err}"),
        },
        CliAction::EnableVirtualDisplay(config) => {
            let vd = display::VirtualDisplay {
                id: config.id,
                width: config.width,
                height: config.height,
                x: config.x,
                y: config.y,
                enabled: true,
            };
            match display::enable_virtual_display(vd) {
                Ok(()) => println!("Virtual display enabled."),
                Err(err) => eprintln!("Failed to enable virtual display: {err}"),
            }
        }
        CliAction::DisableVirtualDisplay { id } => match display::disable_virtual_display(&id) {
            Ok(()) => println!("Virtual display disabled."),
            Err(err) => eprintln!("Failed to disable virtual display: {err}"),
        },
        CliAction::Run(config) => {
            println!("Starting prlx-hostd with config: {config:?}");

            match display::apply_persisted_virtual_displays() {
                Ok(applied) if !applied.is_empty() => {
                    println!("Applied persisted virtual displays: {}", applied.join(", "));
                }
                Ok(_) => {}
                Err(err) => {
                    eprintln!("Warning: failed to apply persisted virtual displays: {err}");
                }
            }

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
    }
}
