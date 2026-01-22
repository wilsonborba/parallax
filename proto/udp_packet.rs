/// UDP packet framing constants for the Parallax wire format.
///
/// All multi-byte fields are serialized in big-endian (network byte order).

pub const MAGIC: u32 = 0x5052_4c58; // "PRLX"
pub const VERSION: u8 = 1;
pub const HEADER_SIZE_BYTES: usize = 24;

pub const FLAG_KEYFRAME: u16 = 1 << 0;
pub const FLAG_CONFIG: u16 = 1 << 1;
pub const FLAG_END_OF_FRAME: u16 = 1 << 2;
pub const FLAG_DISCONTINUITY: u16 = 1 << 3;

pub const PAYLOAD_VIDEO: u8 = 0x01;
pub const PAYLOAD_AUDIO: u8 = 0x02;
pub const PAYLOAD_CONTROL: u8 = 0x03;

pub const MTU_BYTES: usize = 1200;
pub const MAX_PAYLOAD_SIZE: usize = MTU_BYTES - HEADER_SIZE_BYTES;

#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Default)]
pub struct UdpPacketHeader {
    pub magic: u32,
    pub version: u8,
    pub header_length: u8,
    pub flags: u16,
    pub stream_id: u32,
    pub frame_id: u32,
    pub packet_id: u16,
    pub packet_count: u16,
    pub payload_type: u8,
    pub reserved: u8,
    pub payload_length: u16,
}
