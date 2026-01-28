use std::env;

mod capture;
mod control;
mod encode;
mod net;
mod stream;

#[derive(Debug)]
struct CliConfig {
    display: String,
    bind_addr: String,
    target_addr: String,
    control_bind: String,
    pairing_token: String,
    prefer_vaapi: bool,
}

impl CliConfig {
    fn from_env() -> Result<Self, String> {
        let mut display = String::from(":0");
        let mut bind_addr = String::from("0.0.0.0:5000");
        let mut target_addr = String::from("127.0.0.1:5000");
        let mut control_bind = String::from("0.0.0.0:7000");
        let mut pairing_token = String::from("parallax");
        let mut prefer_vaapi = true;
        let mut args = env::args().skip(1);

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--display" => {
                    display = args.next().ok_or("--display requires a value")?;
                }
                "--bind" => {
                    bind_addr = args.next().ok_or("--bind requires a value")?;
                }
                "--target" => {
                    target_addr = args.next().ok_or("--target requires a value")?;
                }
                "--control-bind" => {
                    control_bind = args.next().ok_or("--control-bind requires a value")?;
                }
                "--pairing-token" => {
                    pairing_token = args.next().ok_or("--pairing-token requires a value")?;
                }
                "--prefer-vaapi" => {
                    prefer_vaapi = true;
                }
                "--software" => {
                    prefer_vaapi = false;
                }
                "-h" | "--help" => {
                    return Err(Self::help());
                }
                other => {
                    return Err(format!("Unknown argument: {other}\n\n{}", Self::help()));
                }
            }
        }

        Ok(Self {
            display,
            bind_addr,
            target_addr,
            control_bind,
            pairing_token,
            prefer_vaapi,
        })
    }

    fn help() -> String {
        [
            "Usage: host [options]",
            "",
            "Options:",
            "  --display <DISPLAY>   X11 display to capture (default :0)",
            "  --bind <ADDR>         UDP bind address (default 0.0.0.0:5000)",
            "  --target <ADDR>       UDP target address (default 127.0.0.1:5000)",
            "  --control-bind <ADDR> TCP control bind address (default 0.0.0.0:7000)",
            "  --pairing-token <KEY> Pairing token for control sessions (default parallax)",
            "  --prefer-vaapi        Prefer VAAPI H.264 encoder (default)",
            "  --software            Force software H.264 encoder",
            "  -h, --help            Print this help text",
        ]
        .join("\n")
    }
}

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
