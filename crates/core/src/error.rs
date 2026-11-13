//! Errors, and the translation of 7-Zip's console messages into them.

use std::path::PathBuf;

/// A shorthand for results in this crate.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Everything that can go wrong while finding or running 7-Zip.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// No 7-Zip executable was found on this computer.
    #[error("7-Zip was not found on this computer")]
    NotInstalled,

    /// A path was configured but does not point at a working 7-Zip.
    #[error("{} is not a working 7-Zip executable", path.display())]
    InvalidExecutable { path: PathBuf },

    /// The executable could not be started.
    #[error("could not start 7-Zip: {0}")]
    Spawn(#[source] std::io::Error),

    /// The archive is encrypted and no password was given.
    #[error("this archive is protected by a password")]
    PasswordRequired,

    /// The password does not open the archive.
    #[error("the password is incorrect")]
    WrongPassword,

    /// 7-Zip could not read the file as an archive.
    #[error("{} is not an archive 7-Zip can open", path.display())]
    UnsupportedArchive { path: PathBuf },

    /// A volume of a multi-volume archive is missing.
    #[error("a volume of this archive is missing: {0}")]
    MissingVolume(String),

    /// The archive is damaged.
    #[error("the archive is damaged: {0}")]
    Corrupt(String),

    /// The disk is full or the destination is not writable.
    #[error("could not write the output: {0}")]
    Write(String),

    /// The operation was cancelled by the user.
    #[error("cancelled")]
    Cancelled,

    /// 7-Zip refused the command line (exit code 7). This is a bug in Coffer.
    #[error("7-Zip rejected the command line: {0}")]
    CommandLine(String),

    /// 7-Zip ran out of memory (exit code 8).
    #[error("7-Zip ran out of memory")]
    OutOfMemory,

    /// Any other failure, with 7-Zip's own message.
    #[error("{0}")]
    Failed(String),

    /// The output of 7-Zip could not be understood.
    #[error("unexpected output from 7-Zip: {0}")]
    Parse(String),

    /// An I/O error while reading 7-Zip's output.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl Error {
    /// Whether asking for a (different) password could fix the error.
    pub fn needs_password(&self) -> bool {
        matches!(self, Self::PasswordRequired | Self::WrongPassword)
    }
}

/// Classifies a failed run from its exit code and console output.
///
/// 7-Zip's exit codes are coarse (2 = fatal error), so the messages decide
/// which error the user sees. `archive` names the archive in messages.
pub fn classify(code: Option<i32>, output: &str, archive: Option<&std::path::Path>) -> Error {
    let lower = output.to_ascii_lowercase();
    let path = || archive.map(std::path::Path::to_path_buf).unwrap_or_default();

    if code == Some(255) && !lower.contains("enter password") {
        return Error::Cancelled;
    }
    if lower.contains("enter password") {
        return Error::PasswordRequired;
    }
    if lower.contains("wrong password") {
        return Error::WrongPassword;
    }
    if let Some(line) = output.lines().find(|l| l.to_ascii_lowercase().contains("missing volume")) {
        let name = line.split_once(':').map_or(line, |(_, name)| name).trim();
        return Error::MissingVolume(name.to_owned());
    }
    // A split archive whose last volumes are gone reads as truncated.
    let is_volume = archive.is_some_and(|a| {
        let numbered =
            a.file_name().and_then(|n| n.to_str()).and_then(crate::volumes::parse_name).is_some_and(|p| {
                matches!(p.scheme, crate::volumes::Scheme::Numbered | crate::volumes::Scheme::RarParts)
            });
        numbered || crate::volumes::detect(a).is_some()
    });
    if is_volume && lower.contains("unexpected end of archive") {
        return Error::MissingVolume("one of the last volumes".to_owned());
    }
    if lower.contains("cannot open the file as archive")
        || lower.contains("can not open the file as archive")
        || lower.contains("is not archive")
        || lower.contains("cannot open the file as [")
    {
        return Error::UnsupportedArchive { path: path() };
    }
    if lower.contains("there is not enough space on the disk")
        || lower.contains("no space left on device")
        || lower.contains("access is denied")
        || lower.contains("permission denied")
        || lower.contains("can not open output file")
        || lower.contains("cannot open output file")
    {
        return Error::Write(first_error_line(output));
    }
    if lower.contains("data error")
        || lower.contains("crc failed")
        || lower.contains("headers error")
        || lower.contains("unexpected end of archive")
        || lower.contains("unconfirmed start of archive")
    {
        return Error::Corrupt(first_error_line(output));
    }
    match code {
        Some(7) => Error::CommandLine(first_error_line(output)),
        Some(8) => Error::OutOfMemory,
        _ => Error::Failed(first_error_line(output)),
    }
}

/// The most useful line of 7-Zip's output for an error message.
fn first_error_line(output: &str) -> String {
    let mut lines = output.lines().map(str::trim).filter(|l| !l.is_empty());
    lines
        .clone()
        .find(|l| l.starts_with("ERROR:") || l.starts_with("Command Line Error:"))
        .map(|l| l.trim_start_matches("ERROR:").trim().to_owned())
        .or_else(|| lines.next_back().map(ToOwned::to_owned))
        .unwrap_or_else(|| "7-Zip reported an error".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_prompt_means_a_password_is_required() {
        let out = "Listing archive: hdr.7z\n\nEnter password (will not be echoed):\n\nBreak signaled";
        assert!(matches!(classify(Some(255), out, None), Error::PasswordRequired));
    }

    #[test]
    fn wrong_password_is_recognised() {
        let out = "ERROR: hdr.7z : Cannot open encrypted archive. Wrong password?";
        assert!(matches!(classify(Some(2), out, None), Error::WrongPassword));
        let out = "ERROR: Wrong password : src/a.txt";
        assert!(matches!(classify(Some(2), out, None), Error::WrongPassword));
    }

    #[test]
    fn not_an_archive() {
        let out = "ERROR: notes.txt\nnotes.txt\nCannot open the file as archive\n\nErrors: 1";
        let err = classify(Some(2), out, Some(std::path::Path::new("notes.txt")));
        assert!(matches!(err, Error::UnsupportedArchive { .. }));
    }

    #[test]
    fn user_break_is_cancelled() {
        assert!(matches!(classify(Some(255), "", None), Error::Cancelled));
    }

    #[test]
    fn other_errors_keep_the_message() {
        let err = classify(Some(2), "\nERROR: Something odd happened\n\nErrors: 1\n", None);
        assert_eq!(err.to_string(), "Something odd happened");
        assert!(matches!(classify(Some(8), "", None), Error::OutOfMemory));
        assert!(matches!(classify(Some(7), "Command Line Error:\nUnsupported switch", None), Error::CommandLine(_)));
    }

    #[test]
    fn missing_volumes() {
        let out = "Testing archive: span.zip\n\nERRORS:\nMissing volume : span.z05\n";
        let err = classify(Some(2), out, Some(std::path::Path::new("span.zip")));
        assert!(matches!(&err, Error::MissingVolume(name) if name == "span.z05"), "{err:?}");
        let out = "ERROR: split.7z.001\nsplit.7z\nOpen ERROR: Cannot open the file as [7z] archive\n\nERRORS:\nUnexpected end of archive";
        let err = classify(Some(2), out, Some(std::path::Path::new("split.7z.001")));
        assert!(matches!(err, Error::MissingVolume(_)), "{err:?}");
        // An ordinary archive cut short is damaged, not missing a volume.
        let err = classify(Some(2), "ERRORS:\nUnexpected end of archive", Some(std::path::Path::new("a.7z")));
        assert!(matches!(err, Error::Corrupt(_)), "{err:?}");
    }

    #[test]
    fn corrupt_and_write_errors() {
        assert!(matches!(classify(Some(2), "ERROR: CRC Failed : a.txt", None), Error::Corrupt(_)));
        assert!(matches!(classify(Some(2), "ERROR: No space left on device", None), Error::Write(_)));
    }
}
