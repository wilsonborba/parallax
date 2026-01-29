use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::time::{Duration, Instant};

use eframe::egui::{self, Color32, ColorImage, RichText, TextureHandle};
use qrcode::QrCode;

const DEFAULT_SOCKET_PATH: &str = "~/.local/share/prlx/prlx.sock";
const STATUS_POLL_INTERVAL: Duration = Duration::from_secs(2);
const RECONNECT_INTERVAL: Duration = Duration::from_secs(3);
const SPAWN_INTERVAL: Duration = Duration::from_secs(5);
const SPAWN_RETRY_DELAY: Duration = Duration::from_secs(10);
const EXTERNAL_DAEMON_RETRY_DELAY: Duration = Duration::from_secs(15);

fn main() -> eframe::Result<()> {
    let socket_path = expand_path(DEFAULT_SOCKET_PATH);
    let native_options = eframe::NativeOptions::default();
    let (shutdown_tx, shutdown_rx) = mpsc::channel();
    let handler_tx = shutdown_tx.clone();
    if let Err(err) = ctrlc::set_handler(move || {
        let _ = handler_tx.send(());
    }) {
        eprintln!("Failed to install signal handler: {err}");
    }
    eframe::run_native(
        "Parallax Host UI",
        native_options,
        Box::new(move |cc| Box::new(HostUiApp::new(cc, socket_path, shutdown_rx))),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UiState {
    Idle,
    Waiting,
    Connected,
    Streaming,
}

impl UiState {
    fn label(self) -> &'static str {
        match self {
            Self::Idle => "Idle",
            Self::Waiting => "Waiting",
            Self::Connected => "Connected",
            Self::Streaming => "Streaming",
        }
    }
}

#[derive(Debug, Clone)]
struct DaemonStatus {
    state: UiState,
    pin: Option<String>,
    qr_uri: Option<String>,
}

impl Default for DaemonStatus {
    fn default() -> Self {
        Self {
            state: UiState::Idle,
            pin: None,
            qr_uri: None,
        }
    }
}

#[derive(Debug)]
enum DaemonEvent {
    Status(DaemonStatus),
    Error(String),
    Warning(String),
}

#[derive(Debug)]
enum DaemonCommand {
    Refresh,
    StartStream,
    StopStream,
    Shutdown,
}

struct DaemonHandle {
    command_tx: Sender<DaemonCommand>,
    event_rx: Receiver<DaemonEvent>,
}

impl DaemonHandle {
    fn new(socket_path: PathBuf) -> Self {
        let (command_tx, command_rx) = mpsc::channel();
        let (event_tx, event_rx) = mpsc::channel();

        std::thread::spawn(move || {
            let mut client = DaemonClient::new(socket_path, event_tx);
            client.run(command_rx);
        });

        Self {
            command_tx,
            event_rx,
        }
    }

    fn send(&self, command: DaemonCommand) {
        let _ = self.command_tx.send(command);
    }

    fn try_recv(&self) -> Option<DaemonEvent> {
        self.event_rx.try_recv().ok()
    }
}

impl Drop for DaemonHandle {
    fn drop(&mut self) {
        let _ = self.command_tx.send(DaemonCommand::Shutdown);
    }
}

struct HostUiApp {
    daemon: DaemonHandle,
    status: DaemonStatus,
    last_error: Option<String>,
    last_warning: Option<String>,
    qr_texture: Option<TextureHandle>,
    qr_payload: Option<String>,
    shutdown_rx: Receiver<()>,
    shutdown_initiated: bool,
}

impl HostUiApp {
    fn new(
        cc: &eframe::CreationContext<'_>,
        socket_path: PathBuf,
        shutdown_rx: Receiver<()>,
    ) -> Self {
        let daemon = DaemonHandle::new(socket_path);
        let mut app = Self {
            daemon,
            status: DaemonStatus::default(),
            last_error: None,
            last_warning: None,
            qr_texture: None,
            qr_payload: None,
            shutdown_rx,
            shutdown_initiated: false,
        };
        app.refresh_qr_texture(&cc.egui_ctx);
        app.daemon.send(DaemonCommand::Refresh);
        app
    }

    fn refresh_qr_texture(&mut self, ctx: &egui::Context) {
        let payload = match &self.status.qr_uri {
            Some(payload) => payload.clone(),
            None => {
                self.qr_texture = None;
                self.qr_payload = None;
                return;
            }
        };

        if self.qr_payload.as_deref() == Some(payload.as_str()) {
            return;
        }

        if let Some(image) = qr_to_image(&payload, 4) {
            self.qr_texture =
                Some(ctx.load_texture("pairing_qr", image, egui::TextureOptions::NEAREST));
            self.qr_payload = Some(payload);
        }
    }
}

impl eframe::App for HostUiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.shutdown_initiated {
            match self.shutdown_rx.try_recv() {
                Ok(()) => {
                    self.shutdown_initiated = true;
                    self.daemon.send(DaemonCommand::Shutdown);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    self.shutdown_initiated = true;
                }
            }
        }

        while let Some(event) = self.daemon.try_recv() {
            match event {
                DaemonEvent::Status(status) => {
                    self.status = status;
                    self.last_warning = None;
                }
                DaemonEvent::Error(err) => {
                    self.last_error = Some(err);
                }
                DaemonEvent::Warning(warning) => {
                    self.last_warning = Some(warning);
                }
            }
        }

        self.refresh_qr_texture(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Parallax Host");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("State:");
                ui.label(RichText::new(self.status.state.label()).strong());
            });

            if let Some(err) = &self.last_error {
                ui.add_space(8.0);
                ui.colored_label(Color32::RED, format!("Daemon error: {err}"));
            }

            if let Some(warning) = &self.last_warning {
                ui.add_space(8.0);
                ui.colored_label(Color32::YELLOW, warning);
            }

            ui.add_space(16.0);

            ui.horizontal(|ui| {
                if ui
                    .add_enabled(
                        self.status.state != UiState::Streaming,
                        egui::Button::new("Start"),
                    )
                    .clicked()
                {
                    self.daemon.send(DaemonCommand::StartStream);
                }

                if ui
                    .add_enabled(
                        self.status.state == UiState::Streaming,
                        egui::Button::new("Stop"),
                    )
                    .clicked()
                {
                    self.daemon.send(DaemonCommand::StopStream);
                }

                if ui.button("Refresh").clicked() {
                    self.daemon.send(DaemonCommand::Refresh);
                }
            });

            ui.add_space(16.0);

            ui.group(|ui| {
                ui.label("PIN");
                let pin_text = self.status.pin.as_deref().unwrap_or("----");
                ui.label(RichText::new(pin_text).size(32.0).strong());
            });

            ui.add_space(16.0);

            ui.group(|ui| {
                ui.label("Pairing QR");
                if let Some(texture) = &self.qr_texture {
                    let size = texture.size_vec2();
                    ui.image((texture.id(), size));
                } else {
                    ui.label("No QR payload from daemon.");
                }
            });
        });

        ctx.request_repaint_after(Duration::from_millis(200));
    }
}

struct DaemonClient {
    socket_path: PathBuf,
    event_tx: Sender<DaemonEvent>,
    last_status_poll: Instant,
    last_connect_attempt: Instant,
    last_spawn_attempt: Instant,
    last_spawn_success: Option<Instant>,
    spawn_suppressed_until: Option<Instant>,
    status: DaemonStatus,
    writer: Option<UnixStream>,
    reader: Option<BufReader<UnixStream>>,
    spawned_child: Option<Child>,
    control_bind: String,
    warning_sent: bool,
}

impl DaemonClient {
    fn new(socket_path: PathBuf, event_tx: Sender<DaemonEvent>) -> Self {
        Self {
            socket_path,
            event_tx,
            last_status_poll: Instant::now() - STATUS_POLL_INTERVAL,
            last_connect_attempt: Instant::now() - RECONNECT_INTERVAL,
            last_spawn_attempt: Instant::now() - SPAWN_INTERVAL,
            last_spawn_success: None,
            spawn_suppressed_until: None,
            status: DaemonStatus::default(),
            writer: None,
            reader: None,
            spawned_child: None,
            control_bind: std::env::var("PRLX_CONTROL_BIND")
                .unwrap_or_else(|_| "0.0.0.0:0".to_string()),
            warning_sent: false,
        }
    }

    fn run(&mut self, command_rx: Receiver<DaemonCommand>) {
        loop {
            match command_rx.try_recv() {
                Ok(command) => {
                    if matches!(command, DaemonCommand::Shutdown) {
                        self.handle_command(command);
                        break;
                    }
                    self.handle_command(command);
                }
                Err(TryRecvError::Disconnected) => {
                    break;
                }
                Err(TryRecvError::Empty) => {}
            }

            if self.writer.is_none() {
                self.ensure_connected();
            }

            self.refresh_child_status();
            self.poll_status();
            self.read_responses();

            std::thread::sleep(Duration::from_millis(80));
        }

        self.shutdown_child();
    }

    fn ensure_connected(&mut self) {
        if self.last_connect_attempt.elapsed() < RECONNECT_INTERVAL {
            return;
        }
        self.last_connect_attempt = Instant::now();

        match UnixStream::connect(&self.socket_path) {
            Ok(stream) => {
                self.warning_sent = false;
                self.spawn_suppressed_until = None;
                let _ = stream.set_read_timeout(Some(Duration::from_millis(200)));
                let _ = stream.set_write_timeout(Some(Duration::from_millis(200)));
                match stream.try_clone() {
                    Ok(reader_stream) => {
                        self.reader = Some(BufReader::new(reader_stream));
                        self.writer = Some(stream);
                        let _ = self.event_tx.send(DaemonEvent::Status(self.status.clone()));
                    }
                    Err(err) => {
                        let _ = self
                            .event_tx
                            .send(DaemonEvent::Error(format!("Failed to clone socket: {err}")));
                    }
                }
            }
            Err(err) => {
                self.ensure_daemon_spawn();
                if err.kind() != std::io::ErrorKind::NotFound || self.socket_path.exists() {
                    let _ = self
                        .event_tx
                        .send(DaemonEvent::Error(format!("Failed to connect: {err}")));
                }
            }
        }
    }

    fn ensure_daemon_spawn(&mut self) {
        if self.spawned_child.is_some() {
            return;
        }
        if let Some(suppressed_until) = self.spawn_suppressed_until {
            if suppressed_until > Instant::now() {
                return;
            }
            self.spawn_suppressed_until = None;
        }
        if self.last_spawn_attempt.elapsed() < SPAWN_INTERVAL {
            return;
        }
        if let Some(last_success) = self.last_spawn_success {
            if last_success.elapsed() < SPAWN_RETRY_DELAY {
                return;
            }
        }
        self.last_spawn_attempt = Instant::now();

        if self.attach_existing_socket() {
            self.spawn_suppressed_until = Some(Instant::now() + EXTERNAL_DAEMON_RETRY_DELAY);
            return;
        }

        if self.control_port_in_use() {
            self.spawn_suppressed_until = Some(Instant::now() + EXTERNAL_DAEMON_RETRY_DELAY);
            if !self.warning_sent {
                let _ = self.event_tx.send(DaemonEvent::Warning(
                    "Host daemon already running".to_string(),
                ));
                self.warning_sent = true;
            }
            return;
        }

        let mut spawn_errors = Vec::new();

        match self.spawn_hostd("prlx-hostd") {
            Ok(child) => {
                self.last_spawn_success = Some(Instant::now());
                self.spawned_child = Some(child);
                self.warning_sent = false;
                return;
            }
            Err(err) => {
                spawn_errors.push(format!("prlx-hostd: {err}"));
            }
        }

        if let Some(path) = discover_local_hostd() {
            match self.spawn_hostd(path) {
                Ok(child) => {
                    self.last_spawn_success = Some(Instant::now());
                    self.spawned_child = Some(child);
                    self.warning_sent = false;
                    return;
                }
                Err(err) => {
                    spawn_errors.push(format!("local prlx-hostd: {err}"));
                }
            }
        }

        if !spawn_errors.is_empty() {
            let _ = self.event_tx.send(DaemonEvent::Error(format!(
                "Failed to start daemon: {}",
                spawn_errors.join("; ")
            )));
        }
    }

    fn spawn_hostd<P: AsRef<std::ffi::OsStr>>(&self, path: P) -> std::io::Result<Child> {
        Command::new(path)
            .env("PRLX_SOCKET_PATH", &self.socket_path)
            .arg("--control-bind")
            .arg(&self.control_bind)
            .spawn()
    }

    fn attach_existing_socket(&mut self) -> bool {
        if !self.socket_path.exists() {
            return false;
        }

        let stream = match UnixStream::connect(&self.socket_path) {
            Ok(stream) => stream,
            Err(_) => return false,
        };

        let _ = stream.set_read_timeout(Some(Duration::from_millis(200)));
        let _ = stream.set_write_timeout(Some(Duration::from_millis(200)));
        let reader_stream = match stream.try_clone() {
            Ok(reader_stream) => reader_stream,
            Err(err) => {
                let _ = self
                    .event_tx
                    .send(DaemonEvent::Error(format!("Failed to clone socket: {err}")));
                return false;
            }
        };

        let mut reader = BufReader::new(reader_stream);
        if stream.try_clone().unwrap().write_all(b"status\n").is_err() {
            return false;
        }

        let mut buffer = String::new();
        match reader.read_line(&mut buffer) {
            Ok(0) => return false,
            Ok(_) => {
                let Some(status) = parse_status(&buffer) else {
                    return false;
                };
                self.status = status.clone();
                self.writer = Some(stream);
                self.reader = Some(reader);
                self.warning_sent = false;
                let _ = self.event_tx.send(DaemonEvent::Status(status));
                true
            }
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => false,
            Err(err) if err.kind() == std::io::ErrorKind::TimedOut => false,
            Err(_) => false,
        }
    }

    fn control_port_in_use(&mut self) -> bool {
        let Some((_, port)) = self.control_bind.rsplit_once(':') else {
            return false;
        };
        if port == "0" {
            return false;
        }

        match TcpListener::bind(&self.control_bind) {
            Ok(listener) => {
                drop(listener);
                false
            }
            Err(err) if err.kind() == std::io::ErrorKind::AddrInUse => true,
            Err(err) => {
                let _ = self.event_tx.send(DaemonEvent::Error(format!(
                    "Failed to check control port {}: {err}",
                    self.control_bind
                )));
                false
            }
        }
    }

    fn poll_status(&mut self) {
        if self.writer.is_none() {
            return;
        }
        if self.last_status_poll.elapsed() < STATUS_POLL_INTERVAL {
            return;
        }
        self.last_status_poll = Instant::now();
        self.send_line("status");
    }

    fn read_responses(&mut self) {
        let Some(reader) = self.reader.as_mut() else {
            return;
        };

        let mut buffer = String::new();
        match reader.read_line(&mut buffer) {
            Ok(0) => {
                self.writer = None;
                self.reader = None;
            }
            Ok(_) => {
                if let Some(status) = parse_status(&buffer) {
                    self.status = status.clone();
                    let _ = self.event_tx.send(DaemonEvent::Status(status));
                }
            }
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(err) if err.kind() == std::io::ErrorKind::TimedOut => {}
            Err(err) => {
                self.writer = None;
                self.reader = None;
                let _ = self
                    .event_tx
                    .send(DaemonEvent::Error(format!("Socket error: {err}")));
            }
        }
    }

    fn handle_command(&mut self, command: DaemonCommand) {
        match command {
            DaemonCommand::Refresh => self.send_line("status"),
            DaemonCommand::StartStream => self.send_line("start"),
            DaemonCommand::StopStream => {
                self.send_line("stop");
                self.shutdown_child();
            }
            DaemonCommand::Shutdown => {
                self.send_line("stop");
                self.shutdown_child();
            }
        }
    }

    fn send_line(&mut self, line: &str) {
        let Some(writer) = self.writer.as_mut() else {
            return;
        };

        let payload = format!("{line}\n");
        if writer.write_all(payload.as_bytes()).is_err() {
            self.writer = None;
            self.reader = None;
        }
    }

    fn refresh_child_status(&mut self) {
        let Some(child) = self.spawned_child.as_mut() else {
            return;
        };
        if let Ok(Some(_)) = child.try_wait() {
            self.spawned_child = None;
        }
    }

    fn shutdown_child(&mut self) {
        let Some(mut child) = self.spawned_child.take() else {
            return;
        };
        let _ = child.kill();
        let _ = child.wait();
        self.writer = None;
        self.reader = None;
    }
}

fn parse_status(line: &str) -> Option<DaemonStatus> {
    let mut status = DaemonStatus::default();
    let tokens: Vec<&str> = line.split_whitespace().collect();

    for token in tokens {
        if let Some((key, value)) = token.split_once('=') {
            match key {
                "state" => {
                    status.state = match value {
                        "idle" => UiState::Idle,
                        "waiting" => UiState::Waiting,
                        "connected" => UiState::Connected,
                        "streaming" => UiState::Streaming,
                        _ => status.state,
                    }
                }
                "pin" => {
                    status.pin = Some(value.to_string());
                }
                "qr" => {
                    status.qr_uri = Some(value.to_string());
                }
                _ => {}
            }
        }
    }

    Some(status)
}

fn qr_to_image(payload: &str, scale: usize) -> Option<ColorImage> {
    let code = QrCode::new(payload.as_bytes()).ok()?;
    let width = code.width();
    let image_size = width * scale;
    let mut pixels = vec![Color32::WHITE; image_size * image_size];

    for y in 0..width {
        for x in 0..width {
            let color = if code[(x, y)] == qrcode::Color::Dark {
                Color32::BLACK
            } else {
                Color32::WHITE
            };
            for dy in 0..scale {
                for dx in 0..scale {
                    let idx = (y * scale + dy) * image_size + (x * scale + dx);
                    pixels[idx] = color;
                }
            }
        }
    }

    Some(ColorImage {
        size: [image_size, image_size],
        pixels,
    })
}

fn expand_path(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return Path::new(&home).join(rest);
        }
    }
    PathBuf::from(path)
}

fn discover_local_hostd() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    let candidate = dir.join("prlx-hostd");
    if candidate.exists() {
        return Some(candidate);
    }
    None
}
