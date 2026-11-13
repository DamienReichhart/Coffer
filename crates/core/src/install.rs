//! How to install 7-Zip, for the "7-Zip not found" screen.

/// One way to install 7-Zip on the current platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Method {
    /// Who this is for, e.g. "Debian, Ubuntu".
    pub label: &'static str,
    /// A command to run, or `None` when it's a download.
    pub command: Option<&'static str>,
}

/// The official download page, for platforms without a package manager entry.
pub const DOWNLOAD_PAGE: &str = "https://www.7-zip.org/download.html";

/// Install methods for the platform this was compiled for, best first.
pub fn methods() -> &'static [Method] {
    #[cfg(windows)]
    {
        &[
            Method { label: "winget", command: Some("winget install --id 7zip.7zip -e") },
            Method { label: "Scoop", command: Some("scoop install 7zip") },
            Method { label: "Chocolatey", command: Some("choco install 7zip") },
            Method { label: "Installer from 7-zip.org", command: None },
        ]
    }
    #[cfg(target_os = "macos")]
    {
        &[
            Method { label: "Homebrew", command: Some("brew install sevenzip") },
            Method { label: "MacPorts", command: Some("sudo port install 7zip") },
            Method { label: "Download from 7-zip.org", command: None },
        ]
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        &[
            Method { label: "Debian 12+, Ubuntu 22.04+", command: Some("sudo apt install 7zip") },
            Method { label: "Older Debian and Ubuntu", command: Some("sudo apt install p7zip-full") },
            Method { label: "Fedora", command: Some("sudo dnf install 7zip") },
            Method { label: "openSUSE", command: Some("sudo zypper install 7zip") },
            Method { label: "Arch Linux", command: Some("sudo pacman -S 7zip") },
            Method { label: "Download from 7-zip.org", command: None },
        ]
    }
}
