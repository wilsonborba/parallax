use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::Once;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::time::{Duration, Instant};

use eframe::egui::{
    self, Align, Color32, ColorImage, FontId, Layout, RichText, Stroke, TextureHandle,
};
use qrcode::QrCode;

use host::core::logging as loggins;

const DEFAULT_SOCKET_PATH: &str = "~/.local/share/prlx/prlx.sock";
const STATUS_POLL_INTERVAL: Duration = Duration::from_secs(2);
const RECONNECT_INTERVAL: Duration = Duration::from_secs(3);
const SPAWN_INTERVAL: Duration = Duration::from_secs(5);
const SPAWN_RETRY_DELAY: Duration = Duration::from_secs(10);

static CHILD_PID: AtomicI32 = AtomicI32::new(0);
static CTRL_C_HANDLER: Once = Once::new();

fn main() -> eframe::Result<()> {
    let socket_path = expand_path(DEFAULT_SOCKET_PATH);
    loggins::info(
        "ui",
        format!("Starting Parallax Host UI; socket_path={socket_path:?}"),
    );

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([980.0, 720.0]),
        ..Default::default()
    };
    let (shutdown_tx, shutdown_rx) = mpsc::channel();

    let handler_tx = shutdown_tx.clone();
    if let Err(err) = ctrlc::set_handler(move || {
        let _ = handler_tx.send(());
    }) {
        loggins::error("ui", format!("Failed to install signal handler: {err}"));
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
    connected: bool,
    pin: Option<String>,
    qr_uri: Option<String>,
}

impl Default for DaemonStatus {
    fn default() -> Self {
        Self {
            state: UiState::Idle,
            connected: false,
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

        loggins::debug(
            "daemon_handle",
            format!("Spawning daemon client thread; socket_path={socket_path:?}"),
        );
        std::thread::spawn(move || {
            let mut client = DaemonClient::new(socket_path, event_tx);
            client.run(command_rx);
            loggins::debug("daemon_client", "Client thread exited");
        });

        Self {
            command_tx,
            event_rx,
        }
    }

    fn send(&self, command: DaemonCommand) {
        loggins::debug("daemon_handle", format!("send command: {command:?}"));
        let _ = self.command_tx.send(command);
    }

    fn try_recv(&self) -> Option<DaemonEvent> {
        self.event_rx.try_recv().ok()
    }

    fn command_sender(&self) -> Sender<DaemonCommand> {
        self.command_tx.clone()
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
        loggins::info("ui", "HostUiApp::new");
        let daemon = DaemonHandle::new(socket_path);
        install_ctrlc_handler(daemon.command_sender());

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
        apply_visuals(&cc.egui_ctx);
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

        loggins::debug(
            "ui",
            format!("Refreshing QR texture; payload_len={}", payload.len()),
        );
        if let Some(image) = qr_to_image(&payload, 4) {
            self.qr_texture =
                Some(ctx.load_texture("pairing_qr", image, egui::TextureOptions::NEAREST));
            self.qr_payload = Some(payload);
        } else {
            loggins::warn("ui", "Failed to generate QR image");
        }
    }
}

impl eframe::App for HostUiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        apply_visuals(ctx);
        // Shutdown handling
        if !self.shutdown_initiated {
            match self.shutdown_rx.try_recv() {
                Ok(()) => {
                    loggins::info("ui", "Shutdown requested (signal)");
                    self.shutdown_initiated = true;
                    self.daemon.send(DaemonCommand::Shutdown);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    loggins::warn("ui", "shutdown_rx disconnected");
                    self.shutdown_initiated = true;
                }
            }
        }

        // Drain daemon events
        while let Some(event) = self.daemon.try_recv() {
            match event {
                DaemonEvent::Status(status) => {
                    loggins::debug(
                        "ui",
                        format!(
                            "Daemon status update: state={:?} connected={} pin={} qr={}",
                            status.state,
                            status.connected,
                            status.pin.as_deref().unwrap_or("None"),
                            status.qr_uri.as_deref().unwrap_or("None")
                        ),
                    );
                    self.status = status;
                    self.last_warning = None;
                }
                DaemonEvent::Error(err) => {
                    loggins::error("ui", format!("Daemon error event: {err}"));
                    self.last_error = Some(err);
                }
                DaemonEvent::Warning(warning) => {
                    loggins::warn("ui", format!("Daemon warning event: {warning}"));
                    self.last_warning = Some(warning);
                }
            }
        }

        self.refresh_qr_texture(ctx);

        let palette = UiPalette::new();
        let time = ctx.input(|i| i.time) as f32;
        let pulse = (time * 1.4).sin() * 0.5 + 0.5;
        let accent = lerp_color(palette.accent, palette.accent_glow, pulse);
        let state_color = state_color(self.status.state, &palette);

        egui::TopBottomPanel::top("header")
            .frame(card_frame(&palette, palette.header))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new("Parallax Host")
                                .size(28.0)
                                .strong()
                                .color(palette.text),
                        );
                        ui.label(
                            RichText::new("Centro de controle do streaming local")
                                .size(14.0)
                                .color(palette.subtle_text),
                        );
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        status_pill(
                            ui,
                            "Daemon",
                            daemon_label(self.status.connected),
                            state_color,
                        );
                        ui.add_space(12.0);
                        status_pill(ui, "Estado", self.status.state.label(), accent);
                    });
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(palette.background))
            .show(ctx, |ui| {
                ui.add_space(12.0);
                ui.columns(2, |columns| {
                    let left = &mut columns[0];
                    let right = &mut columns[1];

                    card_frame(&palette, palette.card).show(left, |ui| {
                        section_header(ui, "Status da Sessão");
                        ui.add_space(4.0);
                        info_row(ui, "Estado", self.status.state.label());
                        info_row(ui, "Daemon", daemon_label(self.status.connected));
                        info_row(ui, "Socket", "Monitorando localmente");
                        ui.add_space(12.0);
                        if self.status.state == UiState::Streaming {
                            ui.add(
                                egui::ProgressBar::new(1.0)
                                    .desired_width(f32::INFINITY)
                                    .fill(accent)
                                    .text("Transmitindo agora"),
                            );
                        } else {
                            ui.add(
                                egui::ProgressBar::new(0.4)
                                    .desired_width(f32::INFINITY)
                                    .fill(palette.muted)
                                    .text("Aguardando ação"),
                            );
                        }
                    });

                    left.add_space(16.0);

                    card_frame(&palette, palette.card).show(left, |ui| {
                        section_header(ui, "Controles Rápidos");
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            let start = ui.add_enabled(
                                self.status.state != UiState::Streaming,
                                primary_button("▶ Iniciar"),
                            );
                            if start.clicked() {
                                loggins::info("ui", "Start clicked");
                                self.daemon.send(DaemonCommand::StartStream);
                            }

                            let stop = ui.add_enabled(
                                self.status.state == UiState::Streaming,
                                secondary_button("■ Parar"),
                            );
                            if stop.clicked() {
                                loggins::info("ui", "Stop clicked");
                                self.daemon.send(DaemonCommand::StopStream);
                            }

                            if ui.add(ghost_button("↻ Atualizar")).clicked() {
                                loggins::info("ui", "Refresh clicked");
                                self.daemon.send(DaemonCommand::Refresh);
                            }
                        });
                        ui.add_space(12.0);
                        ui.label(
                            RichText::new("Dica: mantenha o Parallax aberto para parear novos clientes.")
                                .size(13.0)
                                .color(palette.subtle_text),
                        );
                    });

                    if let Some(err) = &self.last_error {
                        left.add_space(16.0);
                        card_frame(&palette, palette.error_bg).show(left, |ui| {
                            section_header_colored(ui, "Erro do daemon", palette.error);
                            ui.label(
                                RichText::new(err)
                                    .size(13.0)
                                    .color(palette.text),
                            );
                        });
                    }

                    if let Some(warning) = &self.last_warning {
                        left.add_space(16.0);
                        card_frame(&palette, palette.warning_bg).show(left, |ui| {
                            section_header_colored(ui, "Aviso", palette.warning);
                            ui.label(
                                RichText::new(warning)
                                    .size(13.0)
                                    .color(palette.text),
                            );
                        });
                    }

                    card_frame(&palette, palette.card).show(right, |ui| {
                        section_header(ui, "Pareamento seguro");
                        ui.add_space(8.0);
                        let pin_text = self.status.pin.as_deref().unwrap_or("----");
                        ui.label(
                            RichText::new("PIN de acesso")
                                .size(13.0)
                                .color(palette.subtle_text),
                        );
                        ui.label(
                            RichText::new(pin_text)
                                .size(36.0)
                                .font(FontId::proportional(36.0))
                                .strong()
                                .color(palette.text),
                        );
                        ui.add_space(12.0);
                        ui.label(
                            RichText::new("QR Code")
                                .size(13.0)
                                .color(palette.subtle_text),
                        );
                        ui.add_space(6.0);
                        let qr_frame = egui::Frame::none()
                            .fill(palette.qr_bg)
                            .rounding(egui::Rounding::same(18.0))
                            .stroke(Stroke::new(1.0, palette.card_border))
                            .inner_margin(egui::Margin::same(12.0));
                        qr_frame.show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                if let Some(texture) = &self.qr_texture {
                                    let size = texture.size_vec2();
                                    ui.image((texture.id(), size));
                                } else {
                                    ui.label(
                                        RichText::new("Nenhum QR disponível.")
                                            .size(13.0)
                                            .color(palette.subtle_text),
                                    );
                                }
                            });
                        });
                        ui.add_space(12.0);
                        ui.label(
                            RichText::new(
                                "Abra o app cliente e escaneie o QR para iniciar a transmissão com segurança.",
                            )
                            .size(13.0)
                            .color(palette.subtle_text),
                        );
                    });
                });
            });

        ctx.request_repaint_after(Duration::from_millis(200));
    }
}

impl Drop for HostUiApp {
    fn drop(&mut self) {
        loggins::info("ui", "HostUiApp::drop -> sending Shutdown");
        self.daemon.send(DaemonCommand::Shutdown);
    }
}

struct DaemonClient {
    socket_path: PathBuf,
    event_tx: Sender<DaemonEvent>,

    last_status_poll: Instant,
    last_connect_attempt: Instant,
    last_spawn_attempt: Instant,
    last_spawn_success: Option<Instant>,

    warning_sent: bool,
    status: DaemonStatus,

    writer: Option<UnixStream>,
    reader: Option<BufReader<UnixStream>>,
    spawned_child: Option<Child>,
    pending_command: Option<DaemonCommand>,
}

impl DaemonClient {
    fn new(socket_path: PathBuf, event_tx: Sender<DaemonEvent>) -> Self {
        loggins::info("daemon_client", format!("new; socket_path={socket_path:?}"));
        Self {
            socket_path,
            event_tx,
            last_status_poll: Instant::now() - STATUS_POLL_INTERVAL,
            last_connect_attempt: Instant::now() - RECONNECT_INTERVAL,
            last_spawn_attempt: Instant::now() - SPAWN_INTERVAL,
            last_spawn_success: None,
            warning_sent: false,
            status: DaemonStatus::default(),
            writer: None,
            reader: None,
            spawned_child: None,
            pending_command: None,
        }
    }

    fn run(&mut self, command_rx: Receiver<DaemonCommand>) {
        loggins::info("daemon_client", "run loop start");

        loop {
            match command_rx.try_recv() {
                Ok(command) => {
                    loggins::debug("daemon_client", format!("received command: {command:?}"));
                    if matches!(command, DaemonCommand::Shutdown) {
                        self.handle_command(command);
                        break;
                    }
                    self.handle_command(command);
                }
                Err(TryRecvError::Disconnected) => {
                    loggins::warn("daemon_client", "command_rx disconnected; exiting");
                    break;
                }
                Err(TryRecvError::Empty) => {}
            }

            if self.writer.is_none() {
                self.ensure_connected();
            }

            // NOTE: This "connected" is socket-level; useful for UI visibility.
            let socket_connected = self.writer.is_some();
            if socket_connected && !self.status.connected {
                self.status.connected = true;
                let _ = self.event_tx.send(DaemonEvent::Status(self.status.clone()));
            } else if !socket_connected && self.status.connected {
                self.status.connected = false;
                let _ = self.event_tx.send(DaemonEvent::Status(self.status.clone()));
            }

            self.flush_pending_command();
            self.refresh_child_status();
            self.poll_status();
            self.read_responses_drain();

            std::thread::sleep(Duration::from_millis(80));
        }

        self.shutdown_child();
        loggins::info("daemon_client", "run loop end");
    }

    fn ensure_connected(&mut self) {
        if self.last_connect_attempt.elapsed() < RECONNECT_INTERVAL {
            return;
        }
        self.last_connect_attempt = Instant::now();

        loggins::debug(
            "daemon_client",
            format!("Attempting connect to {:?}", self.socket_path),
        );
        match UnixStream::connect(&self.socket_path) {
            Ok(stream) => {
                loggins::info("daemon_client", "UnixStream connected");
                self.warning_sent = false;

                let _ = stream.set_read_timeout(Some(Duration::from_millis(200)));
                let _ = stream.set_write_timeout(Some(Duration::from_millis(200)));

                match stream.try_clone() {
                    Ok(reader_stream) => {
                        self.reader = Some(BufReader::new(reader_stream));
                        self.writer = Some(stream);
                        self.warning_sent = false;
                        self.status.connected = true;
                        let _ = self.event_tx.send(DaemonEvent::Status(self.status.clone()));
                    }
                    Err(err) => {
                        loggins::error("daemon_client", format!("Failed to clone socket: {err}"));
                        let _ = self
                            .event_tx
                            .send(DaemonEvent::Error(format!("Failed to clone socket: {err}")));
                    }
                }
            }
            Err(err) => {
                loggins::debug("daemon_client", format!("Connect failed: {err}"));
                self.ensure_daemon_spawn();

                if err.kind() != std::io::ErrorKind::NotFound || self.socket_path.exists() {
                    if !self.warning_sent {
                        let msg = format!("Failed to connect: {err}");
                        loggins::warn("daemon_client", &msg);
                        let _ = self.event_tx.send(DaemonEvent::Error(msg));
                        self.warning_sent = true;
                    }
                }
            }
        }
    }

    fn ensure_daemon_spawn(&mut self) {
        if self.spawned_child.is_some() {
            return;
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

        loggins::info("daemon_spawn", "Attempting spawn prlx-hostd from PATH");
        if let Ok(child) = Command::new("prlx-hostd")
            .arg("--control-bind")
            .arg("0.0.0.0:0")
            .env("PRLX_SOCKET_PATH", &self.socket_path)
            .spawn()
        {
            loggins::info(
                "daemon_spawn",
                format!("Spawned prlx-hostd pid={}", child.id()),
            );
            CHILD_PID.store(child.id() as i32, Ordering::Relaxed);
            self.last_spawn_success = Some(Instant::now());
            self.spawned_child = Some(child);
            return;
        }

        loggins::warn(
            "daemon_spawn",
            "PATH spawn failed; attempting discover_local_hostd()",
        );
        if let Some(path) = discover_local_hostd() {
            loggins::info("daemon_spawn", format!("Attempting spawn from {:?}", path));
            if let Ok(child) = Command::new(path)
                .arg("--control-bind")
                .arg("0.0.0.0:0")
                .env("PRLX_SOCKET_PATH", &self.socket_path)
                .spawn()
            {
                loggins::info(
                    "daemon_spawn",
                    format!("Spawned local prlx-hostd pid={}", child.id()),
                );
                CHILD_PID.store(child.id() as i32, Ordering::Relaxed);
                self.last_spawn_success = Some(Instant::now());
                self.spawned_child = Some(child);
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
        loggins::debug("daemon_client", "poll_status -> sending 'status'");
        self.send_line("status", "poll status");
    }

    /// IMPORTANT: drains all available lines so we don't "lag behind" and look stuck.
    fn read_responses_drain(&mut self) {
        let Some(reader) = self.reader.as_mut() else {
            return;
        };

        loop {
            let mut buffer = String::new();
            match reader.read_line(&mut buffer) {
                Ok(0) => {
                    loggins::warn("daemon_client", "daemon disconnected (EOF)");
                    self.writer = None;
                    self.reader = None;
                    self.status.connected = false;
                    let _ = self.event_tx.send(DaemonEvent::Status(self.status.clone()));
                    break;
                }
                Ok(_) => {
                    let raw = buffer.trim_end().to_string();
                    if raw.is_empty() {
                        continue;
                    }

                    // If it's a status line, apply; otherwise surface as warning for visibility.
                    if let Some(mut status) = parse_status(&raw) {
                        status.connected = true;
                        loggins::debug("daemon_proto", format!("RX status: {raw}"));
                        self.status = status.clone();
                        let _ = self.event_tx.send(DaemonEvent::Status(status));
                    } else {
                        loggins::debug("daemon_proto", format!("RX non-status: {raw}"));
                        let _ = self.event_tx.send(DaemonEvent::Warning(raw));
                    }
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(err) if err.kind() == std::io::ErrorKind::TimedOut => break,
                Err(err) => {
                    loggins::error("daemon_client", format!("Socket read error: {err}"));
                    self.writer = None;
                    self.reader = None;
                    self.status.connected = false;
                    let _ = self.event_tx.send(DaemonEvent::Status(self.status.clone()));
                    let _ = self
                        .event_tx
                        .send(DaemonEvent::Error(format!("Socket error: {err}")));
                    break;
                }
            }
        }
    }

    fn handle_command(&mut self, command: DaemonCommand) {
        match command {
            DaemonCommand::Refresh => {
                loggins::info("daemon_client", "Refresh command");
                self.ensure_connected();
                if self.writer.is_some() {
                    self.send_line("status", "refresh");
                } else {
                    self.pending_command = Some(command);
                }
            }
            DaemonCommand::StartStream => {
                loggins::info("daemon_client", "StartStream command");
                self.ensure_connected();
                if self.writer.is_some() {
                    self.send_line("start", "start");
                } else {
                    self.pending_command = Some(command);
                }
            }
            DaemonCommand::StopStream => {
                loggins::info("daemon_client", "StopStream command");
                self.ensure_connected();
                if self.writer.is_some() {
                    self.send_line("stop", "stop");
                }
                self.shutdown_child();
            }
            DaemonCommand::Shutdown => {
                loggins::info("daemon_client", "Shutdown command");
                self.ensure_connected();
                if self.writer.is_some() {
                    self.send_line("stop", "shutdown");
                }
                self.shutdown_child();
            }
        }
    }

    fn flush_pending_command(&mut self) {
        if self.writer.is_none() {
            return;
        }
        let Some(command) = self.pending_command.take() else {
            return;
        };

        loggins::debug(
            "daemon_client",
            format!("Flushing pending command: {command:?}"),
        );
        match command {
            DaemonCommand::Refresh => {
                self.send_line("status", "refresh");
            }
            DaemonCommand::StartStream => {
                self.send_line("start", "start");
            }
            DaemonCommand::StopStream => {
                self.send_line("stop", "stop");
            }
            DaemonCommand::Shutdown => {
                self.send_line("stop", "shutdown");
            }
        }
    }

    fn send_line(&mut self, line: &str, action: &str) -> bool {
        let Some(writer) = self.writer.as_mut() else {
            let msg = format!("Daemon not connected; cannot {action}");
            loggins::warn("daemon_proto", &msg);
            let _ = self.event_tx.send(DaemonEvent::Error(msg));
            return false;
        };

        let payload = format!("{line}\n");
        loggins::debug(
            "daemon_proto",
            format!("TX {action}: {:?}", payload.trim_end()),
        );

        if let Err(err) = writer.write_all(payload.as_bytes()) {
            loggins::error("daemon_proto", format!("write_all failed: {err}"));
            self.writer = None;
            self.reader = None;
            self.status.connected = false;
            let _ = self.event_tx.send(DaemonEvent::Status(self.status.clone()));
            let _ = self.event_tx.send(DaemonEvent::Error(format!(
                "Failed to send {action} command"
            )));
            return false;
        }
        true
    }

    fn refresh_child_status(&mut self) {
        let Some(child) = self.spawned_child.as_mut() else {
            return;
        };

        if let Ok(Some(exit_status)) = child.try_wait() {
            loggins::warn("daemon_spawn", format!("Child exited: {exit_status}"));
            self.spawned_child = None;
            CHILD_PID.store(0, Ordering::Relaxed);
        }
    }

    fn shutdown_child(&mut self) {
        let Some(mut child) = self.spawned_child.take() else {
            return;
        };

        loggins::info(
            "daemon_spawn",
            format!("Shutting down child pid={}", child.id()),
        );
        let _ = child.kill();
        let _ = child.wait();

        self.writer = None;
        self.reader = None;
        CHILD_PID.store(0, Ordering::Relaxed);
    }
}

fn install_ctrlc_handler(command_tx: Sender<DaemonCommand>) {
    CTRL_C_HANDLER.call_once(|| {
        loggins::debug("ui", "Installing Ctrl-C handler (once)");
        if let Err(err) = ctrlc::set_handler(move || {
            loggins::info("ui", "Ctrl-C -> Shutdown");
            let _ = command_tx.send(DaemonCommand::Shutdown);
            let pid = CHILD_PID.load(Ordering::Relaxed);
            if pid > 0 {
                unsafe {
                    libc::kill(pid, libc::SIGTERM);
                }
            }
        }) {
            loggins::error("ui", format!("Failed to install Ctrl-C handler: {err}"));
        }
    });
}

/// Returns Some(status) only if line contained a recognized key.
/// This prevents random log lines from resetting UI to defaults.
fn parse_status(line: &str) -> Option<DaemonStatus> {
    let mut status = DaemonStatus::default();
    let mut saw_any = false;

    for token in line.split_whitespace() {
        if let Some((key, value)) = token.split_once('=') {
            match key {
                "state" => {
                    saw_any = true;
                    status.state = match value {
                        "idle" => UiState::Idle,
                        "waiting" => UiState::Waiting,
                        "connected" => UiState::Connected,
                        "streaming" => UiState::Streaming,
                        _ => status.state,
                    }
                }
                "pin" => {
                    saw_any = true;
                    status.pin = Some(value.to_string());
                }
                "qr" => {
                    saw_any = true;
                    status.qr_uri = Some(value.to_string());
                }
                _ => {}
            }
        }
    }

    saw_any.then_some(status)
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

struct UiPalette {
    background: Color32,
    header: Color32,
    card: Color32,
    card_border: Color32,
    qr_bg: Color32,
    text: Color32,
    subtle_text: Color32,
    accent: Color32,
    accent_glow: Color32,
    muted: Color32,
    error: Color32,
    error_bg: Color32,
    warning: Color32,
    warning_bg: Color32,
}

impl UiPalette {
    fn new() -> Self {
        Self {
            background: Color32::from_rgb(244, 246, 249),
            header: Color32::from_rgb(250, 251, 252),
            card: Color32::from_rgb(255, 255, 255),
            card_border: Color32::from_rgb(223, 227, 234),
            qr_bg: Color32::from_rgb(247, 248, 251),
            text: Color32::from_rgb(18, 22, 29),
            subtle_text: Color32::from_rgb(104, 112, 125),
            accent: Color32::from_rgb(0, 122, 255),
            accent_glow: Color32::from_rgb(88, 168, 255),
            muted: Color32::from_rgb(210, 214, 221),
            error: Color32::from_rgb(201, 61, 72),
            error_bg: Color32::from_rgb(255, 238, 240),
            warning: Color32::from_rgb(214, 131, 0),
            warning_bg: Color32::from_rgb(255, 246, 230),
        }
    }
}

fn apply_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::light();
    visuals.window_rounding = egui::Rounding::same(18.0);
    visuals.menu_rounding = egui::Rounding::same(12.0);
    visuals.widgets.inactive.rounding = egui::Rounding::same(12.0);
    visuals.widgets.hovered.rounding = egui::Rounding::same(12.0);
    visuals.widgets.active.rounding = egui::Rounding::same(12.0);
    visuals.widgets.noninteractive.rounding = egui::Rounding::same(12.0);
    visuals.window_shadow = egui::Shadow {
        extrusion: 18.0,
        color: Color32::from_black_alpha(20),
    };
    visuals.popup_shadow = egui::Shadow {
        extrusion: 12.0,
        color: Color32::from_black_alpha(25),
    };
    ctx.set_visuals(visuals);
}

fn card_frame(palette: &UiPalette, fill: Color32) -> egui::Frame {
    egui::Frame::none()
        .fill(fill)
        .rounding(egui::Rounding::same(18.0))
        .stroke(Stroke::new(1.0, palette.card_border))
        .shadow(egui::Shadow {
            extrusion: 16.0,
            color: Color32::from_black_alpha(22),
        })
        .inner_margin(egui::Margin::same(16.0))
}

fn state_color(state: UiState, palette: &UiPalette) -> Color32 {
    match state {
        UiState::Idle => palette.muted,
        UiState::Waiting => Color32::from_rgb(120, 136, 255),
        UiState::Connected => Color32::from_rgb(46, 166, 82),
        UiState::Streaming => palette.accent,
    }
}

fn daemon_label(connected: bool) -> &'static str {
    if connected {
        "Conectado"
    } else {
        "Conectando..."
    }
}

fn status_pill(ui: &mut egui::Ui, label: &str, value: &str, color: Color32) {
    let pill_frame = egui::Frame::none()
        .fill(color)
        .rounding(egui::Rounding::same(999.0))
        .inner_margin(egui::Margin::symmetric(12.0, 6.0));
    ui.vertical(|ui| {
        ui.label(
            RichText::new(label)
                .size(12.0)
                .color(Color32::from_rgb(110, 118, 132)),
        );
        pill_frame.show(ui, |ui| {
            ui.label(RichText::new(value).size(13.0).color(Color32::WHITE));
        });
    });
}

fn section_header(ui: &mut egui::Ui, title: &str) {
    ui.label(
        RichText::new(title)
            .size(16.0)
            .strong()
            .color(Color32::from_rgb(24, 28, 36)),
    );
}

fn section_header_colored(ui: &mut egui::Ui, title: &str, color: Color32) {
    ui.label(RichText::new(title).size(16.0).strong().color(color));
}

fn info_row(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(label)
                .size(13.0)
                .color(Color32::from_rgb(96, 104, 118)),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(
                RichText::new(value)
                    .size(13.0)
                    .color(Color32::from_rgb(24, 28, 36)),
            );
        });
    });
}

fn primary_button(label: &str) -> egui::Button {
    egui::Button::new(RichText::new(label).size(14.0).color(Color32::WHITE))
        .fill(Color32::from_rgb(0, 122, 255))
        .min_size(egui::vec2(120.0, 38.0))
}

fn secondary_button(label: &str) -> egui::Button {
    egui::Button::new(
        RichText::new(label)
            .size(14.0)
            .color(Color32::from_rgb(32, 36, 44)),
    )
    .fill(Color32::from_rgb(233, 236, 242))
    .min_size(egui::vec2(110.0, 38.0))
}

fn ghost_button(label: &str) -> egui::Button {
    egui::Button::new(
        RichText::new(label)
            .size(14.0)
            .color(Color32::from_rgb(32, 36, 44)),
    )
    .stroke(Stroke::new(1.0, Color32::from_rgb(210, 214, 221)))
    .min_size(egui::vec2(110.0, 38.0))
}

fn lerp_color(from: Color32, to: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let r = from.r() as f32 + (to.r() as f32 - from.r() as f32) * t;
    let g = from.g() as f32 + (to.g() as f32 - from.g() as f32) * t;
    let b = from.b() as f32 + (to.b() as f32 - from.b() as f32) * t;
    Color32::from_rgb(r as u8, g as u8, b as u8)
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
