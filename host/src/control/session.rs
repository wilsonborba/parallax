use std::collections::HashSet;
use std::sync::Arc;

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
}

pub struct Session {
    state: HandshakeState,
    pairing_token: String,
    stream: Arc<dyn StreamCoordinator>,
    pending_nonce: Option<[u8; crypto::NONCE_LEN]>,
    used_nonces: HashSet<[u8; crypto::NONCE_LEN]>,
    master_key: Vec<u8>,
    session_key: Option<Vec<u8>>,
}

impl Session {
    pub fn new(pairing_token: String, stream: Arc<dyn StreamCoordinator>) -> Self {
        Self {
            state: HandshakeState::AwaitHello,
            master_key: crypto::derive_master_key(&pairing_token),
            pairing_token,
            stream,
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

        let token = match String::from_utf8(payload) {
            Ok(token) => token,
            Err(_) => {
                return vec![Frame::new(
                    MessageType::PairReject,
                    b"invalid token encoding".to_vec(),
                )];
            }
        };

        if token == self.pairing_token {
            if self.state == HandshakeState::AwaitAuth {
                return vec![Frame::new(
                    MessageType::Error,
                    b"auth in progress".to_vec(),
                )];
            }
            let nonce = self.next_nonce();
            self.pending_nonce = Some(nonce);
            self.state = HandshakeState::AwaitAuth;
            vec![Frame::new(MessageType::AuthChallenge, nonce.to_vec())]
        } else {
            vec![Frame::new(
                MessageType::PairReject,
                b"invalid token".to_vec(),
            )]
        }
    }

    fn handle_auth_response(&mut self, payload: Vec<u8>) -> Vec<Frame> {
        if self.state != HandshakeState::AwaitAuth {
            return vec![Frame::new(
                MessageType::Error,
                b"auth not expected".to_vec(),
            )];
        }

        if payload.len() != crypto::HMAC_LEN {
            return vec![Frame::new(
                MessageType::Error,
                b"invalid auth payload".to_vec(),
            )];
        }

        let nonce = match self.pending_nonce.take() {
            Some(nonce) => nonce,
            None => {
                return vec![Frame::new(
                    MessageType::Error,
                    b"auth challenge missing".to_vec(),
                )];
            }
        };

        if self.used_nonces.contains(&nonce) {
            return vec![Frame::new(
                MessageType::Error,
                b"nonce reused".to_vec(),
            )];
        }

        let session_key = crypto::derive_session_key(&self.master_key, &nonce);
        if !crypto::verify_hmac_sha256(&session_key, &nonce, &payload) {
            return vec![Frame::new(
                MessageType::PairReject,
                b"auth failed".to_vec(),
            )];
        }

        self.used_nonces.insert(nonce);
        self.session_key = Some(session_key);
        self.state = HandshakeState::Paired;
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
            Ok(()) => vec![Frame::new(MessageType::StreamStarted, Vec::new())],
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
            Ok(()) => vec![Frame::new(MessageType::StreamStopped, Vec::new())],
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
}
