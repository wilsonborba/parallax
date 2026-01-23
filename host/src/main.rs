use std::env;

mod capture;
mod encode;
mod net;
mod stream;

#[derive(Debug)]
struct CliConfig {
    display: String,
    bind_addr: String,
    target_addr: String,
    prefer_vaapi: bool,
}

impl CliConfig {
    fn from_env() -> Result<Self, String> {
        let mut display = String::from(":0");
        let mut bind_addr = String::from("0.0.0.0:5000");
        let mut target_addr = String::from("127.0.0.1:5000");
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

    println!("Starting host with config: {config:?}");

    let capture = match capture::x11::init(capture::x11::X11CaptureConfig {
        display: config.display.clone(),
    }) {
        Ok(capture) => capture,
        Err(error) => {
            eprintln!("Failed to initialize X11 capture: {error}");
            return;
        }
    };

    let encoder = match encode::h264::init(encode::h264::H264Config {
        prefer_vaapi: config.prefer_vaapi,
    }) {
        Ok(encoder) => encoder,
        Err(error) => {
            eprintln!("Failed to initialize H.264 encoder: {error}");
            return;
        }
    };

    let streamer = match stream::udp::init(stream::udp::UdpConfig {
        bind_addr: config.bind_addr.clone(),
        target_addr: config.target_addr.clone(),
    }) {
        Ok(streamer) => streamer,
        Err(error) => {
            eprintln!("Failed to initialize UDP streamer: {error}");
            return;
        }
    };

    println!(
        "Pipeline ready: capture={:?}, encoder={:?}, stream={:?}",
        capture, encoder, streamer
    );
    println!("Streaming loop not implemented yet.");
}
