//! Turns typed options into 7-Zip command lines.
//!
//! Every builder returns the arguments only, never runs anything, so the
//! exact command line of every operation is covered by unit tests.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::locate::{Flavor, SevenZip};
use crate::options::{ArchiveFormat, CompressOptions, ExtractOptions, HashAlgorithm};

/// Switches that depend on the 7-Zip build rather than on the operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dialect {
    /// Whether `-scc` (console charset) and `-spd` are understood. The legacy
    /// p7zip fork predates them.
    pub modern: bool,
}

impl Dialect {
    pub fn of(seven: &SevenZip) -> Self {
        Self { modern: seven.flavor != Flavor::P7zip }
    }

    pub const MODERN: Self = Self { modern: true };

    fn console(self, args: &mut Vec<OsString>) {
        if self.modern {
            args.push("-sccUTF-8".into());
        }
    }
}

/// A full command line: the arguments after the executable.
pub type Args = Vec<OsString>;

fn password(args: &mut Args, password: Option<&str>) {
    if let Some(p) = password.filter(|p| !p.is_empty()) {
        args.push(format!("-p{p}").into());
    }
}

/// `7z l -slt`: the technical listing of an archive.
pub fn list(dialect: Dialect, archive: &Path, pass: Option<&str>) -> Args {
    let mut args: Args = vec!["l".into(), "-slt".into()];
    dialect.console(&mut args);
    password(&mut args, pass);
    args.push("--".into());
    args.push(archive.into());
    args
}

/// `7z x` (or `e` without folders): extracts all or some entries.
pub fn extract(dialect: Dialect, archive: &Path, options: &ExtractOptions) -> Args {
    let command = if options.keep_paths { "x" } else { "e" };
    let mut args: Args = vec![command.into(), "-bsp1".into(), "-bb1".into(), "-y".into()];
    dialect.console(&mut args);
    args.push(options.overwrite.switch().into());
    let mut out = OsString::from("-o");
    out.push(&options.destination);
    args.push(out);
    password(&mut args, options.password.as_deref());
    if !options.only.is_empty() && dialect.modern {
        // Names are literal: a file called `a[1].txt` is not a wildcard.
        args.push("-spd".into());
    }
    args.push("--".into());
    args.push(archive.into());
    args.extend(options.only.iter().map(OsString::from));
    args
}

/// `7z a`: creates an archive, or adds to an existing one.
pub fn add(dialect: Dialect, archive: &Path, inputs: &[PathBuf], options: &CompressOptions) -> Args {
    let format = options.format;
    let mut args: Args =
        vec!["a".into(), format!("-t{}", format.switch()).into(), "-bsp1".into(), "-bb1".into(), "-y".into()];
    dialect.console(&mut args);

    if format.supports_compression() {
        args.push(format!("-mx={}", options.level.value()).into());
        if let Some(method) = options.method.filter(|m| format.methods().contains(m)) {
            let switch = match format {
                ArchiveFormat::SevenZ => "-m0",
                _ => "-mm",
            };
            args.push(format!("{switch}={}", method.switch()).into());
        }
        if let Some(dict) = options.dictionary_mib
            && matches!(format, ArchiveFormat::SevenZ | ArchiveFormat::Xz)
        {
            args.push(format!("-md={dict}m").into());
        }
    }
    if format.supports_solid() {
        args.push(if options.solid { "-ms=on" } else { "-ms=off" }.into());
    }
    if format.supports_multithreading() {
        match options.threads {
            Some(n) => args.push(format!("-mmt={n}").into()),
            None => args.push("-mmt=on".into()),
        }
    }
    let encrypted = format.supports_encryption() && options.password.as_deref().is_some_and(|p| !p.is_empty());
    if encrypted {
        password(&mut args, options.password.as_deref());
        if format.supports_header_encryption() && options.encrypt_names {
            args.push("-mhe=on".into());
        }
        if format == ArchiveFormat::Zip && options.zip_aes {
            args.push("-mem=AES256".into());
        }
    }
    if let Some(split) = options.split {
        args.push(format!("-v{}b", split.bytes()).into());
    }
    if options.delete_after {
        args.push("-sdel".into());
    }
    for pattern in options.excludes.iter().map(|p| p.trim()).filter(|p| !p.is_empty()) {
        args.push(format!("-xr!{pattern}").into());
    }
    args.push("--".into());
    args.push(archive.into());
    args.extend(inputs.iter().map(OsString::from));
    args
}

/// `7z t`: tests the integrity of an archive.
pub fn test(dialect: Dialect, archive: &Path, pass: Option<&str>) -> Args {
    let mut args: Args = vec!["t".into(), "-bsp1".into(), "-bb1".into()];
    dialect.console(&mut args);
    password(&mut args, pass);
    args.push("--".into());
    args.push(archive.into());
    args
}

/// `7z d`: deletes entries from an archive.
pub fn delete(dialect: Dialect, archive: &Path, entries: &[String], pass: Option<&str>) -> Args {
    let mut args: Args = vec!["d".into(), "-bsp1".into(), "-y".into()];
    dialect.console(&mut args);
    password(&mut args, pass);
    if dialect.modern {
        args.push("-spd".into());
    }
    args.push("--".into());
    args.push(archive.into());
    args.extend(entries.iter().map(OsString::from));
    args
}

/// `7z rn`: renames an entry inside an archive.
pub fn rename(dialect: Dialect, archive: &Path, from: &str, to: &str, pass: Option<&str>) -> Args {
    let mut args: Args = vec!["rn".into(), "-bsp1".into(), "-y".into()];
    dialect.console(&mut args);
    password(&mut args, pass);
    args.push("--".into());
    args.push(archive.into());
    args.push(from.into());
    args.push(to.into());
    args
}

/// `7z h`: computes checksums of files and folders.
pub fn hash(dialect: Dialect, files: &[PathBuf], algorithms: &[HashAlgorithm]) -> Args {
    let mut args: Args = vec!["h".into(), "-bsp1".into()];
    dialect.console(&mut args);
    for algorithm in algorithms {
        args.push(format!("-scrc{}", algorithm.switch()).into());
    }
    args.push("--".into());
    args.extend(files.iter().map(OsString::from));
    args
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::{CompressionLevel, CompressionMethod, OverwriteMode, SplitSize};

    fn strings(args: &Args) -> Vec<String> {
        args.iter().map(|a| a.to_string_lossy().into_owned()).collect()
    }

    #[test]
    fn listing() {
        let args = list(Dialect::MODERN, Path::new("a.7z"), Some("pw"));
        assert_eq!(strings(&args), ["l", "-slt", "-sccUTF-8", "-ppw", "--", "a.7z"]);
        let legacy = list(Dialect { modern: false }, Path::new("a.7z"), None);
        assert_eq!(strings(&legacy), ["l", "-slt", "--", "a.7z"]);
    }

    #[test]
    fn extracting_everything() {
        let options = ExtractOptions {
            destination: "out".into(),
            overwrite: OverwriteMode::RenameNew,
            keep_paths: true,
            password: None,
            only: vec![],
        };
        let args = extract(Dialect::MODERN, Path::new("a.zip"), &options);
        assert_eq!(strings(&args), ["x", "-bsp1", "-bb1", "-y", "-sccUTF-8", "-aou", "-oout", "--", "a.zip"]);
    }

    #[test]
    fn extracting_a_selection_flat() {
        let options = ExtractOptions {
            destination: "out".into(),
            overwrite: OverwriteMode::Overwrite,
            keep_paths: false,
            password: Some("s3cret".into()),
            only: vec!["docs/a[1].txt".into()],
        };
        let args = strings(&extract(Dialect::MODERN, Path::new("a.7z"), &options));
        assert_eq!(args[0], "e");
        assert!(args.contains(&"-ps3cret".to_owned()));
        assert!(args.contains(&"-spd".to_owned()));
        assert_eq!(args.last().map(String::as_str), Some("docs/a[1].txt"));
    }

    #[test]
    fn creating_an_encrypted_split_7z() {
        let options = CompressOptions {
            format: ArchiveFormat::SevenZ,
            level: CompressionLevel::Ultra,
            method: Some(CompressionMethod::Lzma2),
            dictionary_mib: Some(64),
            solid: true,
            threads: Some(4),
            password: Some("pw".into()),
            encrypt_names: true,
            zip_aes: true,
            split: Some(SplitSize::Bytes(1_000_000)),
            delete_after: false,
            excludes: vec!["*.tmp".into(), "  ".into()],
        };
        let inputs = vec![PathBuf::from("/data/photos"), PathBuf::from("/data/notes.txt")];
        let args = strings(&add(Dialect::MODERN, Path::new("out.7z"), &inputs, &options));
        assert_eq!(
            args,
            [
                "a",
                "-t7z",
                "-bsp1",
                "-bb1",
                "-y",
                "-sccUTF-8",
                "-mx=9",
                "-m0=LZMA2",
                "-md=64m",
                "-ms=on",
                "-mmt=4",
                "-ppw",
                "-mhe=on",
                "-v1000000b",
                "-xr!*.tmp",
                "--",
                "out.7z",
                "/data/photos",
                "/data/notes.txt"
            ]
        );
    }

    #[test]
    fn creating_a_zip_with_aes() {
        let options = CompressOptions {
            format: ArchiveFormat::Zip,
            password: Some("pw".into()),
            encrypt_names: true,
            zip_aes: true,
            delete_after: true,
            ..Default::default()
        };
        let args = strings(&add(Dialect::MODERN, Path::new("o.zip"), &[PathBuf::from("a")], &options));
        assert!(args.contains(&"-tzip".to_owned()));
        assert!(args.contains(&"-mem=AES256".to_owned()));
        assert!(args.contains(&"-sdel".to_owned()));
        // Zip can't hide names; 7z's -mhe must not leak into a zip command.
        assert!(!args.contains(&"-mhe=on".to_owned()));
        assert!(!args.iter().any(|a| a.starts_with("-ms=")));
    }

    #[test]
    fn tar_takes_no_compression_switches() {
        let options = CompressOptions { format: ArchiveFormat::Tar, ..Default::default() };
        let args = strings(&add(Dialect::MODERN, Path::new("o.tar"), &[PathBuf::from("a")], &options));
        assert!(!args.iter().any(|a| a.starts_with("-mx") || a.starts_with("-mmt")));
    }

    #[test]
    fn maintenance_commands() {
        let t = strings(&test(Dialect::MODERN, Path::new("a.7z"), None));
        assert_eq!(t, ["t", "-bsp1", "-bb1", "-sccUTF-8", "--", "a.7z"]);
        let d = strings(&delete(Dialect::MODERN, Path::new("a.7z"), &["x/y".into()], None));
        assert_eq!(d, ["d", "-bsp1", "-y", "-sccUTF-8", "-spd", "--", "a.7z", "x/y"]);
        let r = strings(&rename(Dialect::MODERN, Path::new("a.7z"), "old", "new", None));
        assert_eq!(r, ["rn", "-bsp1", "-y", "-sccUTF-8", "--", "a.7z", "old", "new"]);
        let h = strings(&hash(Dialect::MODERN, &[PathBuf::from("f")], &[HashAlgorithm::Crc32, HashAlgorithm::Sha256]));
        assert_eq!(h, ["h", "-bsp1", "-sccUTF-8", "-scrcCRC32", "-scrcSHA256", "--", "f"]);
    }
}
