use std::sync::Arc;

use crate::control::protocol::{Frame, MessageType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandshakeState {
    AwaitHello,
    AwaitPairing,
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
}

impl Session {
    pub fn new(pairing_token: String, stream: Arc<dyn StreamCoordinator>) -> Self {
        Self {
            state: HandshakeState::AwaitHello,
            pairing_token,
            stream,
        }
    }

    pub fn handle_frame(&mut self, frame: Frame) -> Vec<Frame> {
        match frame.message_type {
            MessageType::Hello => self.handle_hello(),
            MessageType::PairRequest => self.handle_pair_request(frame.payload),
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
            self.state = HandshakeState::Paired;
            vec![Frame::new(MessageType::PairAccept, Vec::new())]
        } else {
            vec![Frame::new(
                MessageType::PairReject,
                b"invalid token".to_vec(),
            )]
        }
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
}
