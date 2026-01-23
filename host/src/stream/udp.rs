use std::net::UdpSocket;

use crate::net::UdpPacket;

#[derive(Debug)]
pub struct UdpConfig {
    pub bind_addr: String,
    pub target_addr: String,
}

#[derive(Debug)]
pub struct UdpStreamer {
    socket: UdpSocket,
}

pub fn init(config: UdpConfig) -> Result<UdpStreamer, String> {
    if config.bind_addr.trim().is_empty() || config.target_addr.trim().is_empty() {
        return Err("UDP addresses cannot be empty".to_string());
    }

    println!(
        "Configuring UDP streamer to bind {} and target {}",
        config.bind_addr, config.target_addr
    );

    let socket = UdpSocket::bind(&config.bind_addr)
        .map_err(|error| format!("Failed to bind UDP socket: {error}"))?;
    socket
        .connect(&config.target_addr)
        .map_err(|error| format!("Failed to connect UDP socket: {error}"))?;

    Ok(UdpStreamer {
        socket,
    })
}

impl UdpStreamer {
    pub fn send_packet(&self, packet: &UdpPacket) -> Result<usize, String> {
        self.socket
            .send(&packet.to_bytes())
            .map_err(|error| format!("Failed to send UDP packet: {error}"))
    }
}
