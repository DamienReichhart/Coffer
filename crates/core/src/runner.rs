//! Running 7-Zip and streaming what it does.
//!
//! [`run`] returns a stream of [`Event`]s ending with [`Event::Finished`].
//! Dropping the stream kills 7-Zip, which is how operations are cancelled.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};

use futures_core::Stream;
use tokio::io::AsyncReadExt;

use crate::command::{self, Args, Dialect};
use crate::error::{Error, Result, classify};
use crate::listing::Listing;
use crate::locate::SevenZip;
use crate::progress::{Parser, Signal};

/// What a running operation reports.
#[derive(Debug, Clone)]
pub enum Event {
    /// Overall progress, and the file being processed when known.
    Progress { percent: u8, file: Option<String> },
    /// A line of 7-Zip's output.
    Log(String),
    /// The end of the operation. Always the last event.
    Finished(std::result::Result<Outcome, std::sync::Arc<Error>>),
}

/// A successful run.
#[derive(Debug, Clone)]
pub struct Outcome {
    /// 7-Zip's standard output, one line per line, without the progress redraws.
    pub output: String,
    /// 7-Zip finished with warnings (exit code 1): some files were skipped.
    pub warnings: bool,
    pub elapsed: Duration,
}

/// Hides the console window 7-Zip would otherwise flash on Windows.
pub(crate) fn hide_console(command: &mut tokio::process::Command) {
    #[cfg(windows)]
    {
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(not(windows))]
    let _ = command;
}

fn spawn(seven: &SevenZip, args: &Args) -> Result<tokio::process::Child> {
    let mut command = tokio::process::Command::new(&seven.path);
    command.args(args).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).kill_on_drop(true);
    hide_console(&mut command);
    tracing::debug!(exe = %seven.path.display(), args = ?redact(args), "running 7-Zip");
    command.spawn().map_err(Error::Spawn)
}

/// The arguments with any password replaced, for logs.
pub fn redact(args: &Args) -> Vec<String> {
    args.iter()
        .map(|a| {
            let a = a.to_string_lossy();
            if a.starts_with("-p") && a.len() > 2 { "-p***".to_owned() } else { a.into_owned() }
        })
        .collect()
}

/// Runs 7-Zip with `args`, streaming progress and log lines.
///
/// `archive` only improves error messages.
pub fn run(seven: SevenZip, args: Args, archive: Option<PathBuf>) -> impl Stream<Item = Event> + Send + 'static {
    async_stream::stream! {
        let started = Instant::now();
        let mut child = match spawn(&seven, &args) {
            Ok(child) => child,
            Err(error) => {
                yield Event::Finished(Err(error.into()));
                return;
            }
        };
        let (Some(mut stdout), Some(mut stderr)) = (child.stdout.take(), child.stderr.take()) else {
            yield Event::Finished(Err(Error::Failed("no output pipes".into()).into()));
            return;
        };

        let mut parser = Parser::new();
        let mut output = String::new();
        let mut errors = Vec::new();
        let mut out_buf = [0_u8; 8192];
        let mut err_buf = [0_u8; 4096];
        let (mut out_open, mut err_open) = (true, true);
        let mut last_percent = None;

        while out_open || err_open {
            tokio::select! {
                read = stdout.read(&mut out_buf), if out_open => match read {
                    Ok(0) | Err(_) => out_open = false,
                    Ok(n) => {
                        for signal in parser.feed(&out_buf[..n]) {
                            match signal {
                                Signal::Progress { percent, file } => {
                                    if last_percent != Some(percent) || file.is_some() {
                                        last_percent = Some(percent);
                                        yield Event::Progress { percent, file };
                                    }
                                }
                                Signal::Line(line) => {
                                    output.push_str(&line);
                                    output.push('\n');
                                    yield Event::Log(line.trim().to_owned());
                                }
                            }
                        }
                    }
                },
                read = stderr.read(&mut err_buf), if err_open => match read {
                    Ok(0) | Err(_) => err_open = false,
                    Ok(n) => errors.extend_from_slice(&err_buf[..n]),
                },
            }
        }
        for signal in parser.finish() {
            if let Signal::Line(line) = signal {
                output.push_str(&line);
                output.push('\n');
                yield Event::Log(line.trim().to_owned());
            }
        }
        let errors = String::from_utf8_lossy(&errors).into_owned();
        for line in errors.lines().map(str::trim).filter(|l| !l.is_empty()) {
            yield Event::Log(line.to_owned());
        }

        let status = match child.wait().await {
            Ok(status) => status,
            Err(error) => {
                yield Event::Finished(Err(Error::Io(error).into()));
                return;
            }
        };
        let elapsed = started.elapsed();
        let code = status.code();
        tracing::debug!(?code, ?elapsed, "7-Zip finished");
        let result = match code {
            Some(0) => Ok(Outcome { output, warnings: false, elapsed }),
            Some(1) => Ok(Outcome { output, warnings: true, elapsed }),
            _ => {
                let all = format!("{output}\n{errors}");
                Err(classify(code, &all, archive.as_deref()).into())
            }
        };
        yield Event::Finished(result);
    }
}

/// Runs 7-Zip to completion and returns its standard output.
async fn capture(seven: &SevenZip, args: Args, archive: Option<&Path>) -> Result<String> {
    let child = spawn(seven, &args)?;
    let output = child.wait_with_output().await?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    match output.status.code() {
        Some(0 | 1) => Ok(stdout),
        code => {
            let all = format!("{stdout}\n{}", String::from_utf8_lossy(&output.stderr));
            Err(classify(code, &all, archive))
        }
    }
}

/// Lists the content of an archive.
pub async fn list(seven: &SevenZip, archive: &Path, password: Option<&str>) -> Result<Listing> {
    let args = command::list(Dialect::of(seven), archive, password);
    let output = capture(seven, args, Some(archive)).await?;
    let mut listing = Listing::parse(&output)?;
    listing.info.path = archive.to_path_buf();
    Ok(listing)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passwords_never_reach_the_logs() {
        let args: Args = vec!["x".into(), "-psecret".into(), "-p".into(), "a.7z".into()];
        assert_eq!(redact(&args), ["x", "-p***", "-p", "a.7z"]);
    }
}
