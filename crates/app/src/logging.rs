//! Logging to a file in the data folder (and to stderr in debug builds).
//!
//! Passwords never reach the log: command lines are redacted in
//! `sevenzip_core::runner` before they are logged.

use std::fs::{File, OpenOptions};
use std::sync::Mutex;

use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::writer::MakeWriterExt;

/// A log file is started afresh once it grows past this size.
const MAX_LOG_BYTES: u64 = 5 * 1024 * 1024;

/// Keeps logging alive; drop it at exit.
#[derive(Debug)]
pub struct Guard;

/// Starts logging. `RUST_LOG` overrides the default level (`info`).
pub fn init() -> Guard {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("info,wgpu_core=warn,wgpu_hal=warn,naga=warn,iced_wgpu=warn,cosmic_text=warn")
    });
    let file = log_file().map(Mutex::new);
    let builder = tracing_subscriber::fmt().with_env_filter(filter).with_ansi(false).with_target(false);
    let result = match file {
        Some(file) if cfg!(debug_assertions) => builder.with_writer(file.and(std::io::stderr)).try_init(),
        Some(file) => builder.with_writer(file).try_init(),
        None => builder.with_writer(std::io::stderr).try_init(),
    };
    if let Err(error) = result {
        eprintln!("logging is unavailable: {error}");
    }
    Guard
}

fn log_file() -> Option<File> {
    let dir = crate::settings::paths::log_dir()?;
    std::fs::create_dir_all(&dir).ok()?;
    let path = dir.join("7zip-gui.log");
    let too_big = std::fs::metadata(&path).is_ok_and(|m| m.len() > MAX_LOG_BYTES);
    if too_big {
        let _ = std::fs::rename(&path, dir.join("7zip-gui.old.log"));
    }
    OpenOptions::new().create(true).append(true).open(path).ok()
}
