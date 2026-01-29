use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

pub fn debug(tag: &str, message: impl AsRef<str>) {
    log(LogLevel::Debug, tag, message.as_ref());
}

pub fn info(tag: &str, message: impl AsRef<str>) {
    log(LogLevel::Info, tag, message.as_ref());
}

pub fn warn(tag: &str, message: impl AsRef<str>) {
    log(LogLevel::Warn, tag, message.as_ref());
}

pub fn error(tag: &str, message: impl AsRef<str>) {
    log(LogLevel::Error, tag, message.as_ref());
}

fn log(level: LogLevel, tag: &str, message: &str) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs_f64())
        .unwrap_or_default();
    let level_label = match level {
        LogLevel::Debug => "DEBUG",
        LogLevel::Info => "INFO",
        LogLevel::Warn => "WARN",
        LogLevel::Error => "ERROR",
    };
    let line = format!("[{timestamp:.3}] [{level_label}] [{tag}] {message}");
    match level {
        LogLevel::Warn | LogLevel::Error => eprintln!("{line}"),
        LogLevel::Debug | LogLevel::Info => println!("{line}"),
    }
}
