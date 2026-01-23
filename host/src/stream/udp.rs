use std::net::{SocketAddr, UdpSocket};

use crate::net::UdpPacket;

#[derive(Debug, Clone)]
pub struct UdpConfig {
    pub bind_addr: String,
    pub target_addr: String,
}

#[derive(Debug)]
pub struct UdpStreamer {
    socket: UdpSocket,
    target: SocketAddr,
}

pub fn init(config: UdpConfig) -> Result<UdpStreamer, String> {
    let socket = UdpSocket::bind(&config.bind_addr)
        .map_err(|e| format!("Failed to bind UDP socket on {}: {e}", config.bind_addr))?;

    let target: SocketAddr = config
        .target_addr
        .parse()
        .map_err(|e| format!("Invalid target address {}: {e}", config.target_addr))?;

    println!(
        "Configuring UDP streamer to bind {} and target {}",
        config.bind_addr, target
    );

    Ok(UdpStreamer { socket, target })
}

impl UdpStreamer {
    pub fn send_packet(&self, packet: &UdpPacket) -> Result<(), String> {
        let bytes = packet.to_bytes();
        self.socket
            .send_to(&bytes, self.target)
            .map(|_| ())
            .map_err(|e| format!("Failed to send UDP packet to {}: {e}", self.target))
    }
}
