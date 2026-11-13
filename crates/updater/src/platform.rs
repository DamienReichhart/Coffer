//! Which installer fits the running copy of the app.

use std::fmt;
use std::path::{Path, PathBuf};

/// An operating system the app ships for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Os {
    Linux,
    Windows,
    Macos,
}

/// A CPU architecture the app ships for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Arch {
    X64,
    Arm64,
}

/// The platform the app runs on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Target {
    pub os: Os,
    pub arch: Arch,
}

impl Target {
    /// The platform this binary was built for, `None` elsewhere (BSDs…).
    pub fn current() -> Option<Self> {
        let os = if cfg!(target_os = "linux") {
            Os::Linux
        } else if cfg!(windows) {
            Os::Windows
        } else if cfg!(target_os = "macos") {
            Os::Macos
        } else {
            return None;
        };
        let arch = if cfg!(target_arch = "x86_64") {
            Arch::X64
        } else if cfg!(target_arch = "aarch64") {
            Arch::Arm64
        } else {
            return None;
        };
        Some(Self { os, arch })
    }

    /// The manifest key of this platform's installer of the given kind,
    /// e.g. `linux-x64-deb`.
    pub fn asset_key(self, kind: InstallKind) -> String {
        format!("{}-{}-{}", self.os, self.arch, kind)
    }
}

impl fmt::Display for Os {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Linux => "linux",
            Self::Windows => "windows",
            Self::Macos => "macos",
        })
    }
}

impl fmt::Display for Arch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::X64 => "x64",
            Self::Arm64 => "arm64",
        })
    }
}

/// How the running copy was installed, which decides how to update it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstallKind {
    Deb,
    Rpm,
    AppImage,
    Msi,
    Portable,
    Dmg,
}

impl InstallKind {
    pub const ALL: [Self; 6] = [Self::Deb, Self::Rpm, Self::AppImage, Self::Msi, Self::Portable, Self::Dmg];

    /// Reads the kind of the running copy: the `APPIMAGE` variable, or the
    /// `install-kind` marker each package installs. `None` for a development
    /// build, which can't update itself.
    pub fn detect() -> Option<Self> {
        if cfg!(target_os = "linux") && std::env::var_os("APPIMAGE").is_some() {
            return Some(Self::AppImage);
        }
        let exe = std::env::current_exe().ok()?;
        marker_paths(&exe)
            .into_iter()
            .find_map(|path| std::fs::read_to_string(path).ok())
            .and_then(|text| Self::parse(text.trim()))
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.to_string() == text)
    }

    /// The file extension of this kind's installers.
    pub fn extension(self) -> &'static str {
        match self {
            Self::Deb => "deb",
            Self::Rpm => "rpm",
            Self::AppImage => "AppImage",
            Self::Msi => "msi",
            Self::Portable => "zip",
            Self::Dmg => "dmg",
        }
    }
}

impl fmt::Display for InstallKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Deb => "deb",
            Self::Rpm => "rpm",
            Self::AppImage => "appimage",
            Self::Msi => "msi",
            Self::Portable => "portable",
            Self::Dmg => "dmg",
        })
    }
}

/// Where packages put the `install-kind` marker for an executable at `exe`.
pub fn marker_paths(exe: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(dir) = exe.parent() {
        // Windows (MSI, portable) keeps it beside the binary.
        paths.push(dir.join("install-kind"));
        if let Some(prefix) = dir.parent() {
            // deb and rpm: /usr/bin/coffer → /usr/lib/coffer/install-kind.
            paths.push(prefix.join("lib").join("coffer").join("install-kind"));
            // macOS: Coffer.app/Contents/MacOS/coffer → Contents/Resources/install-kind
            // (codesign allows only code in Contents/MacOS).
            paths.push(prefix.join("Resources").join("install-kind"));
        }
    }
    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_keys_follow_the_release_contract() {
        let target = Target { os: Os::Linux, arch: Arch::Arm64 };
        assert_eq!(target.asset_key(InstallKind::AppImage), "linux-arm64-appimage");
        let target = Target { os: Os::Windows, arch: Arch::X64 };
        assert_eq!(target.asset_key(InstallKind::Portable), "windows-x64-portable");
    }

    #[test]
    fn kinds_round_trip() {
        for kind in InstallKind::ALL {
            assert_eq!(InstallKind::parse(&kind.to_string()), Some(kind));
        }
        assert_eq!(InstallKind::parse("snap"), None);
    }

    #[test]
    fn marker_locations() {
        let paths = marker_paths(Path::new("/usr/bin/coffer"));
        assert_eq!(paths[0], Path::new("/usr/bin/install-kind"));
        assert_eq!(paths[1], Path::new("/usr/lib/coffer/install-kind"));
        let mac = marker_paths(Path::new("/Applications/Coffer.app/Contents/MacOS/coffer"));
        assert!(mac.contains(&PathBuf::from("/Applications/Coffer.app/Contents/Resources/install-kind")));
    }

    #[test]
    fn the_build_platform_is_supported() {
        // CI builds on every shipped platform, so this documents the matrix.
        if cfg!(any(target_arch = "x86_64", target_arch = "aarch64"))
            && cfg!(any(target_os = "linux", windows, target_os = "macos"))
        {
            assert!(Target::current().is_some());
        }
    }
}
