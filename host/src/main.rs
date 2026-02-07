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

    println!("Starting host control daemon with config: {config:?}");

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

    if let Err(error) = control::server::run(control_config) {
        eprintln!("Control server error: {error}");
    }
}
