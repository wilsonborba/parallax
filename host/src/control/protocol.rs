use std::io::{Read, Write};
use std::net::TcpStream;

const PROTOCOL_VERSION: u8 = 1;
const HEADER_LEN: usize = 4;
const MAX_PAYLOAD_LEN: usize = u16::MAX as usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageType {
    Hello = 0x01,
    HelloAck = 0x02,
    PairRequest = 0x03,
    PairAccept = 0x04,
    PairReject = 0x05,
    AuthChallenge = 0x06,
    AuthResponse = 0x07,
    StartStream = 0x10,
    StopStream = 0x11,
    StreamStarted = 0x12,
    StreamStopped = 0x13,
    ListStreams = 0x14,
    Streams = 0x15,
    SetStreamConfig = 0x16,
    StreamConfigAck = 0x17,
    Ping = 0x20,
    Pong = 0x21,
    ListDisplays = 0x30,
    Displays = 0x31,
    AddVirtualDisplay = 0x32,
    RemoveVirtualDisplay = 0x33,
    DisplayOpAck = 0x34,
    Error = 0x7f,
}

impl TryFrom<u8> for MessageType {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, <Self as TryFrom<u8>>::Error> {
        match value {
            0x01 => Ok(Self::Hello),
            0x02 => Ok(Self::HelloAck),
            0x03 => Ok(Self::PairRequest),
            0x04 => Ok(Self::PairAccept),
            0x05 => Ok(Self::PairReject),
            0x06 => Ok(Self::AuthChallenge),
            0x07 => Ok(Self::AuthResponse),
            0x10 => Ok(Self::StartStream),
            0x11 => Ok(Self::StopStream),
            0x12 => Ok(Self::StreamStarted),
            0x13 => Ok(Self::StreamStopped),
            0x14 => Ok(Self::ListStreams),
            0x15 => Ok(Self::Streams),
            0x16 => Ok(Self::SetStreamConfig),
            0x17 => Ok(Self::StreamConfigAck),
            0x20 => Ok(Self::Ping),
            0x21 => Ok(Self::Pong),
            0x30 => Ok(Self::ListDisplays),
            0x31 => Ok(Self::Displays),
            0x32 => Ok(Self::AddVirtualDisplay),
            0x33 => Ok(Self::RemoveVirtualDisplay),
            0x34 => Ok(Self::DisplayOpAck),
            0x7f => Ok(Self::Error),
            other => Err(format!("Unknown message type: {other}")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub message_type: MessageType,
    pub payload: Vec<u8>,
}

impl Frame {
    pub fn new(message_type: MessageType, payload: Vec<u8>) -> Self {
        Self {
            message_type,
            payload,
        }
    }

    pub fn encode(&self) -> Result<Vec<u8>, String> {
        if self.payload.len() > MAX_PAYLOAD_LEN {
            return Err(format!(
                "Payload too large: {} bytes",
                self.payload.len()
            ));
        }

        let mut bytes = Vec::with_capacity(HEADER_LEN + self.payload.len());
        bytes.push(PROTOCOL_VERSION);
        bytes.push(self.message_type as u8);
        bytes.extend_from_slice(&(self.payload.len() as u16).to_be_bytes());
        bytes.extend_from_slice(&self.payload);
        Ok(bytes)
    }
}

pub fn read_frame(stream: &mut TcpStream) -> Result<Option<Frame>, String> {
    let mut header = [0u8; HEADER_LEN];
    match stream.read_exact(&mut header) {
        Ok(()) => {}
        Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(err) => return Err(format!("Failed to read frame header: {err}")),
    }

    if header[0] != PROTOCOL_VERSION {
        return Err(format!(
            "Unsupported protocol version: {}",
            header[0]
        ));
    }

    let message_type = MessageType::try_from(header[1])?;
    let payload_len = u16::from_be_bytes([header[2], header[3]]) as usize;

    if payload_len > MAX_PAYLOAD_LEN {
        return Err(format!("Frame payload too large: {payload_len} bytes"));
    }

    let mut payload = vec![0u8; payload_len];
    if payload_len > 0 {
        stream
            .read_exact(&mut payload)
            .map_err(|err| format!("Failed to read frame payload: {err}"))?;
    }

    Ok(Some(Frame {
        message_type,
        payload,
    }))
}

pub fn write_frame(stream: &mut TcpStream, frame: &Frame) -> Result<(), String> {
    let bytes = frame.encode()?;
    stream
        .write_all(&bytes)
        .map_err(|err| format!("Failed to write frame: {err}"))?;
    stream
        .flush()
        .map_err(|err| format!("Failed to flush frame: {err}"))
}
