//! The command line: `coffer [PATHS…]`, `--extract`, `--compress`.
//!
//! Desktop integrations (file associations, "Open with", the context menus a
//! packager adds) call the app this way.

use std::path::PathBuf;

use clap::Parser;

/// A modern desktop interface for 7-Zip.
#[derive(Debug, Parser)]
#[command(name = "coffer", version, about, long_about = None)]
pub struct Cli {
    /// An archive to open, or files and folders to compress.
    pub paths: Vec<PathBuf>,

    /// Prepare the extraction of this archive.
    #[arg(long, value_name = "ARCHIVE", conflicts_with = "compress")]
    pub extract: Option<PathBuf>,

    /// Where to extract (with --extract).
    #[arg(long, value_name = "DIR", requires = "extract")]
    pub to: Option<PathBuf>,

    /// Prepare a new archive of these files and folders.
    #[arg(long, value_name = "PATH", num_args = 1..)]
    pub compress: Vec<PathBuf>,
}

/// What the app should show once it has started.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Launch {
    #[default]
    Home,
    Open(PathBuf),
    Extract {
        archive: PathBuf,
        to: Option<PathBuf>,
    },
    Compress(Vec<PathBuf>),
}

impl Cli {
    pub fn into_launch(self) -> Launch {
        let absolute = |p: PathBuf| std::path::absolute(&p).unwrap_or(p);
        if let Some(archive) = self.extract {
            return Launch::Extract { archive: absolute(archive), to: self.to.map(absolute) };
        }
        if !self.compress.is_empty() {
            return Launch::Compress(self.compress.into_iter().map(absolute).collect());
        }
        let paths: Vec<PathBuf> = self.paths.into_iter().map(absolute).collect();
        // Opening every volume of a split archive opens the archive once.
        let mut archives = coffer_core::volumes::collapse(&paths);
        match archives.len() {
            0 => Launch::Home,
            1 if archives[0].is_file() && crate::app::looks_like_archive(&archives[0]) => {
                Launch::Open(archives.remove(0))
            }
            _ => Launch::Compress(paths),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn launch(args: &[&str]) -> Launch {
        Cli::try_parse_from(std::iter::once("coffer").chain(args.iter().copied()))
            .expect("valid arguments")
            .into_launch()
    }

    #[test]
    fn no_arguments_opens_home() {
        assert_eq!(launch(&[]), Launch::Home);
    }

    #[test]
    fn extract_and_compress_flags() {
        assert!(matches!(launch(&["--extract", "a.7z", "--to", "out"]), Launch::Extract { to: Some(_), .. }));
        assert!(matches!(launch(&["--compress", "a", "b"]), Launch::Compress(p) if p.len() == 2));
    }

    #[test]
    fn plain_paths_compress_unless_one_archive() {
        assert!(matches!(launch(&["notes.txt", "photos"]), Launch::Compress(_)));
        let dir = tempfile::tempdir().expect("tempdir");
        let archive = dir.path().join("a.zip");
        std::fs::write(&archive, b"PK").expect("write");
        let arg = archive.to_string_lossy().into_owned();
        assert!(matches!(launch(&[&arg]), Launch::Open(_)));

        // Every volume of a split archive, as a file manager passes them.
        for n in 1..=3 {
            std::fs::write(dir.path().join(format!("set.7z.00{n}")), b"x").expect("write");
        }
        let volumes: Vec<String> =
            (1..=3).map(|n| dir.path().join(format!("set.7z.00{n}")).display().to_string()).collect();
        let args: Vec<&str> = volumes.iter().map(String::as_str).collect();
        assert!(matches!(launch(&args), Launch::Open(p) if p.ends_with("set.7z.001")));
    }

    #[test]
    fn rejects_to_without_extract() {
        assert!(Cli::try_parse_from(["coffer", "--to", "x"]).is_err());
    }
}
