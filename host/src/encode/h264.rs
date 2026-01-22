//! H.264 encoding pipeline.
//!
//! Dependency approach:
//! - Uses the `ffmpeg-next` crate, which binds to the host FFmpeg installation.
//! - Hardware acceleration is attempted via FFmpeg's VAAPI device (`/dev/dri/render*`).
//! - Software fallback uses the `libx264` encoder exposed by FFmpeg.
//!
//! The encoder outputs **Annex B** byte streams (start-code delimited NAL units).
//! This keeps host/client framing consistent and avoids container headers on the wire.

use std::collections::VecDeque;

#[derive(Debug, Clone, Copy)]
pub enum EncoderBackend {
    Vaapi,
    Software,
}

#[derive(Debug, Clone, Copy)]
pub enum RawPixelFormat {
    Bgra,
    Rgba,
}

impl RawPixelFormat {
    fn bytes_per_pixel(self) -> usize {
        4
    }
}

#[derive(Debug, Clone)]
pub struct RawFrame {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub stride: usize,
    pub format: RawPixelFormat,
}

impl RawFrame {
    pub fn new(data: Vec<u8>, width: u32, height: u32, format: RawPixelFormat) -> Self {
        let stride = width as usize * format.bytes_per_pixel();
        Self {
            data,
            width,
            height,
            stride,
            format,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EncodedFormat {
    AnnexB,
}

#[derive(Debug, Clone)]
pub struct EncodedFrame {
    pub data: Vec<u8>,
    pub is_keyframe: bool,
    pub format: EncodedFormat,
}

#[derive(Debug)]
pub struct H264Config {
    pub prefer_vaapi: bool,
}

#[derive(Debug)]
pub struct H264Encoder {
    backend: EncoderBackend,
    state: Option<EncoderState>,
    pending: VecDeque<EncodedFrame>,
}

#[derive(Debug)]
enum EncoderState {
    Vaapi(VaapiEncoder),
    Software(SoftwareEncoder),
}

#[derive(Debug)]
struct VaapiEncoder {
    encoder: ffmpeg_next::encoder::video::Video,
    scaler: ffmpeg_next::software::scaling::Context,
    sw_frame: ffmpeg_next::util::frame::video::Video,
    hw_frame: ffmpeg_next::util::frame::video::Video,
}

#[derive(Debug)]
struct SoftwareEncoder {
    encoder: ffmpeg_next::encoder::video::Video,
    scaler: ffmpeg_next::software::scaling::Context,
    sw_frame: ffmpeg_next::util::frame::video::Video,
}

pub fn init(config: H264Config) -> Result<H264Encoder, String> {
    ffmpeg_next::init().map_err(|error| format!("FFmpeg init failed: {error}"))?;

    if config.prefer_vaapi {
        match init_vaapi() {
            Ok(encoder) => return Ok(encoder),
            Err(error) => {
                println!("VAAPI unavailable, falling back to software: {error}");
            }
        }
    }

    init_software()
}

pub fn encode_frame(encoder: &mut H264Encoder, raw_frame: &RawFrame) -> Result<EncodedFrame, String> {
    if let Some(frame) = encoder.pending.pop_front() {
        return Ok(frame);
    }

    if encoder.state.is_none() {
        let state = match encoder.backend {
            EncoderBackend::Vaapi => EncoderState::Vaapi(init_vaapi_encoder(raw_frame)?),
            EncoderBackend::Software => EncoderState::Software(init_software_encoder(raw_frame)?),
        };
        encoder.state = Some(state);
    }

    match encoder.state.as_mut().ok_or("Encoder state missing")? {
        EncoderState::Vaapi(state) => encode_with_vaapi(state, raw_frame, &mut encoder.pending),
        EncoderState::Software(state) => encode_with_software(state, raw_frame, &mut encoder.pending),
    }
}

fn init_vaapi() -> Result<H264Encoder, String> {
    let device = ffmpeg_next::hardware::Device::create(
        ffmpeg_next::hardware::Type::VAAPI,
        None,
    )
    .map_err(|error| format!("VAAPI device init failed: {error}"))?;

    drop(device);

    Ok(H264Encoder {
        backend: EncoderBackend::Vaapi,
        state: None,
        pending: VecDeque::new(),
    })
}

fn init_software() -> Result<H264Encoder, String> {
    println!("Configuring software H.264 encoder");
    Ok(H264Encoder {
        backend: EncoderBackend::Software,
        state: None,
        pending: VecDeque::new(),
    })
}

fn init_vaapi_encoder(raw_frame: &RawFrame) -> Result<VaapiEncoder, String> {
    let codec = ffmpeg_next::encoder::find_by_name("h264_vaapi")
        .ok_or("FFmpeg does not expose h264_vaapi")?;
    let mut context = codec.video().map_err(|error| format!("VAAPI context: {error}"))?;

    context.set_width(raw_frame.width as i32);
    context.set_height(raw_frame.height as i32);
    context.set_format(ffmpeg_next::format::Pixel::VAAPI);
    context.set_time_base((1, 60));
    context.set_frame_rate(Some((60, 1)));
    context.set_bit_rate(4_000_000);

    let device = ffmpeg_next::hardware::Device::create(
        ffmpeg_next::hardware::Type::VAAPI,
        None,
    )
    .map_err(|error| format!("VAAPI device init failed: {error}"))?;
    context.set_hw_device_context(device);

    let encoder = context
        .open_as(codec)
        .map_err(|error| format!("VAAPI open encoder: {error}"))?;

    let input_format = match raw_frame.format {
        RawPixelFormat::Bgra => ffmpeg_next::format::Pixel::BGRA,
        RawPixelFormat::Rgba => ffmpeg_next::format::Pixel::RGBA,
    };

    let scaler = ffmpeg_next::software::scaling::Context::get(
        input_format,
        raw_frame.width as i32,
        raw_frame.height as i32,
        ffmpeg_next::format::Pixel::NV12,
        raw_frame.width as i32,
        raw_frame.height as i32,
        ffmpeg_next::software::scaling::flag::Flags::BILINEAR,
    )
    .map_err(|error| format!("VAAPI scaler: {error}"))?;

    let sw_frame = ffmpeg_next::util::frame::video::Video::new(
        ffmpeg_next::format::Pixel::NV12,
        raw_frame.width as u32,
        raw_frame.height as u32,
    );
    let hw_frame = ffmpeg_next::util::frame::video::Video::new(
        ffmpeg_next::format::Pixel::VAAPI,
        raw_frame.width as u32,
        raw_frame.height as u32,
    );

    Ok(VaapiEncoder {
        encoder,
        scaler,
        sw_frame,
        hw_frame,
    })
}

fn init_software_encoder(raw_frame: &RawFrame) -> Result<SoftwareEncoder, String> {
    let codec = ffmpeg_next::encoder::find_by_name("libx264")
        .ok_or("FFmpeg does not expose libx264")?;
    let mut context = codec.video().map_err(|error| format!("x264 context: {error}"))?;

    context.set_width(raw_frame.width as i32);
    context.set_height(raw_frame.height as i32);
    context.set_format(ffmpeg_next::format::Pixel::YUV420P);
    context.set_time_base((1, 60));
    context.set_frame_rate(Some((60, 1)));
    context.set_bit_rate(4_000_000);

    let encoder = context
        .open_as(codec)
        .map_err(|error| format!("x264 open encoder: {error}"))?;

    let input_format = match raw_frame.format {
        RawPixelFormat::Bgra => ffmpeg_next::format::Pixel::BGRA,
        RawPixelFormat::Rgba => ffmpeg_next::format::Pixel::RGBA,
    };

    let scaler = ffmpeg_next::software::scaling::Context::get(
        input_format,
        raw_frame.width as i32,
        raw_frame.height as i32,
        ffmpeg_next::format::Pixel::YUV420P,
        raw_frame.width as i32,
        raw_frame.height as i32,
        ffmpeg_next::software::scaling::flag::Flags::BILINEAR,
    )
    .map_err(|error| format!("x264 scaler: {error}"))?;

    let sw_frame = ffmpeg_next::util::frame::video::Video::new(
        ffmpeg_next::format::Pixel::YUV420P,
        raw_frame.width as u32,
        raw_frame.height as u32,
    );

    Ok(SoftwareEncoder {
        encoder,
        scaler,
        sw_frame,
    })
}

fn encode_with_vaapi(
    encoder: &mut VaapiEncoder,
    raw_frame: &RawFrame,
    pending: &mut VecDeque<EncodedFrame>,
) -> Result<EncodedFrame, String> {
    let input_format = match raw_frame.format {
        RawPixelFormat::Bgra => ffmpeg_next::format::Pixel::BGRA,
        RawPixelFormat::Rgba => ffmpeg_next::format::Pixel::RGBA,
    };

    let mut input = ffmpeg_next::util::frame::video::Video::new(
        input_format,
        raw_frame.width,
        raw_frame.height,
    );

    let expected_stride = raw_frame.width as usize * raw_frame.format.bytes_per_pixel();
    if raw_frame.stride != expected_stride {
        return Err("VAAPI encoder expects tightly packed BGRA/RGBA data".to_string());
    }

    input
        .data_mut(0)
        .ok_or("Missing input plane")?
        .copy_from_slice(&raw_frame.data);

    encoder
        .scaler
        .run(&input, &mut encoder.sw_frame)
        .map_err(|error| format!("VAAPI scale: {error}"))?;

    encoder
        .hw_frame
        .transfer_from(&encoder.sw_frame)
        .map_err(|error| format!("VAAPI upload: {error}"))?;

    encoder
        .encoder
        .send_frame(&encoder.hw_frame)
        .map_err(|error| format!("VAAPI send frame: {error}"))?;

    drain_packets(&mut encoder.encoder, pending)
}

fn encode_with_software(
    encoder: &mut SoftwareEncoder,
    raw_frame: &RawFrame,
    pending: &mut VecDeque<EncodedFrame>,
) -> Result<EncodedFrame, String> {
    let input_format = match raw_frame.format {
        RawPixelFormat::Bgra => ffmpeg_next::format::Pixel::BGRA,
        RawPixelFormat::Rgba => ffmpeg_next::format::Pixel::RGBA,
    };

    let mut input = ffmpeg_next::util::frame::video::Video::new(
        input_format,
        raw_frame.width,
        raw_frame.height,
    );

    let expected_stride = raw_frame.width as usize * raw_frame.format.bytes_per_pixel();
    if raw_frame.stride != expected_stride {
        return Err("Software encoder expects tightly packed BGRA/RGBA data".to_string());
    }

    input
        .data_mut(0)
        .ok_or("Missing input plane")?
        .copy_from_slice(&raw_frame.data);

    encoder
        .scaler
        .run(&input, &mut encoder.sw_frame)
        .map_err(|error| format!("x264 scale: {error}"))?;

    encoder
        .encoder
        .send_frame(&encoder.sw_frame)
        .map_err(|error| format!("x264 send frame: {error}"))?;

    drain_packets(&mut encoder.encoder, pending)
}

fn drain_packets(
    encoder: &mut ffmpeg_next::encoder::video::Video,
    pending: &mut VecDeque<EncodedFrame>,
) -> Result<EncodedFrame, String> {
    let mut packet = ffmpeg_next::Packet::empty();
    while encoder
        .receive_packet(&mut packet)
        .map(|_| true)
        .unwrap_or(false)
    {
        let data = packet.data().to_vec();
        let is_keyframe = packet.is_key();
        pending.push_back(EncodedFrame {
            data,
            is_keyframe,
            format: EncodedFormat::AnnexB,
        });
    }

    pending
        .pop_front()
        .ok_or("No encoded packets produced".to_string())
}
