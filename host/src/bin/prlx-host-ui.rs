// src/bin/prlx-host-ui.rs
//
// Full adjusted version (layout + spacing + purple theme):
// - All UI text in English
// - Fixes vertical badge stacking by making badges horizontal + limiting wrap
// - Adds outer padding around entire window + spacing between cards
// - Keeps older-egui compatibility (no egui::Shadow / Frame::shadow)
// - Keeps daemon debug logging + socket protocol
// - Avoids double Ctrl-C handler registration (ONLY uses shutdown channel handler in main())

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
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

// Layout constants (tune here)
const OUTER_MARGIN: f32 = 16.0;
const CARD_GAP: f32 = 16.0;
const SECTION_GAP: f32 = 14.0;

fn main() -> eframe::Result<()> {
    let socket_path = expand_path(DEFAULT_SOCKET_PATH);
    loggins::info(
        "ui",
        format!("Starting Parallax Host UI; socket_path={socket_path:?}"),
    );

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1020.0, 760.0]),
        ..Default::default()
    };

    // Keep ONLY this Ctrl-C handler (no second handler anywhere else).
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
    visuals_applied: bool,
}

impl HostUiApp {
    fn new(
        cc: &eframe::CreationContext<'_>,
        socket_path: PathBuf,
        shutdown_rx: Receiver<()>,
    ) -> Self {
        loggins::info("ui", "HostUiApp::new");

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
            visuals_applied: false,
        };

        app.apply_visuals_once(&cc.egui_ctx);
        app.refresh_qr_texture(&cc.egui_ctx);
        app.daemon.send(DaemonCommand::Refresh);
        app
    }

    fn apply_visuals_once(&mut self, ctx: &egui::Context) {
        if self.visuals_applied {
            return;
        }
        self.visuals_applied = true;

        let palette = UiPalette::new();

        // Older-egui safe visuals (no Shadow types).
        let mut visuals = egui::Visuals::light();

        // Window/panel fill prevents the “black corners” look when using rounding.
        // If your egui version doesn't have these fields, comment them out.
        visuals.window_fill = palette.background;
        visuals.panel_fill = palette.background;

        // Rounding
        visuals.window_rounding = egui::Rounding::same(16.0);
        visuals.menu_rounding = egui::Rounding::same(12.0);

        // Widget rounding
        visuals.widgets.inactive.rounding = egui::Rounding::same(12.0);
        visuals.widgets.hovered.rounding = egui::Rounding::same(12.0);
        visuals.widgets.active.rounding = egui::Rounding::same(12.0);
        visuals.widgets.noninteractive.rounding = egui::Rounding::same(12.0);

        // Strokes
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, palette.card_border);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(195, 202, 214));
        visuals.widgets.active.bg_stroke = Stroke::new(1.0, palette.accent);

        ctx.set_visuals(visuals);
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
        self.apply_visuals_once(ctx);

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

        let palette = UiPalette::new();
        let time = ctx.input(|i| i.time) as f32;
        let pulse = (time * 1.0).sin() * 0.5 + 0.5;
        let accent = lerp_color(palette.accent, palette.accent_glow, pulse);

        // ---------- HEADER ----------
        egui::TopBottomPanel::top("header")
            .frame(header_frame(&palette))
            .show(ctx, |ui| {
                // Make header tall enough + padded
                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    // Left title stack
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new("Parallax Host")
                                .size(28.0)
                                .strong()
                                .color(palette.text),
                        );
                        ui.label(
                            RichText::new("Local streaming host control panel")
                                .size(13.0)
                                .color(palette.subtle_text),
                        );
                    });

                    // Right badges (force horizontal, prevent wrap weirdness)
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.spacing_mut().item_spacing.x = 10.0;

                        badge(
                            ui,
                            "Daemon",
                            daemon_label(self.status.connected),
                            state_color(self.status.connected, &palette),
                            &palette,
                        );

                        badge(ui, "State", self.status.state.label(), accent, &palette);
                    });
                });

                ui.add_space(2.0);
            });

        // ---------- MAIN BODY ----------
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(palette.background))
            .show(ctx, |ui| {
                // Outer padding around everything
                ui.add_space(OUTER_MARGIN);

                // Use a fixed two-column layout that doesn't compress text weirdly
                ui.columns(2, |columns| {
                    let (left_slice, right_slice) = columns.split_at_mut(1);
                    let left = &mut left_slice[0];
                    let right = &mut right_slice[0];

                    left.spacing_mut().item_spacing = egui::vec2(0.0, CARD_GAP);
                    right.spacing_mut().item_spacing = egui::vec2(0.0, CARD_GAP);

                    // LEFT COLUMN
                    card_frame(&palette, palette.card).show(left, |ui| {
                        section_header(ui, "Session", &palette);
                        ui.add_space(SECTION_GAP);

                        info_row(ui, "State", self.status.state.label(), &palette);
                        info_row(ui, "Daemon", daemon_label(self.status.connected), &palette);
                        info_row(ui, "Socket", "Local IPC (Unix socket)", &palette);

                        ui.add_space(SECTION_GAP);

                        if self.status.state == UiState::Streaming {
                            ui.add(
                                egui::ProgressBar::new(1.0)
                                    .desired_width(f32::INFINITY)
                                    .fill(accent)
                                    .text("Streaming"),
                            );
                        } else if self.status.connected {
                            ui.add(
                                egui::ProgressBar::new(0.70)
                                    .desired_width(f32::INFINITY)
                                    .fill(lerp_color(palette.accent, palette.accent_glow, 0.35))
                                    .text("Ready"),
                            );
                        } else {
                            ui.add(
                                egui::ProgressBar::new(0.30)
                                    .desired_width(f32::INFINITY)
                                    .fill(palette.muted)
                                    .text("Connecting…"),
                            );
                        }
                    });

                    card_frame(&palette, palette.card).show(left, |ui| {
                        section_header(ui, "Controls", &palette);
                        ui.add_space(SECTION_GAP);

                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 10.0;

                            let start = ui.add_enabled(
                                self.status.state != UiState::Streaming,
                                primary_button("▶ Start", &palette),
                            );
                            if start.clicked() {
                                loggins::info("ui", "Start clicked");
                                self.daemon.send(DaemonCommand::StartStream);
                            }

                            let stop = ui.add_enabled(
                                self.status.state == UiState::Streaming,
                                secondary_button("■ Stop", &palette),
                            );
                            if stop.clicked() {
                                loggins::info("ui", "Stop clicked");
                                self.daemon.send(DaemonCommand::StopStream);
                            }

                            if ui.add(ghost_button("↻ Refresh", &palette)).clicked() {
                                loggins::info("ui", "Refresh clicked");
                                self.daemon.send(DaemonCommand::Refresh);
                            }
                        });

                        ui.add_space(SECTION_GAP);
                        ui.label(
                            RichText::new(
                                "Tip: Keep Parallax Host running to allow pairing new clients.",
                            )
                            .size(13.0)
                            .color(palette.subtle_text),
                        );
                    });

                    if let Some(err) = &self.last_error {
                        card_frame(&palette, palette.error_bg).show(left, |ui| {
                            section_header_colored(ui, "Daemon Error", palette.error);
                            ui.add_space(8.0);
                            ui.label(RichText::new(err).size(13.0).color(palette.text));
                        });
                    }

                    if let Some(warning) = &self.last_warning {
                        card_frame(&palette, palette.warning_bg).show(left, |ui| {
                            section_header_colored(ui, "Warning", palette.warning);
                            ui.add_space(8.0);
                            ui.label(RichText::new(warning).size(13.0).color(palette.text));
                        });
                    }

                    // RIGHT COLUMN
                    card_frame(&palette, palette.card).show(right, |ui| {
                        section_header(ui, "Secure Pairing", &palette);
                        ui.add_space(SECTION_GAP);

                        ui.label(
                            RichText::new("Access PIN")
                                .size(13.0)
                                .color(palette.subtle_text),
                        );
                        ui.add_space(6.0);

                        let pin_text = self.status.pin.as_deref().unwrap_or("----");
                        ui.label(
                            RichText::new(pin_text)
                                .size(40.0)
                                .font(FontId::proportional(40.0))
                                .strong()
                                .color(palette.text),
                        );

                        ui.add_space(SECTION_GAP);

                        ui.label(
                            RichText::new("QR Code")
                                .size(13.0)
                                .color(palette.subtle_text),
                        );
                        ui.add_space(8.0);

                        let qr_frame = egui::Frame::none()
                            .fill(palette.qr_bg)
                            .rounding(egui::Rounding::same(18.0))
                            .stroke(Stroke::new(1.0, palette.card_border))
                            .inner_margin(egui::Margin::same(14.0));

                        qr_frame.show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                if let Some(texture) = &self.qr_texture {
                                    // Clamp QR size so it doesn’t explode or compress layout
                                    let mut size = texture.size_vec2();
                                    let max_side = 220.0;
                                    let scale = (max_side / size.x).min(max_side / size.y).min(1.0);
                                    size *= scale;
                                    ui.image((texture.id(), size));
                                } else {
                                    ui.label(
                                        RichText::new("No QR payload from daemon.")
                                            .size(13.0)
                                            .color(palette.subtle_text),
                                    );
                                }
                            });
                        });

                        ui.add_space(SECTION_GAP);
                        ui.label(
                            RichText::new(
                                "Open the client app and scan the QR to connect securely.",
                            )
                            .size(13.0)
                            .color(palette.subtle_text),
                        );
                    });
                });

                ui.add_space(OUTER_MARGIN);
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

// --------------------------
// Daemon client (Unix socket)
// --------------------------

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
                    if matches!(command, DaemonCommand::Shutdown) {
                        self.handle_command(command);
                        break;
                    }
                    self.handle_command(command);
                }
                Err(TryRecvError::Disconnected) => break,
                Err(TryRecvError::Empty) => {}
            }

            if self.writer.is_none() {
                self.ensure_connected();
            }

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

        match UnixStream::connect(&self.socket_path) {
            Ok(stream) => {
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
                        let _ = self
                            .event_tx
                            .send(DaemonEvent::Error(format!("Failed to clone socket: {err}")));
                    }
                }
            }
            Err(err) => {
                self.ensure_daemon_spawn();
                if err.kind() != std::io::ErrorKind::NotFound || self.socket_path.exists() {
                    if !self.warning_sent {
                        let msg = format!("Failed to connect: {err}");
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

        if let Ok(child) = Command::new("prlx-hostd")
            .arg("--control-bind")
            .arg("0.0.0.0:0")
            .env("PRLX_SOCKET_PATH", &self.socket_path)
            .spawn()
        {
            CHILD_PID.store(child.id() as i32, Ordering::Relaxed);
            self.last_spawn_success = Some(Instant::now());
            self.spawned_child = Some(child);
            return;
        }

        if let Some(path) = discover_local_hostd() {
            if let Ok(child) = Command::new(path)
                .arg("--control-bind")
                .arg("0.0.0.0:0")
                .env("PRLX_SOCKET_PATH", &self.socket_path)
                .spawn()
            {
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
        self.send_line("status", "poll status");
    }

    fn read_responses_drain(&mut self) {
        let Some(reader) = self.reader.as_mut() else {
            return;
        };

        loop {
            let mut buffer = String::new();
            match reader.read_line(&mut buffer) {
                Ok(0) => {
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
                    if let Some(mut status) = parse_status(&raw) {
                        status.connected = true;
                        self.status = status.clone();
                        let _ = self.event_tx.send(DaemonEvent::Status(status));
                    } else {
                        let _ = self.event_tx.send(DaemonEvent::Warning(raw));
                    }
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(err) if err.kind() == std::io::ErrorKind::TimedOut => break,
                Err(err) => {
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
                self.ensure_connected();
                if self.writer.is_some() {
                    self.send_line("status", "refresh");
                } else {
                    self.pending_command = Some(command);
                }
            }
            DaemonCommand::StartStream => {
                self.ensure_connected();
                if self.writer.is_some() {
                    self.send_line("start", "start");
                } else {
                    self.pending_command = Some(command);
                }
            }
            DaemonCommand::StopStream => {
                self.ensure_connected();
                if self.writer.is_some() {
                    self.send_line("stop", "stop");
                }
                self.shutdown_child();
            }
            DaemonCommand::Shutdown => {
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

        match command {
            DaemonCommand::Refresh => self.send_line("status", "refresh"),
            DaemonCommand::StartStream => self.send_line("start", "start"),
            DaemonCommand::StopStream => self.send_line("stop", "stop"),
            DaemonCommand::Shutdown => self.send_line("stop", "shutdown"),
        };
    }

    fn send_line(&mut self, line: &str, action: &str) -> bool {
        let Some(writer) = self.writer.as_mut() else {
            let _ = self.event_tx.send(DaemonEvent::Error(format!(
                "Daemon not connected; cannot {action}"
            )));
            return false;
        };

        let payload = format!("{line}\n");
        if writer.write_all(payload.as_bytes()).is_err() {
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
        if let Ok(Some(_)) = child.try_wait() {
            self.spawned_child = None;
            CHILD_PID.store(0, Ordering::Relaxed);
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
        CHILD_PID.store(0, Ordering::Relaxed);
    }
}

// --------------------------
// Protocol helpers
// --------------------------

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

// --------------------------
// UI helpers
// --------------------------

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
        // Purple theme
        Self {
            background: Color32::from_rgb(245, 246, 250),
            header: Color32::from_rgb(252, 252, 254),
            card: Color32::from_rgb(255, 255, 255),
            card_border: Color32::from_rgb(223, 227, 236),
            qr_bg: Color32::from_rgb(247, 248, 252),
            text: Color32::from_rgb(18, 22, 29),
            subtle_text: Color32::from_rgb(104, 112, 125),

            // Purple accent (not blue)
            accent: Color32::from_rgb(124, 77, 255), // #7C4DFF
            accent_glow: Color32::from_rgb(176, 145, 255), // softer glow

            muted: Color32::from_rgb(210, 214, 222),
            error: Color32::from_rgb(201, 61, 72),
            error_bg: Color32::from_rgb(255, 238, 240),
            warning: Color32::from_rgb(214, 131, 0),
            warning_bg: Color32::from_rgb(255, 246, 230),
        }
    }
}

fn header_frame(palette: &UiPalette) -> egui::Frame {
    // More breathing room at the top area + consistent padding
    egui::Frame::none()
        .fill(palette.header)
        .stroke(Stroke::new(1.0, palette.card_border))
        .inner_margin(egui::Margin::symmetric(OUTER_MARGIN, 14.0))
}

fn card_frame(palette: &UiPalette, fill: Color32) -> egui::Frame {
    egui::Frame::none()
        .fill(fill)
        .rounding(egui::Rounding::same(18.0))
        .stroke(Stroke::new(1.0, palette.card_border))
        .inner_margin(egui::Margin::same(18.0))
}

fn section_header(ui: &mut egui::Ui, title: &str, palette: &UiPalette) {
    ui.label(RichText::new(title).size(16.0).strong().color(palette.text));
}

fn section_header_colored(ui: &mut egui::Ui, title: &str, color: Color32) {
    ui.label(RichText::new(title).size(16.0).strong().color(color));
}

fn info_row(ui: &mut egui::Ui, label: &str, value: &str, palette: &UiPalette) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).size(13.0).color(palette.subtle_text));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            // Prevent weird wrapping by keeping it single-line when possible
            ui.label(RichText::new(value).size(13.0).color(palette.text));
        });
    });
}

fn badge(ui: &mut egui::Ui, label: &str, value: &str, color: Color32, palette: &UiPalette) {
    ui.vertical(|ui| {
        ui.label(RichText::new(label).size(12.0).color(palette.subtle_text));

        // Horizontal pill (prevents the vertical-stacking mess)
        let pill = egui::Frame::none()
            .fill(color)
            .rounding(egui::Rounding::same(999.0))
            .inner_margin(egui::Margin::symmetric(12.0, 6.0));

        pill.show(ui, |ui| {
            ui.set_min_width(110.0); // helps keep it horizontal
            ui.label(RichText::new(value).size(13.0).color(Color32::WHITE));
        });
    });
}

fn state_color(connected: bool, palette: &UiPalette) -> Color32 {
    if connected {
        palette.accent
    } else {
        Color32::from_rgb(140, 146, 156)
    }
}

fn daemon_label(connected: bool) -> &'static str {
    if connected {
        "Connected"
    } else {
        "Connecting…"
    }
}

fn primary_button<'a>(label: &'a str, palette: &'a UiPalette) -> egui::Button<'a> {
    egui::Button::new(RichText::new(label).size(14.0).color(Color32::WHITE))
        .fill(palette.accent)
        .min_size(egui::vec2(120.0, 38.0))
}

fn secondary_button<'a>(label: &'a str, palette: &'a UiPalette) -> egui::Button<'a> {
    egui::Button::new(RichText::new(label).size(14.0).color(palette.text))
        .fill(Color32::from_rgb(233, 236, 244))
        .min_size(egui::vec2(110.0, 38.0))
}

fn ghost_button<'a>(label: &'a str, palette: &'a UiPalette) -> egui::Button<'a> {
    egui::Button::new(RichText::new(label).size(14.0).color(palette.text))
        .stroke(Stroke::new(1.0, palette.card_border))
        .min_size(egui::vec2(110.0, 38.0))
}

fn lerp_color(from: Color32, to: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let r = from.r() as f32 + (to.r() as f32 - from.r() as f32) * t;
    let g = from.g() as f32 + (to.g() as f32 - from.g() as f32) * t;
    let b = from.b() as f32 + (to.b() as f32 - from.b() as f32) * t;
    Color32::from_rgb(r as u8, g as u8, b as u8)
}

// --------------------------
// Path helpers
// --------------------------

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
