#[derive(Debug)]
pub struct UdpConfig {
    pub bind_addr: String,
    pub target_addr: String,
}

#[derive(Debug)]
pub struct UdpStreamer {
    bind_addr: String,
    target_addr: String,
}

pub fn init(config: UdpConfig) -> Result<UdpStreamer, String> {
    if config.bind_addr.trim().is_empty() || config.target_addr.trim().is_empty() {
        return Err("UDP addresses cannot be empty".to_string());
    }

    println!(
        "Configuring UDP streamer to bind {} and target {}",
        config.bind_addr, config.target_addr
    );

    Ok(UdpStreamer {
        bind_addr: config.bind_addr,
        target_addr: config.target_addr,
    })
}
