use std::io::{BufRead, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU32, Ordering},
};
use std::thread;
use std::time::{Duration, Instant};
use std::{collections::BTreeMap};

use crate::capture;
use crate::control::protocol::{read_frame, write_frame};
use crate::control::session::{DaemonState, DaemonStatus, Session, StreamCoordinator};
use crate::display;
use crate::encode;
use crate::net;
use crate::stream;

const DEFAULT_SOCKET_PATH: &str = "~/.local/share/prlx/prlx.sock";
const SOCKET_ENV_VAR: &str = "PRLX_SOCKET_PATH";
const MAX_STREAMS: u32 = 3;

#[derive(Debug, Clone)]
pub struct StreamConfig {
    pub display: String,
    pub bind_addr: String,
    pub target_addr: String,
    pub prefer_vaapi: bool,
}

#[derive(Debug, Clone)]
pub struct ControlConfig {
    pub control_bind: String,
    pub pairing_token: String,
    pub stream: StreamConfig,
}

#[derive(Debug)]
struct StreamingHandle {
    stop: Arc<AtomicBool>,
    join: thread::JoinHandle<()>,
}

#[derive(Debug)]
pub struct StreamController {
    slots: Mutex<BTreeMap<u32, StreamSlot>>,
}

#[derive(Debug, Clone)]
struct StreamSlotMeta {
    display: String,
    bind_addr: String,
    target_addr: String,
    prefer_vaapi: bool,
}

#[derive(Debug)]
struct StreamSlot {
    meta: StreamSlotMeta,
    target: Option<String>,
    handle: Option<StreamingHandle>,
    metrics: Arc<StreamMetrics>,
}

#[derive(Debug, Default)]
struct StreamMetrics {
    width: AtomicU32,
    height: AtomicU32,
    fps_x100: AtomicU32,
    bitrate_kbps: AtomicU32,
}

impl StreamController {
    fn new(config: StreamConfig) -> Self {
        let mut slots = BTreeMap::new();
        for stream_id in 1..=MAX_STREAMS {
            let derived = derive_stream_config(&config, stream_id);
            slots.insert(
                stream_id,
                StreamSlot {
                    meta: StreamSlotMeta {
                        display: derived.display,
                        bind_addr: derived.bind_addr,
                        target_addr: derived.target_addr,
                        prefer_vaapi: derived.prefer_vaapi,
                    },
                    target: None,
                    handle: None,
                    metrics: Arc::new(StreamMetrics::default()),
                },
            );
        }
        Self {
            slots: Mutex::new(slots),
        }
    }

    fn shutdown(&self) -> Result<(), String> {
        let handles = {
            let mut slots = self
                .slots
                .lock()
                .map_err(|_| "stream lock poisoned".to_string())?;
            let mut handles = Vec::new();
            for slot in slots.values_mut() {
                if let Some(handle) = slot.handle.take() {
                    handles.push(handle);
                }
            }
            handles
        };

        for handle in handles {
            handle.stop.store(true, Ordering::Relaxed);
            handle
                .join
                .join()
                .map_err(|_| "stream thread panicked".to_string())?;
        }

        Ok(())
    }
}

impl StreamCoordinator for StreamController {
    fn start_stream(&self, stream_id: u32) -> Result<(), String> {
        let mut slots = self
            .slots
            .lock()
            .map_err(|_| "stream lock poisoned".to_string())?;
        let slot = slots
            .get_mut(&stream_id)
            .ok_or_else(|| "stream_id not found".to_string())?;

        if slot.handle.is_some() {
            return Ok(());
        }

        let target = slot
            .target
            .clone()
            .or_else(|| normalize_target(slot.meta.target_addr.clone()));
        let Some(target_addr) = target else {
            return Err("no target configured; waiting for client".to_string());
        };

        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        let config = StreamConfig {
            display: slot.meta.display.clone(),
            bind_addr: slot.meta.bind_addr.clone(),
            target_addr,
            prefer_vaapi: slot.meta.prefer_vaapi,
        };
        slot.metrics.fps_x100.store(0, Ordering::Relaxed);
        slot.metrics.bitrate_kbps.store(0, Ordering::Relaxed);
        let metrics = Arc::clone(&slot.metrics);

        let join = thread::spawn(move || {
            if let Err(err) = run_streaming(stream_id, config, thread_stop, metrics) {
                eprintln!("Streaming loop exited for stream {stream_id}: {err}");
            }
        });

        slot.handle = Some(StreamingHandle { stop, join });
        Ok(())
    }

    fn stop_stream(&self, stream_id: u32) -> Result<(), String> {
        let handle = {
            let mut slots = self
                .slots
                .lock()
                .map_err(|_| "stream lock poisoned".to_string())?;
            let slot = slots
                .get_mut(&stream_id)
                .ok_or_else(|| "stream_id not found".to_string())?;
            slot.handle.take().ok_or("stream not running".to_string())?
        };

        handle.stop.store(true, Ordering::Relaxed);
        handle
            .join
            .join()
            .map_err(|_| "stream thread panicked".to_string())?;
        Ok(())
    }

    fn set_target(&self, target: String) -> Result<(), String> {
        let mut to_restart = Vec::new();
        {
            let mut slots = self
                .slots
                .lock()
                .map_err(|_| "target lock poisoned".to_string())?;
            for (&stream_id, slot) in slots.iter_mut() {
                slot.target = Some(offset_port(&target, stream_id.saturating_sub(1)));
                if slot.handle.is_some() {
                    to_restart.push(stream_id);
                }
            }
        }
        for stream_id in to_restart {
            self.stop_stream(stream_id)?;
            self.start_stream(stream_id)?;
        }
        Ok(())
    }

    fn list_streams(&self) -> Result<String, String> {
        let slots = self
            .slots
            .lock()
            .map_err(|_| "config lock poisoned".to_string())?;
        let mut payload = String::from("protocol=2\nstreams:\n");
        for (&stream_id, slot) in slots.iter() {
            let target = slot
                .target
                .clone()
                .unwrap_or_else(|| slot.meta.target_addr.clone());
            let running = slot.handle.is_some();
            let width = slot.metrics.width.load(Ordering::Relaxed);
            let height = slot.metrics.height.load(Ordering::Relaxed);
            let fps_x100 = slot.metrics.fps_x100.load(Ordering::Relaxed);
            let bitrate_kbps = slot.metrics.bitrate_kbps.load(Ordering::Relaxed);
            let fps = format!("{}.{:02}", fps_x100 / 100, fps_x100 % 100);
            payload.push_str(&format!(
                "{stream_id},{},{},{},{},{},{},{},{},{}\n",
                slot.meta.display,
                slot.meta.bind_addr,
                target,
                slot.meta.prefer_vaapi,
                running,
                width,
                height,
                fps,
                bitrate_kbps
            ));
        }
        Ok(payload)
    }

    fn set_stream_config(
        &self,
        stream_id: u32,
        display: Option<String>,
        bind_addr: Option<String>,
        target_addr: Option<String>,
        prefer_vaapi: Option<bool>,
    ) -> Result<(), String> {
        if let Some(display_id) = display.as_ref() {
            validate_display_id(display_id)?;
        }

        let should_restart;
        let mut changed = false;
        {
            let mut slots = self
                .slots
                .lock()
                .map_err(|_| "config lock poisoned".to_string())?;
            let slot = slots
                .get_mut(&stream_id)
                .ok_or_else(|| "stream_id not found".to_string())?;

            if let Some(v) = display {
                if slot.meta.display != v {
                    slot.meta.display = v;
                    changed = true;
                }
            }
            if let Some(v) = bind_addr {
                if slot.meta.bind_addr != v {
                    slot.meta.bind_addr = v;
                    changed = true;
                }
            }
            if let Some(v) = target_addr {
                if slot.meta.target_addr != v {
                    slot.meta.target_addr = v.clone();
                    slot.target = Some(v);
                    changed = true;
                }
            }
            if let Some(v) = prefer_vaapi {
                if slot.meta.prefer_vaapi != v {
                    slot.meta.prefer_vaapi = v;
                    changed = true;
                }
            }
            should_restart = changed && slot.handle.is_some();
        }

        if should_restart {
            self.stop_stream(stream_id)?;
            self.start_stream(stream_id)?;
        }

        Ok(())
    }
}

fn derive_stream_config(base: &StreamConfig, stream_id: u32) -> StreamConfig {
    let delta = stream_id.saturating_sub(1);
    let display = if stream_id == 1 {
        base.display.clone()
    } else {
        format!("prlx-v{stream_id}")
    };
    StreamConfig {
        display,
        bind_addr: offset_port(&base.bind_addr, delta),
        target_addr: offset_port(&base.target_addr, delta),
        prefer_vaapi: base.prefer_vaapi,
    }
}

fn normalize_target(target_addr: String) -> Option<String> {
    let trimmed = target_addr.trim();
    if trimmed.is_empty() || trimmed.ends_with(":0") {
        None
    } else {
        Some(target_addr)
    }
}

fn offset_port(addr: &str, delta: u32) -> String {
    if delta == 0 {
        return addr.to_string();
    }
    let Some((host, port)) = split_host_port(addr) else {
        return addr.to_string();
    };
    let Some(offset) = port.checked_add(delta as u16) else {
        return addr.to_string();
    };
    format!("{host}:{offset}")
}

fn split_host_port(addr: &str) -> Option<(&str, u16)> {
    let (host, port_str) = addr.rsplit_once(':')?;
    let port = port_str.parse::<u16>().ok()?;
    Some((host, port))
}

fn validate_display_id(display_id: &str) -> Result<(), String> {
    if display_id.trim().is_empty() {
        return Err("display id is required".to_string());
    }
    // Allow explicit X11 display selectors while still validating monitor ids.
    if display_id.starts_with(':') {
        return Ok(());
    }

    let physical = display::list_displays().map_err(|err| err.to_string())?;
    let virtuals = display::list_virtual_displays().map_err(|err| err.to_string())?;
    let physical_match = physical
        .iter()
        .any(|d| d.id == display_id || d.name == display_id);
    let virtual_match = virtuals.iter().any(|d| d.id == display_id && d.enabled);
    if physical_match || virtual_match {
        Ok(())
    } else {
        Err("display_id not found".to_string())
    }
}

pub fn run(config: ControlConfig) -> Result<(), String> {
    let running = Arc::new(AtomicBool::new(true));
    run_with_shutdown(config, running)
}

pub fn run_with_shutdown(config: ControlConfig, running: Arc<AtomicBool>) -> Result<(), String> {
    let listener = TcpListener::bind(&config.control_bind).map_err(|err| {
        format!(
            "Failed to bind control listener {}: {err}",
            config.control_bind
        )
    })?;
    listener
        .set_nonblocking(true)
        .map_err(|err| format!("Failed to set non-blocking mode: {err}"))?;
    let control_addr = listener
        .local_addr()
        .map_err(|err| format!("Failed to read listener addr: {err}"))?;
    println!("Control listener bound on {}", control_addr);

    let stream_controller = Arc::new(StreamController::new(config.stream.clone()));
    let pairing_token = resolve_pairing_token(&config.pairing_token);

    let local_ip = resolve_local_ip().unwrap_or_else(|| "127.0.0.1".to_string());
    let mut qr_uri = format!("prlx://{}:{}", local_ip, control_addr.port());
    let mut query_parts = Vec::new();
    if let Some(stream_port) = parse_port(&config.stream.target_addr) {
        query_parts.push(format!("streamPort={stream_port}"));
    }
    if !pairing_token.is_empty() {
        query_parts.push(format!("pin={pairing_token}"));
    }
    if !query_parts.is_empty() {
        qr_uri.push('?');
        qr_uri.push_str(&query_parts.join("&"));
    }

    let daemon_status = Arc::new(Mutex::new(DaemonStatus {
        state: DaemonState::Waiting,
        pin: Some(pairing_token.clone()),
        qr_uri: Some(qr_uri),
    }));

    // ✅ IMPORTANT: print the resolved socket path so you can verify it matches the UI.
    let socket_path = resolve_socket_path();
    println!("Status socket path resolved to {:?}", socket_path);

    let status_stream_controller = Arc::clone(&stream_controller);
    let status_state = Arc::clone(&daemon_status);
    let status_running = Arc::clone(&running);

    thread::spawn(move || {
        if let Err(err) = run_status_socket(
            socket_path,
            status_stream_controller,
            status_state,
            status_running,
        ) {
            eprintln!("Status socket error: {err}");
        }
    });

    while running.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((stream, peer_addr)) => {
                let pairing_token = pairing_token.clone();
                let stream_controller = Arc::clone(&stream_controller);
                let daemon_status = Arc::clone(&daemon_status);
                let default_port = parse_port(&config.stream.target_addr);
                thread::spawn(move || {
                    if let Err(err) = handle_client(
                        stream,
                        pairing_token,
                        stream_controller,
                        daemon_status,
                        peer_addr,
                        default_port,
                    ) {
                        eprintln!("Control session error: {err}");
                    }
                });
            }
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(200));
            }
            Err(err) => {
                eprintln!("Control accept failed: {err}");
                thread::sleep(Duration::from_millis(200));
            }
        }
    }

    println!("Shutting down control server");
    stream_controller.shutdown()?;
    Ok(())
}

fn handle_client(
    mut stream: TcpStream,
    pairing_token: String,
    stream_controller: Arc<StreamController>,
    daemon_status: Arc<Mutex<DaemonStatus>>,
    client_addr: SocketAddr,
    default_target_port: Option<u16>,
) -> Result<(), String> {
    let mut session = Session::new(
        pairing_token,
        stream_controller,
        daemon_status,
        client_addr,
        default_target_port,
    );

    loop {
        let frame = match read_frame(&mut stream)? {
            Some(frame) => frame,
            None => return Ok(()),
        };

        let responses = session.handle_frame(frame);
        for response in responses {
            write_frame(&mut stream, &response)?;
        }
    }
}

fn run_streaming(
    stream_id: u32,
    config: StreamConfig,
    stop: Arc<AtomicBool>,
    metrics: Arc<StreamMetrics>,
) -> Result<(), String> {
    println!(
        "Starting streaming pipeline stream_id={} target={}",
        stream_id, config.target_addr
    );

    let capture = capture::x11::init(capture::x11::X11CaptureConfig {
        display: config.display.clone(),
    })
    .map_err(|err| format!("Failed to initialize X11 capture: {err}"))?;

    let encoder = encode::h264::init(encode::h264::H264Config {
        prefer_vaapi: config.prefer_vaapi,
    })
    .map_err(|err| format!("Failed to initialize H.264 encoder: {err}"))?;

    let streamer = stream::udp::init(stream::udp::UdpConfig {
        bind_addr: config.bind_addr.clone(),
        target_addr: config.target_addr.clone(),
    })
    .map_err(|err| format!("Failed to initialize UDP streamer: {err}"))?;

    println!(
        "Pipeline ready: capture={:?}, encoder={:?}, stream={:?}",
        capture, encoder, streamer
    );

    let mut capture = capture;
    let mut encoder = encoder;
    let mut frame_counter: u64 = 0;
    let mut packet_counter: u64 = 0;
    let mut bytes_counter: u64 = 0;
    let mut window_frames: u64 = 0;
    let mut window_bytes: u64 = 0;
    let mut window_start = Instant::now();

    while !stop.load(Ordering::Relaxed) {
        let (pixels, width, height) = match capture.next_frame() {
            Ok(frame) => frame,
            Err(error) => {
                eprintln!("Capture error: {error}");
                continue;
            }
        };

        let raw_frame =
            encode::h264::RawFrame::new(pixels, width, height, encode::h264::RawPixelFormat::Bgra);
        metrics.width.store(width, Ordering::Relaxed);
        metrics.height.store(height, Ordering::Relaxed);

        let encoded_frame = match encoder.encode_frame(&raw_frame) {
            Ok(frame) => frame,
            Err(error) => {
                eprintln!("Encode error: {error}");
                continue;
            }
        };

        let encoded_size = encoded_frame.data.len() as u64;
        let packets = net::packetize_frame(stream_id, &encoded_frame);
        for packet in &packets {
            if stop.load(Ordering::Relaxed) {
                break;
            }
            if let Err(error) = streamer.send_packet(packet) {
                eprintln!("UDP send error: {error}");
                break;
            }
        }

        frame_counter += 1;
        packet_counter += packets.len() as u64;
        bytes_counter += encoded_size;
        window_frames += 1;
        window_bytes += encoded_size;

        let elapsed = window_start.elapsed();
        if elapsed >= Duration::from_secs(1) {
            let elapsed_secs = elapsed.as_secs_f64().max(0.001);
            let fps_x100 = ((window_frames as f64 * 100.0) / elapsed_secs) as u32;
            let bitrate_kbps = (((window_bytes as f64 * 8.0) / elapsed_secs) / 1000.0) as u32;
            metrics.fps_x100.store(fps_x100, Ordering::Relaxed);
            metrics.bitrate_kbps.store(bitrate_kbps, Ordering::Relaxed);
            window_frames = 0;
            window_bytes = 0;
            window_start = Instant::now();
        }

        if frame_counter % 60 == 0 {
            println!(
                "Streaming progress stream_id={stream_id}: frames_sent={frame_counter}, packets_sent={packet_counter}, bytes_sent={bytes_counter}"
            );
        }
    }

    metrics.fps_x100.store(0, Ordering::Relaxed);
    metrics.bitrate_kbps.store(0, Ordering::Relaxed);
    println!("Streaming loop stopped stream_id={stream_id}");
    Ok(())
}

fn run_status_socket(
    path: PathBuf,
    stream_controller: Arc<StreamController>,
    daemon_status: Arc<Mutex<DaemonStatus>>,
    running: Arc<AtomicBool>,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| format!("Failed to create socket directory {parent:?}: {err}"))?;
    }

    // remove stale socket
    if path.exists() {
        let _ = std::fs::remove_file(&path);
    }

    let listener = UnixListener::bind(&path)
        .map_err(|err| format!("Failed to bind socket {path:?}: {err}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|err| format!("Failed to set socket non-blocking: {err}"))?;

    // ✅ This print is what you were missing.
    println!("Status socket bound on {:?}", path);

    while running.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((stream, _)) => {
                let controller = Arc::clone(&stream_controller);
                let status = Arc::clone(&daemon_status);
                thread::spawn(move || {
                    if let Err(err) = handle_status_client(stream, controller, status) {
                        eprintln!("Status client error: {err}");
                    }
                });
            }
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(100));
            }
            Err(err) => return Err(format!("Status accept error: {err}")),
        }
    }
    Ok(())
}

fn handle_status_client(
    mut stream: UnixStream,
    stream_controller: Arc<StreamController>,
    daemon_status: Arc<Mutex<DaemonStatus>>,
) -> Result<(), String> {
    let mut buffer = String::new();
    let mut reader = std::io::BufReader::new(stream.try_clone().map_err(|err| err.to_string())?);

    loop {
        buffer.clear();
        if reader
            .read_line(&mut buffer)
            .map_err(|err| err.to_string())?
            == 0
        {
            return Ok(());
        }

        let line = buffer.trim();
        match line {
            "status" => {
                let status = daemon_status
                    .lock()
                    .map_err(|_| "status lock poisoned".to_string())?;
                write_status(&mut stream, &status)?;
            }
            "start" => {
                let result = stream_controller.start_stream(1);
                if let Ok(mut status) = daemon_status.lock() {
                    if result.is_ok() {
                        status.state = DaemonState::Streaming;
                    }
                }
                let status = daemon_status
                    .lock()
                    .map_err(|_| "status lock poisoned".to_string())?;
                write_status(&mut stream, &status)?;
            }
            l if l.starts_with("start ") => {
                let stream_id = l
                    .strip_prefix("start ")
                    .and_then(|v| v.parse::<u32>().ok())
                    .unwrap_or(1);
                let result = stream_controller.start_stream(stream_id);
                if let Ok(mut status) = daemon_status.lock() {
                    if result.is_ok() {
                        status.state = DaemonState::Streaming;
                    }
                }
                let status = daemon_status
                    .lock()
                    .map_err(|_| "status lock poisoned".to_string())?;
                write_status(&mut stream, &status)?;
            }
            "stop" => {
                let result = stream_controller.stop_stream(1);
                if let Ok(mut status) = daemon_status.lock() {
                    if result.is_ok() {
                        status.state = DaemonState::Connected;
                    }
                }
                let status = daemon_status
                    .lock()
                    .map_err(|_| "status lock poisoned".to_string())?;
                write_status(&mut stream, &status)?;
            }
            l if l.starts_with("stop ") => {
                let stream_id = l
                    .strip_prefix("stop ")
                    .and_then(|v| v.parse::<u32>().ok())
                    .unwrap_or(1);
                let result = stream_controller.stop_stream(stream_id);
                if let Ok(mut status) = daemon_status.lock() {
                    if result.is_ok() {
                        status.state = DaemonState::Connected;
                    }
                }
                let status = daemon_status
                    .lock()
                    .map_err(|_| "status lock poisoned".to_string())?;
                write_status(&mut stream, &status)?;
            }
            "streams" => {
                let payload = stream_controller.list_streams()?;
                write_streams(&mut stream, &payload)?;
            }
            _ => {
                let status = daemon_status
                    .lock()
                    .map_err(|_| "status lock poisoned".to_string())?;
                write_status(&mut stream, &status)?;
            }
        }
    }
}

fn write_status(stream: &mut UnixStream, status: &DaemonStatus) -> Result<(), String> {
    let state = match status.state {
        DaemonState::Idle => "idle",
        DaemonState::Waiting => "waiting",
        DaemonState::Connected => "connected",
        DaemonState::Streaming => "streaming",
    };

    let mut line = format!("state={state}");
    if let Some(pin) = &status.pin {
        line.push_str(&format!(" pin={pin}"));
    }
    if let Some(qr) = &status.qr_uri {
        line.push_str(&format!(" qr={qr}"));
    }
    line.push('\n');

    stream
        .write_all(line.as_bytes())
        .map_err(|err| format!("Failed to write status: {err}"))?;
    Ok(())
}

fn write_streams(stream: &mut UnixStream, payload: &str) -> Result<(), String> {
    stream
        .write_all(payload.as_bytes())
        .map_err(|err| format!("Failed to write streams payload: {err}"))?;
    if !payload.ends_with('\n') {
        stream
            .write_all(b"\n")
            .map_err(|err| format!("Failed to finalize streams payload: {err}"))?;
    }
    Ok(())
}

fn resolve_pairing_token(value: &str) -> String {
    if value != "auto" {
        return value.to_string();
    }
    let mut rng = rand::thread_rng();
    let pin: u32 = rand::Rng::gen_range(&mut rng, 100_000..=999_999);
    format!("{pin}")
}

fn parse_port(addr: &str) -> Option<u16> {
    addr.rsplit_once(':')
        .and_then(|(_, port)| port.parse().ok())
}

fn resolve_local_ip() -> Option<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|addr| addr.ip().to_string())
}

/// Robust home dir: uses HOME, else libc getpwuid fallback.
/// This fixes cases where "~/" would otherwise NOT expand and you end up creating ./~/.local/...
fn home_dir() -> Option<PathBuf> {
    if let Some(home) = std::env::var_os("HOME") {
        if !home.is_empty() {
            return Some(PathBuf::from(home));
        }
    }

    unsafe {
        let uid = libc::getuid();
        let pw = libc::getpwuid(uid);
        if pw.is_null() {
            return None;
        }
        let dir = (*pw).pw_dir;
        if dir.is_null() {
            return None;
        }
        let cstr = std::ffi::CStr::from_ptr(dir);
        Some(PathBuf::from(cstr.to_string_lossy().to_string()))
    }
}

fn expand_socket_path(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = home_dir() {
            return home.join(rest);
        }
        // If HOME can't be resolved, do NOT leave "~" in the path silently.
        // Fall back to /tmp to avoid "invisible" ./~ directory bugs.
        return PathBuf::from("/tmp").join(rest);
    }
    PathBuf::from(path)
}

fn resolve_socket_path() -> PathBuf {
    let candidate =
        std::env::var(SOCKET_ENV_VAR).unwrap_or_else(|_| DEFAULT_SOCKET_PATH.to_string());
    expand_socket_path(&candidate)
}
