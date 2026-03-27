use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::time::{Duration, Instant};

use host::core::logging as loggins;

const STATUS_POLL_INTERVAL: Duration = Duration::from_secs(2);
const RECONNECT_INTERVAL: Duration = Duration::from_secs(3);
const SPAWN_INTERVAL: Duration = Duration::from_secs(5);
const SPAWN_RETRY_DELAY: Duration = Duration::from_secs(10);

static CHILD_PID: AtomicI32 = AtomicI32::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UiState {
    Idle,
    Waiting,
    Connected,
    Streaming,
}

impl UiState {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Idle => "Idle",
            Self::Waiting => "Waiting",
            Self::Connected => "Connected",
            Self::Streaming => "Streaming",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct DaemonStatus {
    pub(crate) state: UiState,
    pub(crate) connected: bool,
    pub(crate) pin: Option<String>,
    pub(crate) qr_uri: Option<String>,
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
pub(crate) enum DaemonEvent {
    Status(DaemonStatus),
    Error(String),
    Warning(String),
}

#[derive(Debug)]
pub(crate) enum DaemonCommand {
    Refresh,
    StartStream,
    StopStream,
    Shutdown,
}

pub(crate) struct DaemonHandle {
    command_tx: Sender<DaemonCommand>,
    event_rx: Receiver<DaemonEvent>,
}

impl DaemonHandle {
    pub(crate) fn new(socket_path: PathBuf) -> Self {
        let (command_tx, command_rx) = std::sync::mpsc::channel();
        let (event_tx, event_rx) = std::sync::mpsc::channel();

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

    pub(crate) fn send(&self, command: DaemonCommand) {
        loggins::debug("daemon_handle", format!("send command: {command:?}"));
        let _ = self.command_tx.send(command);
    }

    pub(crate) fn try_recv(&self) -> Option<DaemonEvent> {
        self.event_rx.try_recv().ok()
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
                if err.kind() == std::io::ErrorKind::ConnectionRefused && self.socket_path.exists()
                {
                    // A stale socket file can survive crashes; remove it so the daemon can re-bind.
                    let _ = std::fs::remove_file(&self.socket_path);
                }
                self.ensure_daemon_spawn();
                if err.kind() != std::io::ErrorKind::NotFound || self.socket_path.exists() {
                    if !self.warning_sent {
                        let msg = format!("Waiting for daemon socket: {err}");
                        let _ = self.event_tx.send(DaemonEvent::Warning(msg));
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
            } else {
                let _ = self.event_tx.send(DaemonEvent::Error(
                    "Failed to start prlx-hostd from local install path".to_string(),
                ));
            }
        } else {
            let _ = self.event_tx.send(DaemonEvent::Error(
                "prlx-hostd binary not found in PATH or alongside prlx-host-ui".to_string(),
            ));
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

fn discover_local_hostd() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    let candidate = dir.join("prlx-hostd");
    if candidate.exists() {
        return Some(candidate);
    }
    None
}
