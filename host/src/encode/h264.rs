//! H.264 encoding pipeline (Debian FFmpeg 7.x + ffmpeg-next 8.x compatible).
//!
//! Notes for your current stack (Debian FFmpeg 7.1.x, ffmpeg-next = "8"):
//! - `ffmpeg_next::util::hwdevice` is not exposed in the safe wrapper API.
//! - A proper VAAPI path requires lower-level FFI (AVHWDeviceContext / AVHWFramesContext).
//! - For a two-week MVP, keep VAAPI as "not implemented" and ship software (`libx264`) first.
//!
//! Output: **Annex B** (start-code delimited NAL units) via `packet.data()`.

use std::collections::VecDeque;

use ffmpeg_next as ffmpeg;

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

pub struct H264Encoder {
    backend: EncoderBackend,
    state: Option<EncoderState>,
    pending: VecDeque<EncodedFrame>,
}

impl std::fmt::Debug for H264Encoder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("H264Encoder")
            .field("backend", &self.backend)
            .field("state", &self.state)
            .field("pending_len", &self.pending.len())
            .finish()
    }
}

#[derive(Debug)]
enum EncoderState {
    Vaapi,
    Software(SoftwareEncoder),
}

struct SoftwareEncoder {
    encoder: ffmpeg::codec::encoder::video::Encoder,
    scaler: ffmpeg::software::scaling::Context,
    yuv: ffmpeg::frame::Video,
    next_pts: i64,
}

impl std::fmt::Debug for SoftwareEncoder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SoftwareEncoder")
            .field("encoder", &"<ffmpeg encoder>")
            .field("scaler", &"<ffmpeg scaler>")
            .field("yuv", &"<ffmpeg frame>")
            .field("next_pts", &self.next_pts)
            .finish()
    }
}

pub fn init(config: H264Config) -> Result<H264Encoder, String> {
    ffmpeg::init().map_err(|e| format!("FFmpeg init failed: {e}"))?;

    if config.prefer_vaapi {
        // Keep the switch for later; MVP uses software.
        println!("VAAPI requested but not implemented in this MVP build; using software.");
    }

    Ok(H264Encoder {
        backend: EncoderBackend::Software,
        state: None,
        pending: VecDeque::new(),
    })
}

pub fn encode_frame(encoder: &mut H264Encoder, raw: &RawFrame) -> Result<EncodedFrame, String> {
    if let Some(frame) = encoder.pending.pop_front() {
        return Ok(frame);
    }

    if encoder.state.is_none() {
        encoder.state = Some(match encoder.backend {
            EncoderBackend::Vaapi => EncoderState::Vaapi,
            EncoderBackend::Software => EncoderState::Software(init_software_encoder(raw)?),
        });
    }

    match encoder.state.as_mut().ok_or("Encoder state missing")? {
        EncoderState::Vaapi => Err(
            "VAAPI backend is not implemented for ffmpeg-next 8 in this MVP (software works)."
                .to_string(),
        ),
        EncoderState::Software(sw) => encode_with_software(sw, raw, &mut encoder.pending),
    }
}

impl H264Encoder {
    pub fn encode_frame(&mut self, raw: &RawFrame) -> Result<EncodedFrame, String> {
        encode_frame(self, raw)
    }
}

fn init_software_encoder(raw: &RawFrame) -> Result<SoftwareEncoder, String> {
    let codec = ffmpeg::codec::encoder::find_by_name("libx264")
        .ok_or("FFmpeg does not expose libx264 (is it built with --enable-libx264?)")?;

    // Create a codec context bound to this codec, then turn it into a video encoder context.
    let ctx = ffmpeg::codec::context::Context::new_with_codec(codec);
    let mut v = ctx
        .encoder()
        .video()
        .map_err(|e| format!("encoder.video() failed: {e}"))?;

    // -----------------------------
    // REQUIRED FOR X264:
    // time_base MUST be set before opening the encoder.
    // -----------------------------
    // 60 fps MVP target
    let tb = ffmpeg::Rational::new(1, 60);
    v.set_time_base(tb);

    // If your ffmpeg-next exposes frame-rate setter, this is useful but not strictly required
    // for x264 to open once time_base is set. We keep it conservative: no frame_rate call
    // here because the signature varies across wrapper versions.

    v.set_width(raw.width);
    v.set_height(raw.height);
    v.set_format(ffmpeg::format::Pixel::YUV420P);

    // Basic GOP for low-latency streaming
    v.set_gop(60);
    v.set_max_b_frames(0);

    // Bitrate is in bits/sec.
    v.set_bit_rate(4_000_000);

    // Open encoder with x264 options. Force AnnexB and low latency-ish behavior.
    let mut opts = ffmpeg::Dictionary::new();
    opts.set("preset", "veryfast");
    opts.set("tune", "zerolatency");
    opts.set("profile", "baseline");
    // Keep bytestream explicit
    opts.set("x264-params", "annexb=1:repeat-headers=1");

    let opened = v
        .open_as_with(codec, opts)
        .map_err(|e| format!("open_as_with(libx264) failed: {e}"))?;

    let input_format = match raw.format {
        RawPixelFormat::Bgra => ffmpeg::format::Pixel::BGRA,
        RawPixelFormat::Rgba => ffmpeg::format::Pixel::RGBA,
    };

    let scaler = ffmpeg::software::scaling::Context::get(
        input_format,
        raw.width,
        raw.height,
        ffmpeg::format::Pixel::YUV420P,
        raw.width,
        raw.height,
        ffmpeg::software::scaling::flag::Flags::BILINEAR,
    )
    .map_err(|e| format!("scaler init failed: {e}"))?;

    let yuv = ffmpeg::frame::Video::new(ffmpeg::format::Pixel::YUV420P, raw.width, raw.height);

    Ok(SoftwareEncoder {
        encoder: opened,
        scaler,
        yuv,
        next_pts: 0,
    })
}

fn encode_with_software(
    sw: &mut SoftwareEncoder,
    raw: &RawFrame,
    pending: &mut VecDeque<EncodedFrame>,
) -> Result<EncodedFrame, String> {
    let input_format = match raw.format {
        RawPixelFormat::Bgra => ffmpeg::format::Pixel::BGRA,
        RawPixelFormat::Rgba => ffmpeg::format::Pixel::RGBA,
    };

    let expected_stride = raw.width as usize * raw.format.bytes_per_pixel();
    if raw.stride != expected_stride {
        return Err("Encoder expects tightly packed BGRA/RGBA data".to_string());
    }

    // Build an input frame and copy raw bytes into plane 0.
    let mut input = ffmpeg::frame::Video::new(input_format, raw.width, raw.height);

    {
        let plane0 = input.data_mut(0);
        if plane0.is_empty() {
            return Err("Input frame plane 0 is empty".to_string());
        }
        if plane0.len() != raw.data.len() {
            return Err(format!(
                "Input plane length mismatch: plane0={} raw={}",
                plane0.len(),
                raw.data.len()
            ));
        }
        plane0.copy_from_slice(&raw.data);
    }

    // Convert RGBA/BGRA -> YUV420P.
    sw.scaler
        .run(&input, &mut sw.yuv)
        .map_err(|e| format!("scale (RGBA/BGRA -> YUV420P) failed: {e}"))?;

    // Provide monotonically increasing timestamps.
    sw.yuv.set_pts(Some(sw.next_pts));
    sw.next_pts += 1;

    // Send and drain.
    sw.encoder
        .send_frame(&sw.yuv)
        .map_err(|e| format!("send_frame failed: {e}"))?;

    drain_packets(&mut sw.encoder, pending)
}

fn drain_packets(
    enc: &mut ffmpeg::codec::encoder::video::Encoder,
    pending: &mut VecDeque<EncodedFrame>,
) -> Result<EncodedFrame, String> {
    let mut pkt = ffmpeg::Packet::empty();

    // We stop draining on the first error (EAGAIN / EOF / etc).
    // This avoids depending on ffmpeg-next error variant names.
    while enc.receive_packet(&mut pkt).is_ok() {
        let data = pkt.data().ok_or("Encoded packet missing data")?.to_vec();

        if data.is_empty() {
            return Err("Encoded packet was empty".to_string());
        }

        let is_keyframe = pkt.is_key();
        pending.push_back(EncodedFrame {
            data,
            is_keyframe,
            format: EncodedFormat::AnnexB,
        });
    }

    pending
        .pop_front()
        .ok_or_else(|| "No encoded packets produced".to_string())
}
