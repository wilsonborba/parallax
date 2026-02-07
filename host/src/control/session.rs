use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use crate::control::crypto;
use crate::control::protocol::{Frame, MessageType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandshakeState {
    AwaitHello,
    AwaitPairing,
    AwaitAuth,
    Paired,
}

pub trait StreamCoordinator: Send + Sync {
    fn start_stream(&self) -> Result<(), String>;
    fn stop_stream(&self) -> Result<(), String>;
    fn set_target(&self, target: String) -> Result<(), String>;
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
            MessageType::StartStream => self.handle_start_stream(),
            MessageType::StopStream => self.handle_stop_stream(),
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

    fn handle_start_stream(&mut self) -> Vec<Frame> {
        if self.state != HandshakeState::Paired {
            return vec![Frame::new(
                MessageType::Error,
                b"pairing required".to_vec(),
            )];
        }

        match self.stream.start_stream() {
            Ok(()) => {
                self.update_daemon_state(DaemonState::Streaming);
                vec![Frame::new(MessageType::StreamStarted, Vec::new())]
            }
            Err(err) => vec![Frame::new(MessageType::Error, err.into_bytes())],
        }
    }

    fn handle_stop_stream(&mut self) -> Vec<Frame> {
        if self.state != HandshakeState::Paired {
            return vec![Frame::new(
                MessageType::Error,
                b"pairing required".to_vec(),
            )];
        }

        match self.stream.stop_stream() {
            Ok(()) => {
                self.update_daemon_state(DaemonState::Connected);
                vec![Frame::new(MessageType::StreamStopped, Vec::new())]
            }
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
