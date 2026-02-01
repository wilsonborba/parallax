use std::io::{BufRead, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::Duration;

use crate::capture;
use crate::control::protocol::{read_frame, write_frame};
use crate::control::session::{DaemonState, DaemonStatus, Session, StreamCoordinator};
use crate::encode;
use crate::net;
use crate::stream;

const DEFAULT_SOCKET_PATH: &str = "~/.local/share/prlx/prlx.sock";
const SOCKET_ENV_VAR: &str = "PRLX_SOCKET_PATH";

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
struct StreamState {
    handle: Option<StreamingHandle>,
}

#[derive(Debug)]
struct StreamingHandle {
    stop: Arc<AtomicBool>,
    join: thread::JoinHandle<()>,
}

#[derive(Debug)]
pub struct StreamController {
    state: Mutex<StreamState>,
    config: StreamConfig,
    target: Mutex<Option<String>>,
}

impl StreamController {
    fn new(config: StreamConfig) -> Self {
        Self {
            state: Mutex::new(StreamState { handle: None }),
            config,
            target: Mutex::new(None),
        }
    }

    fn shutdown(&self) -> Result<(), String> {
        let handle = {
            let mut state = self
                .state
                .lock()
                .map_err(|_| "stream lock poisoned".to_string())?;
            state.handle.take()
        };

        if let Some(handle) = handle {
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
    fn start_stream(&self) -> Result<(), String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "stream lock poisoned".to_string())?;

        if state.handle.is_some() {
            return Err("stream already running".to_string());
        }

        let target = {
            let target = self
                .target
                .lock()
                .map_err(|_| "target lock poisoned".to_string())?;
            target.clone().or_else(|| {
                if self.config.target_addr.trim().is_empty()
                    || self.config.target_addr.ends_with(":0")
                {
                    None
                } else {
                    Some(self.config.target_addr.clone())
                }
            })
        };
        let Some(target_addr) = target else {
            return Err("no target configured; waiting for client".to_string());
        };

        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        let mut config = self.config.clone();
        config.target_addr = target_addr;

        let join = thread::spawn(move || {
            if let Err(err) = run_streaming(config, thread_stop) {
                eprintln!("Streaming loop exited: {err}");
            }
        });

        state.handle = Some(StreamingHandle { stop, join });
        Ok(())
    }

    fn stop_stream(&self) -> Result<(), String> {
        let handle = {
            let mut state = self
                .state
                .lock()
                .map_err(|_| "stream lock poisoned".to_string())?;
            state
                .handle
                .take()
                .ok_or("stream not running".to_string())?
        };

        handle.stop.store(true, Ordering::Relaxed);
        handle
            .join
            .join()
            .map_err(|_| "stream thread panicked".to_string())?;
        Ok(())
    }

    fn set_target(&self, target: String) -> Result<(), String> {
        let mut current = self
            .target
            .lock()
            .map_err(|_| "target lock poisoned".to_string())?;
        *current = Some(target);
        Ok(())
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

fn run_streaming(config: StreamConfig, stop: Arc<AtomicBool>) -> Result<(), String> {
    println!("Starting streaming pipeline for {}", config.target_addr);

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

        let encoded_frame = match encoder.encode_frame(&raw_frame) {
            Ok(frame) => frame,
            Err(error) => {
                eprintln!("Encode error: {error}");
                continue;
            }
        };

        let packets = net::packetize_frame(&encoded_frame);
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

        if frame_counter % 60 == 0 {
            println!(
                "Streaming progress: frames_sent={frame_counter}, packets_sent={packet_counter}"
            );
        }
    }

    println!("Streaming loop stopped");
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
                let result = stream_controller.start_stream();
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
                let result = stream_controller.stop_stream();
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
