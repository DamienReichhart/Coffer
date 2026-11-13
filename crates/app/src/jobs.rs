//! The operations 7-Zip runs for the user, and their live state.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use coffer_core::command::{self, Args, Dialect};
use coffer_core::{CompressOptions, Error, ExtractOptions, HashAlgorithm, Outcome};

use crate::icons::Icon;

pub type JobId = u64;

/// Keep this many log lines per job; enough to diagnose, bounded in memory.
const LOG_LINES: usize = 400;

/// What a job does. Kept after it runs so a failed job can be retried, for
/// instance with a password.
#[derive(Debug, Clone, PartialEq)]
pub enum Spec {
    Extract {
        archive: PathBuf,
        options: ExtractOptions,
        open_when_done: bool,
    },
    Compress {
        archive: PathBuf,
        inputs: Vec<PathBuf>,
        options: CompressOptions,
    },
    Test {
        archive: PathBuf,
        password: Option<String>,
    },
    Delete {
        archive: PathBuf,
        entries: Vec<String>,
        password: Option<String>,
    },
    Rename {
        archive: PathBuf,
        from: String,
        to: String,
        password: Option<String>,
    },
    Hash {
        files: Vec<PathBuf>,
        algorithms: Vec<HashAlgorithm>,
    },
    /// Extract one file to a temporary folder and open it.
    Preview {
        archive: PathBuf,
        entry: String,
        password: Option<String>,
        dir: PathBuf,
    },
}

impl Spec {
    pub fn args(&self, dialect: Dialect) -> Args {
        match self {
            Self::Extract { archive, options, .. } => command::extract(dialect, archive, options),
            Self::Compress { archive, inputs, options } => command::add(dialect, archive, inputs, options),
            Self::Test { archive, password } => command::test(dialect, archive, password.as_deref()),
            Self::Delete { archive, entries, password } => {
                command::delete(dialect, archive, entries, password.as_deref())
            }
            Self::Rename { archive, from, to, password } => {
                command::rename(dialect, archive, from, to, password.as_deref())
            }
            Self::Hash { files, algorithms } => command::hash(dialect, files, algorithms),
            Self::Preview { archive, entry, password, dir } => command::extract(
                dialect,
                archive,
                &ExtractOptions {
                    destination: dir.clone(),
                    overwrite: coffer_core::OverwriteMode::Overwrite,
                    keep_paths: false,
                    password: password.clone(),
                    only: vec![entry.clone()],
                },
            ),
        }
    }

    /// The archive this job reads or writes.
    pub fn archive(&self) -> Option<&Path> {
        match self {
            Self::Extract { archive, .. }
            | Self::Compress { archive, .. }
            | Self::Test { archive, .. }
            | Self::Delete { archive, .. }
            | Self::Rename { archive, .. }
            | Self::Preview { archive, .. } => Some(archive),
            Self::Hash { .. } => None,
        }
    }

    /// Whether the job changes the archive, so an open view of it is stale.
    pub fn modifies_archive(&self) -> bool {
        matches!(self, Self::Compress { .. } | Self::Delete { .. } | Self::Rename { .. })
    }

    /// The same job with a password, for a retry after "password required".
    #[must_use]
    pub fn with_password(mut self, value: String) -> Self {
        match &mut self {
            Self::Extract { options, .. } => options.password = Some(value),
            Self::Test { password, .. }
            | Self::Delete { password, .. }
            | Self::Rename { password, .. }
            | Self::Preview { password, .. } => *password = Some(value),
            Self::Compress { .. } | Self::Hash { .. } => {}
        }
        self
    }

    pub fn icon(&self) -> Icon {
        match self {
            Self::Extract { .. } => Icon::FolderOutput,
            Self::Compress { .. } => Icon::PackagePlus,
            Self::Test { .. } => Icon::ShieldCheck,
            Self::Delete { .. } => Icon::Trash2,
            Self::Rename { .. } => Icon::Pencil,
            Self::Hash { .. } => Icon::Hash,
            Self::Preview { .. } => Icon::Eye,
        }
    }

    /// A short description, e.g. "Extracting photos.7z".
    pub fn title(&self) -> String {
        let name =
            |p: &Path| p.file_name().map_or_else(|| p.display().to_string(), |n| n.to_string_lossy().into_owned());
        match self {
            Self::Extract { archive, .. } => format!("Extract {}", name(archive)),
            Self::Compress { archive, .. } => format!("Create {}", name(archive)),
            Self::Test { archive, .. } => format!("Test {}", name(archive)),
            Self::Delete { archive, entries, .. } => {
                format!("Delete {} from {}", plural(entries.len(), "item"), name(archive))
            }
            Self::Rename { from, to, .. } => format!("Rename {from} to {to}"),
            Self::Hash { files, .. } => format!("Checksums of {}", plural(files.len(), "item")),
            Self::Preview { entry, .. } => format!("Open {}", entry.rsplit('/').next().unwrap_or(entry)),
        }
    }

    /// Where the result lands, shown under the title.
    pub fn detail(&self) -> String {
        match self {
            Self::Extract { options, .. } => format!("to {}", options.destination.display()),
            Self::Compress { archive, inputs, .. } => {
                format!(
                    "{} into {}",
                    plural(inputs.len(), "item"),
                    archive.parent().map(|p| p.display().to_string()).unwrap_or_default()
                )
            }
            Self::Test { archive, .. } | Self::Delete { archive, .. } | Self::Rename { archive, .. } => {
                archive.display().to_string()
            }
            Self::Hash { algorithms, .. } => algorithms.iter().map(ToString::to_string).collect::<Vec<_>>().join(", "),
            Self::Preview { archive, .. } => format!("from {}", archive.display()),
        }
    }

    /// The folder to reveal when the job is done.
    pub fn output_folder(&self) -> Option<PathBuf> {
        match self {
            Self::Extract { options, .. } => Some(options.destination.clone()),
            Self::Compress { archive, .. } => archive.parent().map(Path::to_path_buf),
            _ => None,
        }
    }
}

pub fn plural(n: usize, word: &str) -> String {
    if n == 1 { format!("1 {word}") } else { format!("{n} {word}s") }
}

/// Where a job is.
#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Running,
    Succeeded { warnings: bool },
    Failed(AppError),
    Cancelled,
}

/// A clonable, displayable error for the interface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppError {
    pub message: String,
    pub needs_password: bool,
    pub wrong_password: bool,
}

impl From<&Error> for AppError {
    fn from(error: &Error) -> Self {
        Self {
            message: error.to_string(),
            needs_password: error.needs_password(),
            wrong_password: matches!(error, Error::WrongPassword),
        }
    }
}

impl AppError {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into(), needs_password: false, wrong_password: false }
    }
}

/// A job and its live state.
#[derive(Debug)]
pub struct Job {
    pub id: JobId,
    pub spec: Spec,
    pub status: Status,
    pub percent: u8,
    pub current: Option<String>,
    pub started: Instant,
    pub elapsed: Option<Duration>,
    pub log: VecDeque<String>,
    pub output: Option<String>,
    pub handle: Option<iced::task::Handle>,
    pub show_log: bool,
}

impl Job {
    pub fn new(id: JobId, spec: Spec) -> Self {
        Self {
            id,
            spec,
            status: Status::Running,
            percent: 0,
            current: None,
            started: Instant::now(),
            elapsed: None,
            log: VecDeque::new(),
            output: None,
            handle: None,
            show_log: false,
        }
    }

    pub fn is_running(&self) -> bool {
        self.status == Status::Running
    }

    pub fn elapsed(&self) -> Duration {
        self.elapsed.unwrap_or_else(|| self.started.elapsed())
    }

    pub fn push_log(&mut self, line: String) {
        if self.log.len() == LOG_LINES {
            self.log.pop_front();
        }
        self.log.push_back(line);
    }

    /// Records the end of the job.
    pub fn finish(&mut self, result: &Result<Outcome, std::sync::Arc<Error>>) {
        self.handle = None;
        self.elapsed = Some(self.started.elapsed());
        match result {
            Ok(outcome) => {
                self.percent = 100;
                self.current = None;
                self.output = Some(outcome.output.clone());
                self.status = Status::Succeeded { warnings: outcome.warnings };
            }
            Err(error) if matches!(**error, Error::Cancelled) => self.status = Status::Cancelled,
            Err(error) => self.status = Status::Failed(AppError::from(error.as_ref())),
        }
    }

    pub fn cancel(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
        if self.is_running() {
            self.status = Status::Cancelled;
            self.elapsed = Some(self.started.elapsed());
        }
    }
}

/// Every job of this session, newest first.
#[derive(Debug, Default)]
pub struct Jobs {
    next: JobId,
    pub list: Vec<Job>,
}

impl Jobs {
    pub fn add(&mut self, spec: Spec) -> JobId {
        self.next += 1;
        self.list.insert(0, Job::new(self.next, spec));
        self.next
    }

    pub fn get_mut(&mut self, id: JobId) -> Option<&mut Job> {
        self.list.iter_mut().find(|j| j.id == id)
    }

    pub fn get(&self, id: JobId) -> Option<&Job> {
        self.list.iter().find(|j| j.id == id)
    }

    pub fn running(&self) -> usize {
        self.list.iter().filter(|j| j.is_running()).count()
    }

    pub fn clear_finished(&mut self) {
        self.list.retain(Job::is_running);
    }

    pub fn cancel_all(&mut self) {
        for job in &mut self.list {
            job.cancel();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retrying_with_a_password() {
        let spec = Spec::Test { archive: "a.7z".into(), password: None };
        assert_eq!(spec.with_password("pw".into()), Spec::Test { archive: "a.7z".into(), password: Some("pw".into()) });
    }

    #[test]
    fn titles_and_plurals() {
        let spec = Spec::Delete { archive: "/x/a.zip".into(), entries: vec!["a".into(), "b".into()], password: None };
        assert_eq!(spec.title(), "Delete 2 items from a.zip");
        assert!(spec.modifies_archive());
        assert_eq!(plural(1, "file"), "1 file");
    }

    #[test]
    fn jobs_are_newest_first_and_cleared() {
        let mut jobs = Jobs::default();
        let a = jobs.add(Spec::Test { archive: "a".into(), password: None });
        let b = jobs.add(Spec::Test { archive: "b".into(), password: None });
        assert_eq!(jobs.list[0].id, b);
        jobs.get_mut(a).expect("job").status = Status::Cancelled;
        jobs.clear_finished();
        assert_eq!(jobs.list.len(), 1);
        assert_eq!(jobs.running(), 1);
    }
}
