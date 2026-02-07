use std::sync::atomic::{AtomicU32, Ordering};

use crate::encode::h264::EncodedFrame;

const MAGIC: [u8; 4] = *b"PRLX";
const VERSION: u8 = 1;
const HEADER_LENGTH: u8 = 24;
const MAX_MTU: usize = 1200;
const MAX_PAYLOAD_SIZE: usize = MAX_MTU - HEADER_LENGTH as usize;
const PAYLOAD_TYPE_VIDEO: u8 = 0x01;
const RESERVED: u8 = 0;
const STREAM_ID: u32 = 1;

const FLAG_KEYFRAME: u16 = 1 << 0;
const FLAG_CONFIG: u16 = 1 << 1;
const FLAG_END_OF_FRAME: u16 = 1 << 2;

static FRAME_COUNTER: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Clone)]
pub struct PacketHeader {
    pub flags: u16,
    pub stream_id: u32,
    pub frame_id: u32,
    pub packet_id: u16,
    pub packet_count: u16,
    pub payload_type: u8,
    pub payload_length: u16,
}

impl PacketHeader {
    fn encode(&self) -> [u8; HEADER_LENGTH as usize] {
        let mut header = [0u8; HEADER_LENGTH as usize];
        header[0..4].copy_from_slice(&MAGIC);
        header[4] = VERSION;
        header[5] = HEADER_LENGTH;
        header[6..8].copy_from_slice(&self.flags.to_be_bytes());
        header[8..12].copy_from_slice(&self.stream_id.to_be_bytes());
        header[12..16].copy_from_slice(&self.frame_id.to_be_bytes());
        header[16..18].copy_from_slice(&self.packet_id.to_be_bytes());
        header[18..20].copy_from_slice(&self.packet_count.to_be_bytes());
        header[20] = self.payload_type;
        header[21] = RESERVED;
        header[22..24].copy_from_slice(&self.payload_length.to_be_bytes());
        header
    }
}

#[derive(Debug, Clone)]
pub struct UdpPacket {
    pub header: PacketHeader,
    pub payload: Vec<u8>,
}

impl UdpPacket {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(HEADER_LENGTH as usize + self.payload.len());
        bytes.extend_from_slice(&self.header.encode());
        bytes.extend_from_slice(&self.payload);
        bytes
    }
}

pub fn packetize_frame(encoded_frame: &EncodedFrame) -> Vec<UdpPacket> {
    let frame_id = FRAME_COUNTER.fetch_add(1, Ordering::Relaxed);
    let mut flags = 0u16;

    if encoded_frame.is_keyframe {
        flags |= FLAG_KEYFRAME;
    }

    if contains_config_nal(&encoded_frame.data) {
        flags |= FLAG_CONFIG;
    }

    if encoded_frame.data.is_empty() {
        let header = PacketHeader {
            flags: flags | FLAG_END_OF_FRAME,
            stream_id: STREAM_ID,
            frame_id,
            packet_id: 0,
            packet_count: 1,
            payload_type: PAYLOAD_TYPE_VIDEO,
            payload_length: 0,
        };

        return vec![UdpPacket {
            header,
            payload: Vec::new(),
        }];
    }

    let total_packets = chunk_count(encoded_frame.data.len(), MAX_PAYLOAD_SIZE);
    let mut packets = Vec::with_capacity(total_packets as usize);

    for (index, chunk) in encoded_frame
        .data
        .chunks(MAX_PAYLOAD_SIZE)
        .enumerate()
    {
        let packet_id = index as u16;
        let mut packet_flags = flags;
        if packet_id == total_packets - 1 {
            packet_flags |= FLAG_END_OF_FRAME;
        }

        let header = PacketHeader {
            flags: packet_flags,
            stream_id: STREAM_ID,
            frame_id,
            packet_id,
            packet_count: total_packets,
            payload_type: PAYLOAD_TYPE_VIDEO,
            payload_length: chunk.len() as u16,
        };

        packets.push(UdpPacket {
            header,
            payload: chunk.to_vec(),
        });
    }

    packets
}

fn chunk_count(total_len: usize, chunk_size: usize) -> u16 {
    if total_len == 0 {
        return 1;
    }

    let count = (total_len + chunk_size - 1) / chunk_size;
    count.min(u16::MAX as usize) as u16
}

fn contains_config_nal(data: &[u8]) -> bool {
    let mut offset = 0;
    while let Some(start) = find_start_code(data, offset) {
        let nal_start = start.code_start + start.code_len;
        if nal_start >= data.len() {
            break;
        }

        let nal_header = data[nal_start];
        let nal_type = nal_header & 0x1f;
        if nal_type == 7 || nal_type == 8 {
            return true;
        }

        offset = nal_start + 1;
    }

    false
}

struct StartCode {
    code_start: usize,
    code_len: usize,
}

fn find_start_code(data: &[u8], offset: usize) -> Option<StartCode> {
    let mut i = offset;
    while i + 3 < data.len() {
        if data[i] == 0 && data[i + 1] == 0 {
            if data[i + 2] == 1 {
                return Some(StartCode {
                    code_start: i,
                    code_len: 3,
                });
            }
            if i + 3 < data.len() && data[i + 2] == 0 && data[i + 3] == 1 {
                return Some(StartCode {
                    code_start: i,
                    code_len: 4,
                });
            }
        }
        i += 1;
    }
    None
}
