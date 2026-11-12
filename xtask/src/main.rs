//! `cargo xtask <command>`: the release pipeline's tooling, in Rust so it runs
//! the same on every CI runner and on a developer's machine.
//!
//! | Command | What it does |
//! | --- | --- |
//! | `release-plan` | Validates the version to release (semver, not yet tagged, newer than every tag, a CHANGELOG entry, a consistent lockfile) and writes the release notes. |
//! | `check-signing` | Checks the update-signing secret matches `packaging/keys/update.pub`. |
//! | `package` | Builds the installers of one target from a release binary. |
//! | `sign` | Writes a minisign signature next to each file. |
//! | `keygen` | Creates the update-signing key pair. |
//! | `icons` | Renders every icon size from `assets/brand/icon.svg`. |
//! | `notes` | Prints the CHANGELOG section of a version. |

mod changelog;
mod icons;
mod package;
mod release;
mod sign;

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "xtask", about = "7zip-GUI release tooling")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Validate the release and write its notes; prints `key=value` outputs
    /// (also appended to `$GITHUB_OUTPUT` when set).
    ReleasePlan {
        /// Where to write the release notes (Markdown).
        #[arg(long)]
        notes: Option<PathBuf>,
        /// Skip the git tag checks (for local dry runs).
        #[arg(long)]
        no_git: bool,
    },
    /// Check that `UPDATE_SIGNING_KEY` (and `UPDATE_SIGNING_PASSWORD`) match the public key.
    CheckSigning,
    /// Package a release binary into this target's installers.
    Package {
        /// The Rust target triple the binary was built for.
        #[arg(long)]
        target: String,
        /// Formats to build: deb, rpm, appimage, msi, portable, dmg. Defaults to all of the target's.
        #[arg(long, value_delimiter = ',')]
        formats: Vec<String>,
        /// Output folder.
        #[arg(long, default_value = "dist")]
        out: PathBuf,
        /// The binary; defaults to `target/<target>/release/7zip-gui[.exe]`.
        #[arg(long)]
        binary: Option<PathBuf>,
    },
    /// Sign files with the update key (minisign), writing `<file>.minisig`.
    Sign {
        /// The version being released, bound into each signature.
        #[arg(long)]
        version: String,
        files: Vec<PathBuf>,
    },
    /// Create the update-signing key pair.
    Keygen {
        /// Where to write the secret key (keep it out of the repository).
        #[arg(long)]
        secret: PathBuf,
        /// Encrypt the secret key with `UPDATE_SIGNING_PASSWORD`.
        #[arg(long)]
        encrypted: bool,
    },
    /// Render all icon sizes from the SVG source.
    Icons,
    /// Print the CHANGELOG section of a version.
    Notes { version: String },
}

fn main() -> Result<()> {
    let root = workspace_root()?;
    std::env::set_current_dir(&root)?;
    match Cli::parse().command {
        Command::ReleasePlan { notes, no_git } => release::plan(&root, notes.as_deref(), !no_git),
        Command::CheckSigning => sign::check(&root),
        Command::Package { target, formats, out, binary } => package::run(&root, &target, &formats, &out, binary),
        Command::Sign { version, files } => sign::sign_files(&version, &files),
        Command::Keygen { secret, encrypted } => sign::keygen(&root, &secret, encrypted),
        Command::Icons => icons::render(&root),
        Command::Notes { version } => {
            let text = std::fs::read_to_string(root.join("CHANGELOG.md"))?;
            let section =
                changelog::section(&text, &version).with_context(|| format!("no CHANGELOG entry for {version}"))?;
            println!("{}", section.body);
            Ok(())
        }
    }
}

fn workspace_root() -> Result<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    dir.parent().map(Path::to_path_buf).context("xtask has no parent folder")
}

/// The version in `[workspace.package]`.
pub fn workspace_version(root: &Path) -> Result<semver::Version> {
    let manifest: toml::Table = std::fs::read_to_string(root.join("Cargo.toml"))?.parse()?;
    let version = manifest
        .get("workspace")
        .and_then(|w| w.get("package"))
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .context("no [workspace.package] version in Cargo.toml")?;
    version.parse().with_context(|| format!("{version} is not a semantic version"))
}

/// Runs a command, failing with its output when it fails.
pub fn run(command: &mut std::process::Command) -> Result<()> {
    eprintln!("$ {command:?}");
    let status = command.status().with_context(|| format!("could not start {}", command.get_program().display()))?;
    if !status.success() {
        bail!("{} failed with {status}", command.get_program().display());
    }
    Ok(())
}

/// Runs a command and returns its standard output.
pub fn capture(command: &mut std::process::Command) -> Result<String> {
    let output = command.output().with_context(|| format!("could not start {}", command.get_program().display()))?;
    if !output.status.success() {
        bail!("{} failed: {}", command.get_program().display(), String::from_utf8_lossy(&output.stderr));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}
