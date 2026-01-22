#[derive(Debug)]
pub struct X11CaptureConfig {
    pub display: String,
}

#[derive(Debug)]
pub struct X11Capture {
    display: String,
}

pub fn init(config: X11CaptureConfig) -> Result<X11Capture, String> {
    if config.display.trim().is_empty() {
        return Err("X11 display cannot be empty".to_string());
    }

    println!("Configuring X11 capture for display {}", config.display);

    Ok(X11Capture {
        display: config.display,
    })
}
