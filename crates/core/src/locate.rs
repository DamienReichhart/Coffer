//! Finding the 7-Zip executable the user installed.
//!
//! The search order is: an explicit path from the settings, then `PATH`
//! (`7zz`, `7z`, `7za`), then the places installers put 7-Zip on each platform.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::version::Version;

/// Which build of 7-Zip was found. They accept the same switches; the
/// difference matters for the formats they can write and for messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Flavor {
    /// The official 7-Zip console (`7zz` on Linux/macOS, `7z.exe` on Windows).
    Official,
    /// A distribution build with plugins (`7z`), including p7zip.
    Full,
    /// The standalone reduced build (`7za`), without RAR support.
    Standalone,
    /// The legacy p7zip fork.
    P7zip,
}

/// A working 7-Zip executable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SevenZip {
    pub path: PathBuf,
    pub version: Version,
    pub flavor: Flavor,
}

/// The executable names, in order of preference.
#[cfg(not(windows))]
const NAMES: &[&str] = &["7zz", "7z", "7za"];
#[cfg(windows)]
const NAMES: &[&str] = &["7z", "7zz", "7za"];

impl SevenZip {
    /// Finds 7-Zip: `configured` first when given, then `PATH`, then the
    /// usual install locations. Returns [`Error::NotInstalled`] when none works.
    pub async fn locate(configured: Option<&Path>) -> Result<Self> {
        if let Some(path) = configured {
            return Self::probe(path).await;
        }
        for candidate in candidates() {
            match Self::probe(&candidate).await {
                Ok(found) => return Ok(found),
                Err(error) => {
                    tracing::debug!(path = %candidate.display(), %error, "not a usable 7-Zip");
                }
            }
        }
        Err(Error::NotInstalled)
    }

    /// Checks that `path` is a 7-Zip executable and reads its version.
    pub async fn probe(path: &Path) -> Result<Self> {
        let invalid = || Error::InvalidExecutable { path: path.to_path_buf() };
        let mut command = tokio::process::Command::new(path);
        command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).kill_on_drop(true);
        crate::runner::hide_console(&mut command);
        let output = tokio::time::timeout(Duration::from_secs(10), command.output())
            .await
            .map_err(|_| invalid())?
            .map_err(|_| invalid())?;
        let banner = String::from_utf8_lossy(&output.stdout);
        let version = Version::from_banner(&banner).ok_or_else(invalid)?;
        Ok(Self { path: path.to_path_buf(), version, flavor: flavor(path, &banner) })
    }

    /// Whether this build can create archives of the given format.
    pub fn can_write(&self, format: crate::ArchiveFormat) -> bool {
        use crate::ArchiveFormat as F;
        match format {
            // The reduced 7za/p7zip builds lack WIM writing in old versions.
            F::Wim => self.flavor != Flavor::P7zip,
            _ => true,
        }
    }
}

fn flavor(path: &Path, banner: &str) -> Flavor {
    if banner.contains("p7zip") {
        return Flavor::P7zip;
    }
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or_default();
    match stem.to_ascii_lowercase().as_str() {
        "7za" => Flavor::Standalone,
        "7z" if !cfg!(windows) => Flavor::Full,
        _ => Flavor::Official,
    }
}

/// Every place 7-Zip may be, most likely first, without duplicates.
pub fn candidates() -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    let mut push = |path: PathBuf| {
        if !found.contains(&path) {
            found.push(path);
        }
    };
    for name in NAMES {
        if let Ok(path) = which::which(name) {
            push(path);
        }
    }
    for path in well_known_locations() {
        if path.is_file() {
            push(path);
        }
    }
    found
}

#[cfg(windows)]
fn well_known_locations() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    // The official installer records its folder in the registry.
    for root in [winreg::enums::HKEY_LOCAL_MACHINE, winreg::enums::HKEY_CURRENT_USER] {
        let key = winreg::RegKey::predef(root);
        if let Ok(sub) = key.open_subkey("SOFTWARE\\7-Zip") {
            for value in ["Path64", "Path"] {
                if let Ok(dir) = sub.get_value::<String, _>(value) {
                    paths.push(PathBuf::from(dir).join("7z.exe"));
                }
            }
        }
    }
    for var in ["ProgramFiles", "ProgramW6432", "ProgramFiles(x86)", "ProgramFiles(Arm)"] {
        if let Some(dir) = std::env::var_os(var) {
            paths.push(PathBuf::from(dir).join("7-Zip").join("7z.exe"));
        }
    }
    if let Some(home) = std::env::var_os("USERPROFILE") {
        let home = PathBuf::from(home);
        paths.push(home.join("scoop\\apps\\7zip\\current\\7z.exe"));
        paths.push(home.join("scoop\\shims\\7z.exe"));
    }
    if let Some(choco) = std::env::var_os("ChocolateyInstall") {
        paths.push(PathBuf::from(choco).join("bin\\7z.exe"));
    }
    paths
}

#[cfg(target_os = "macos")]
fn well_known_locations() -> Vec<PathBuf> {
    // GUI apps on macOS don't inherit the shell's PATH, so Homebrew and
    // MacPorts are looked up explicitly.
    ["/opt/homebrew/bin", "/usr/local/bin", "/opt/local/bin"]
        .iter()
        .flat_map(|dir| NAMES.iter().map(move |name| Path::new(dir).join(name)))
        .collect()
}

#[cfg(all(unix, not(target_os = "macos")))]
fn well_known_locations() -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = ["/usr/bin", "/usr/local/bin", "/bin", "/snap/bin", "/usr/lib/p7zip"]
        .iter()
        .flat_map(|dir| NAMES.iter().map(move |name| Path::new(dir).join(name)))
        .collect();
    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        paths.push(home.join(".local/bin/7zz"));
        paths.push(home.join(".nix-profile/bin/7zz"));
        paths.push(home.join(".nix-profile/bin/7z"));
    }
    paths.push(PathBuf::from("/run/current-system/sw/bin/7zz"));
    paths.push(PathBuf::from("/var/lib/flatpak/exports/bin/7zz"));
    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flavors() {
        let official = "7-Zip (z) 24.08 (x64)";
        assert_eq!(flavor(Path::new("/usr/bin/7zz"), official), Flavor::Official);
        assert_eq!(flavor(Path::new("/usr/bin/7za"), official), Flavor::Standalone);
        assert_eq!(flavor(Path::new("/usr/bin/7z"), "p7zip Version 16.02"), Flavor::P7zip);
    }

    #[test]
    fn candidates_have_no_duplicates() {
        let all = candidates();
        let mut unique = all.clone();
        unique.dedup();
        assert_eq!(all.len(), unique.len());
    }

    #[tokio::test]
    async fn a_missing_configured_path_is_invalid() {
        let err = SevenZip::locate(Some(Path::new("/definitely/not/here/7z"))).await;
        assert!(matches!(err, Err(Error::InvalidExecutable { .. })));
    }
}
