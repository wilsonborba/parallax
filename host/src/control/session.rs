use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use crate::control::crypto;
use crate::control::protocol::{Frame, MessageType};
use crate::display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandshakeState {
    AwaitHello,
    AwaitPairing,
    AwaitAuth,
    Paired,
}

pub trait StreamCoordinator: Send + Sync {
    fn start_stream(&self, stream_id: u32) -> Result<(), String>;
    fn stop_stream(&self, stream_id: u32) -> Result<(), String>;
    fn set_target(&self, target: String) -> Result<(), String>;
    fn list_streams(&self) -> Result<String, String>;
    fn set_stream_config(
        &self,
        stream_id: u32,
        display: Option<String>,
        bind_addr: Option<String>,
        target_addr: Option<String>,
        prefer_vaapi: Option<bool>,
    ) -> Result<(), String>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaemonState {
    Idle,
    Waiting,
    Connected,
    Streaming,
}

#[derive(Debug, Clone)]
pub struct DaemonStatus {
    pub state: DaemonState,
    pub pin: Option<String>,
    pub qr_uri: Option<String>,
}

pub struct Session {
    state: HandshakeState,
    pairing_token: String,
    stream: Arc<dyn StreamCoordinator>,
    daemon_status: Arc<Mutex<DaemonStatus>>,
    client_addr: SocketAddr,
    default_target_port: Option<u16>,
    pending_nonce: Option<[u8; crypto::NONCE_LEN]>,
    used_nonces: HashSet<[u8; crypto::NONCE_LEN]>,
    master_key: Vec<u8>,
    session_key: Option<Vec<u8>>,
}

impl Session {
    pub fn new(
        pairing_token: String,
        stream: Arc<dyn StreamCoordinator>,
        daemon_status: Arc<Mutex<DaemonStatus>>,
        client_addr: SocketAddr,
        default_target_port: Option<u16>,
    ) -> Self {
        Self {
            state: HandshakeState::AwaitHello,
            master_key: crypto::derive_master_key(&pairing_token),
            pairing_token,
            stream,
            daemon_status,
            client_addr,
            default_target_port,
            pending_nonce: None,
            used_nonces: HashSet::new(),
            session_key: None,
        }
    }

    pub fn handle_frame(&mut self, frame: Frame) -> Vec<Frame> {
        match frame.message_type {
            MessageType::Hello => self.handle_hello(),
            MessageType::PairRequest => self.handle_pair_request(frame.payload),
            MessageType::AuthResponse => self.handle_auth_response(frame.payload),
            MessageType::StartStream => self.handle_start_stream(frame.payload),
            MessageType::StopStream => self.handle_stop_stream(frame.payload),
            MessageType::ListStreams => self.handle_list_streams(),
            MessageType::SetStreamConfig => self.handle_set_stream_config(frame.payload),
            MessageType::ListDisplays => self.handle_list_displays(),
            MessageType::AddVirtualDisplay => self.handle_add_virtual_display(frame.payload),
            MessageType::RemoveVirtualDisplay => self.handle_remove_virtual_display(frame.payload),
            MessageType::Ping => vec![Frame::new(MessageType::Pong, Vec::new())],
            _ => vec![Frame::new(
                MessageType::Error,
                b"Unexpected message".to_vec(),
            )],
        }
    }

    fn handle_hello(&mut self) -> Vec<Frame> {
        if self.state == HandshakeState::AwaitHello {
            self.state = HandshakeState::AwaitPairing;
            self.update_daemon_state(DaemonState::Waiting);
        }

        vec![Frame::new(
            MessageType::HelloAck,
            b"prlx-host".to_vec(),
        )]
    }

    fn handle_pair_request(&mut self, payload: Vec<u8>) -> Vec<Frame> {
        if self.state == HandshakeState::AwaitHello {
            return vec![Frame::new(
                MessageType::Error,
                b"hello required".to_vec(),
            )];
        }

        let (token, target_port) = match parse_pair_request(payload) {
            Ok(token) => token,
            Err(_) => {
                eprintln!(
                    "[control] PairRequest parse failed from {} (state={:?})",
                    self.client_addr, self.state
                );
                return vec![Frame::new(
                    MessageType::PairReject,
                    b"invalid token encoding".to_vec(),
                )];
            }
        };

        if token == self.pairing_token {
            if self.state == HandshakeState::AwaitAuth {
                eprintln!(
                    "[control] PairRequest received while auth pending from {}",
                    self.client_addr
                );
                return vec![Frame::new(
                    MessageType::Error,
                    b"auth in progress".to_vec(),
                )];
            }
            if let Some(target_port) = target_port.or(self.default_target_port) {
                let target = format!("{}:{}", self.client_addr.ip(), target_port);
                if let Err(err) = self.stream.set_target(target) {
                    eprintln!(
                        "[control] Failed to set stream target for {}: {}",
                        self.client_addr, err
                    );
                    return vec![Frame::new(MessageType::Error, err.into_bytes())];
                }
            }
            eprintln!(
                "[control] PairRequest accepted for {} (token match, target_port={:?})",
                self.client_addr, target_port
            );
            let nonce = self.next_nonce();
            self.pending_nonce = Some(nonce);
            self.state = HandshakeState::AwaitAuth;
            vec![Frame::new(MessageType::AuthChallenge, nonce.to_vec())]
        } else {
            eprintln!(
                "[control] PairRequest rejected for {} (token mismatch)",
                self.client_addr
            );
            vec![Frame::new(
                MessageType::PairReject,
                b"invalid token".to_vec(),
            )]
        }
    }

    fn handle_auth_response(&mut self, payload: Vec<u8>) -> Vec<Frame> {
        if self.state != HandshakeState::AwaitAuth {
            eprintln!(
                "[control] AuthResponse unexpected from {} (state={:?})",
                self.client_addr, self.state
            );
            return vec![Frame::new(
                MessageType::Error,
                b"auth not expected".to_vec(),
            )];
        }

        if payload.len() != crypto::HMAC_LEN {
            eprintln!(
                "[control] AuthResponse invalid length from {} (len={})",
                self.client_addr,
                payload.len()
            );
            return vec![Frame::new(
                MessageType::Error,
                b"invalid auth payload".to_vec(),
            )];
        }

        let nonce = match self.pending_nonce.take() {
            Some(nonce) => nonce,
            None => {
                eprintln!(
                    "[control] AuthResponse missing nonce for {}",
                    self.client_addr
                );
                return vec![Frame::new(
                    MessageType::Error,
                    b"auth challenge missing".to_vec(),
                )];
            }
        };

        if self.used_nonces.contains(&nonce) {
            eprintln!(
                "[control] AuthResponse nonce reused for {}",
                self.client_addr
            );
            return vec![Frame::new(
                MessageType::Error,
                b"nonce reused".to_vec(),
            )];
        }

        let session_key = crypto::derive_session_key(&self.master_key, &nonce);
        if !crypto::verify_hmac_sha256(&session_key, &nonce, &payload) {
            eprintln!(
                "[control] AuthResponse HMAC failed for {}",
                self.client_addr
            );
            return vec![Frame::new(
                MessageType::PairReject,
                b"auth failed".to_vec(),
            )];
        }

        self.used_nonces.insert(nonce);
        self.session_key = Some(session_key);
        self.state = HandshakeState::Paired;
        self.update_daemon_state(DaemonState::Connected);
        vec![Frame::new(MessageType::PairAccept, Vec::new())]
    }

    fn handle_start_stream(&mut self, payload: Vec<u8>) -> Vec<Frame> {
        if self.state != HandshakeState::Paired {
            return vec![Frame::new(
                MessageType::Error,
                b"pairing required".to_vec(),
            )];
        }

        let stream_id = match parse_optional_stream_id_payload(payload) {
            Ok(id) => id,
            Err(err) => return vec![Frame::new(MessageType::Error, err.into_bytes())],
        };

        match self.stream.start_stream(stream_id) {
            Ok(()) => {
                self.update_daemon_state(DaemonState::Streaming);
                vec![Frame::new(MessageType::StreamStarted, Vec::new())]
            }
            Err(err) => vec![Frame::new(MessageType::Error, err.into_bytes())],
        }
    }

    fn handle_stop_stream(&mut self, payload: Vec<u8>) -> Vec<Frame> {
        if self.state != HandshakeState::Paired {
            return vec![Frame::new(
                MessageType::Error,
                b"pairing required".to_vec(),
            )];
        }

        let stream_id = match parse_optional_stream_id_payload(payload) {
            Ok(id) => id,
            Err(err) => return vec![Frame::new(MessageType::Error, err.into_bytes())],
        };

        match self.stream.stop_stream(stream_id) {
            Ok(()) => {
                self.update_daemon_state(DaemonState::Connected);
                vec![Frame::new(MessageType::StreamStopped, Vec::new())]
            }
            Err(err) => vec![Frame::new(MessageType::Error, err.into_bytes())],
        }
    }

    fn handle_list_streams(&mut self) -> Vec<Frame> {
        if self.state != HandshakeState::Paired {
            return vec![Frame::new(
                MessageType::Error,
                b"pairing required".to_vec(),
            )];
        }

        match self.stream.list_streams() {
            Ok(payload) => vec![Frame::new(MessageType::Streams, payload.into_bytes())],
            Err(err) => vec![Frame::new(MessageType::Error, err.into_bytes())],
        }
    }

    fn handle_set_stream_config(&mut self, payload: Vec<u8>) -> Vec<Frame> {
        if self.state != HandshakeState::Paired {
            return vec![Frame::new(
                MessageType::Error,
                b"pairing required".to_vec(),
            )];
        }

        let parsed = match parse_set_stream_config_payload(payload) {
            Ok(v) => v,
            Err(err) => return vec![Frame::new(MessageType::Error, err.into_bytes())],
        };

        match self.stream.set_stream_config(
            parsed.stream_id,
            parsed.display,
            parsed.bind_addr,
            parsed.target_addr,
            parsed.prefer_vaapi,
        ) {
            Ok(()) => vec![Frame::new(
                MessageType::StreamConfigAck,
                b"stream config updated".to_vec(),
            )],
            Err(err) => vec![Frame::new(MessageType::Error, err.into_bytes())],
        }
    }

    fn handle_list_displays(&mut self) -> Vec<Frame> {
        if self.state != HandshakeState::Paired {
            return vec![Frame::new(
                MessageType::Error,
                b"pairing required".to_vec(),
            )];
        }

        let physical = match display::list_displays() {
            Ok(displays) => displays,
            Err(err) => {
                return vec![Frame::new(MessageType::Error, err.into_bytes())];
            }
        };

        let virtuals = match display::list_virtual_displays() {
            Ok(displays) => displays,
            Err(err) => {
                return vec![Frame::new(MessageType::Error, err.into_bytes())];
            }
        };

        let mut payload = String::new();
        payload.push_str("protocol=2\n");
        payload.push_str("physical:\n");
        for d in physical {
            let width = d.width.unwrap_or_default();
            let height = d.height.unwrap_or_default();
            let x = d.pos_x.unwrap_or_default();
            let y = d.pos_y.unwrap_or_default();
            payload.push_str(&format!(
                "{},{},{},{},{},{},{},{}\n",
                d.id, d.name, d.primary, d.connected, width, height, x, y
            ));
        }
        payload.push_str("virtual:\n");
        for d in virtuals {
            payload.push_str(&format!(
                "{},{},{},{},{},{}\n",
                d.id, d.enabled, d.width, d.height, d.x, d.y
            ));
        }

        vec![Frame::new(MessageType::Displays, payload.into_bytes())]
    }

    fn handle_add_virtual_display(&mut self, payload: Vec<u8>) -> Vec<Frame> {
        if self.state != HandshakeState::Paired {
            return vec![Frame::new(
                MessageType::Error,
                b"pairing required".to_vec(),
            )];
        }

        let raw = match String::from_utf8(payload) {
            Ok(value) => value,
            Err(_) => {
                return vec![Frame::new(
                    MessageType::Error,
                    b"invalid add display payload".to_vec(),
                )];
            }
        };

        let parts: Vec<&str> = raw.split(',').collect();
        if parts.len() != 5 {
            return vec![Frame::new(
                MessageType::Error,
                b"add payload format: id,width,height,x,y".to_vec(),
            )];
        }

        let width = match parts[1].parse::<u32>() {
            Ok(v) => v,
            Err(_) => return vec![Frame::new(MessageType::Error, b"invalid width".to_vec())],
        };
        let height = match parts[2].parse::<u32>() {
            Ok(v) => v,
            Err(_) => return vec![Frame::new(MessageType::Error, b"invalid height".to_vec())],
        };
        let x = match parts[3].parse::<i32>() {
            Ok(v) => v,
            Err(_) => return vec![Frame::new(MessageType::Error, b"invalid x".to_vec())],
        };
        let y = match parts[4].parse::<i32>() {
            Ok(v) => v,
            Err(_) => return vec![Frame::new(MessageType::Error, b"invalid y".to_vec())],
        };

        let display = display::VirtualDisplay {
            id: parts[0].to_string(),
            width,
            height,
            x,
            y,
            enabled: true,
        };

        match display::enable_virtual_display(display) {
            Ok(()) => vec![Frame::new(
                MessageType::DisplayOpAck,
                b"virtual display added".to_vec(),
            )],
            Err(err) => vec![Frame::new(MessageType::Error, err.into_bytes())],
        }
    }

    fn handle_remove_virtual_display(&mut self, payload: Vec<u8>) -> Vec<Frame> {
        if self.state != HandshakeState::Paired {
            return vec![Frame::new(
                MessageType::Error,
                b"pairing required".to_vec(),
            )];
        }

        let id = match String::from_utf8(payload) {
            Ok(value) => value.trim().to_string(),
            Err(_) => {
                return vec![Frame::new(
                    MessageType::Error,
                    b"invalid remove display payload".to_vec(),
                )];
            }
        };

        if id.is_empty() {
            return vec![Frame::new(MessageType::Error, b"display id required".to_vec())];
        }

        match self.stream.list_streams() {
            Ok(payload) => {
                if is_display_used_by_running_stream(&payload, &id) {
                    return vec![Frame::new(
                        MessageType::Error,
                        b"display is in use by an active stream".to_vec(),
                    )];
                }
            }
            Err(err) => return vec![Frame::new(MessageType::Error, err.into_bytes())],
        }

        match display::disable_virtual_display(&id) {
            Ok(()) => vec![Frame::new(
                MessageType::DisplayOpAck,
                b"virtual display removed".to_vec(),
            )],
            Err(err) => vec![Frame::new(MessageType::Error, err.into_bytes())],
        }
    }

    fn next_nonce(&mut self) -> [u8; crypto::NONCE_LEN] {
        loop {
            let nonce = crypto::generate_nonce();
            if !self.used_nonces.contains(&nonce) {
                return nonce;
            }
        }
    }

    fn update_daemon_state(&self, state: DaemonState) {
        if let Ok(mut status) = self.daemon_status.lock() {
            status.state = state;
        }
    }
}

fn parse_pair_request(payload: Vec<u8>) -> Result<(String, Option<u16>), ()> {
    let token = String::from_utf8(payload).map_err(|_| ())?;
    if let Some((token, port)) = token.split_once('|') {
        let port = port.parse::<u16>().map_err(|_| ())?;
        return Ok((token.to_string(), Some(port)));
    }
    Ok((token, None))
}

struct SetStreamConfigPayload {
    stream_id: u32,
    display: Option<String>,
    bind_addr: Option<String>,
    target_addr: Option<String>,
    prefer_vaapi: Option<bool>,
}

fn parse_set_stream_config_payload(payload: Vec<u8>) -> Result<SetStreamConfigPayload, String> {
    let raw = String::from_utf8(payload).map_err(|_| "invalid set stream payload".to_string())?;

    let mut stream_id = None;
    let mut display = None;
    let mut bind_addr = None;
    let mut target_addr = None;
    let mut prefer_vaapi = None;

    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (k, v) = line
            .split_once('=')
            .ok_or_else(|| "set stream payload expects key=value lines".to_string())?;

        match k {
            "stream_id" => {
                stream_id = Some(
                    v.parse::<u32>()
                        .map_err(|_| "invalid stream_id".to_string())?,
                )
            }
            "display" => display = Some(v.to_string()),
            "bind_addr" => bind_addr = Some(v.to_string()),
            "target_addr" => target_addr = Some(v.to_string()),
            "prefer_vaapi" => match v {
                "true" | "1" | "yes" => prefer_vaapi = Some(true),
                "false" | "0" | "no" => prefer_vaapi = Some(false),
                _ => return Err("invalid prefer_vaapi".to_string()),
            },
            _ => return Err(format!("unknown key in stream config payload: {k}")),
        }
    }

    let stream_id = stream_id.ok_or_else(|| "stream_id is required".to_string())?;

    Ok(SetStreamConfigPayload {
        stream_id,
        display,
        bind_addr,
        target_addr,
        prefer_vaapi,
    })
}

fn parse_optional_stream_id_payload(payload: Vec<u8>) -> Result<u32, String> {
    if payload.is_empty() {
        return Ok(1);
    }

    let raw = String::from_utf8(payload).map_err(|_| "invalid stream payload".to_string())?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(1);
    }

    if let Some((k, v)) = trimmed.split_once('=') {
        if k == "stream_id" {
            return v
                .parse::<u32>()
                .map_err(|_| "invalid stream_id".to_string());
        }
    }

    trimmed
        .parse::<u32>()
        .map_err(|_| "stream payload expects stream_id or stream_id=<id>".to_string())
}

fn is_display_used_by_running_stream(payload: &str, display_id: &str) -> bool {
    for line in payload.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("protocol=") || line == "streams:" {
            continue;
        }
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 6 {
            continue;
        }
        let line_display_id = parts[1];
        let running = matches!(parts[5], "true" | "1" | "yes");
        if running && line_display_id == display_id {
            return true;
        }
    }
    false
}
