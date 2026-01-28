use std::net::{TcpListener, TcpStream};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::Duration;

use crate::capture;
use crate::control::protocol::{read_frame, write_frame};
use crate::control::session::{Session, StreamCoordinator};
use crate::encode;
use crate::net;
use crate::stream;

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
}

impl StreamController {
    fn new(config: StreamConfig) -> Self {
        Self {
            state: Mutex::new(StreamState { handle: None }),
            config,
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

        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        let config = self.config.clone();
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
            state.handle.take().ok_or("stream not running".to_string())?
        };

        handle.stop.store(true, Ordering::Relaxed);
        handle
            .join
            .join()
            .map_err(|_| "stream thread panicked".to_string())?;
        Ok(())
    }
}

pub fn run(config: ControlConfig) -> Result<(), String> {
    let running = Arc::new(AtomicBool::new(true));
    run_with_shutdown(config, running)
}

pub fn run_with_shutdown(
    config: ControlConfig,
    running: Arc<AtomicBool>,
) -> Result<(), String> {
    let listener = TcpListener::bind(&config.control_bind)
        .map_err(|err| format!("Failed to bind control listener {}: {err}", config.control_bind))?;
    listener
        .set_nonblocking(true)
        .map_err(|err| format!("Failed to set non-blocking mode: {err}"))?;
    println!("Control listener bound on {}", config.control_bind);

    let stream_controller = Arc::new(StreamController::new(config.stream.clone()));

    while running.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((stream, _)) => {
                let pairing_token = config.pairing_token.clone();
                let stream_controller = Arc::clone(&stream_controller);
                thread::spawn(move || {
                    if let Err(err) = handle_client(stream, pairing_token, stream_controller) {
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
) -> Result<(), String> {
    let mut session = Session::new(pairing_token, stream_controller);

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
        let raw_frame = encode::h264::RawFrame::new(
            pixels,
            width,
            height,
            encode::h264::RawPixelFormat::Bgra,
        );
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
