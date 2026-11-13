//! User settings, stored as TOML in the platform's configuration folder.

use std::path::{Path, PathBuf};

use coffer_core::{ArchiveFormat, CompressionLevel, OverwriteMode};
use serde::{Deserialize, Serialize};

use crate::theme::ThemeChoice;

/// How many recently opened archives are remembered.
pub const RECENT_LIMIT: usize = 8;

/// Everything the user can configure. Unknown keys are ignored and missing
/// ones take their defaults, so settings files survive upgrades both ways.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub theme: ThemeChoice,
    /// An explicit 7-Zip executable; `None` searches for one.
    pub seven_zip_path: Option<PathBuf>,
    pub compress: CompressDefaults,
    pub extract: ExtractDefaults,
    pub updates: UpdateSettings,
    /// Ask before deleting files from an archive.
    pub confirm_delete: bool,
    pub recent: Vec<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: ThemeChoice::System,
            seven_zip_path: None,
            compress: CompressDefaults::default(),
            extract: ExtractDefaults::default(),
            updates: UpdateSettings::default(),
            confirm_delete: true,
            recent: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct CompressDefaults {
    pub format: ArchiveFormat,
    pub level: CompressionLevel,
    pub solid: bool,
    pub encrypt_names: bool,
    pub zip_aes: bool,
}

impl Default for CompressDefaults {
    fn default() -> Self {
        Self {
            format: ArchiveFormat::SevenZ,
            level: CompressionLevel::Normal,
            solid: true,
            encrypt_names: true,
            zip_aes: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ExtractDefaults {
    pub overwrite: OverwriteMode,
    pub keep_paths: bool,
    /// Extract into a new folder named after the archive.
    pub into_folder: bool,
    pub open_when_done: bool,
}

impl Default for ExtractDefaults {
    fn default() -> Self {
        Self { overwrite: OverwriteMode::Skip, keep_paths: true, into_folder: true, open_when_done: true }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UpdateSettings {
    /// Check the website for a new version at startup and every few hours.
    pub check_automatically: bool,
    /// A version the user chose to skip.
    pub skipped_version: Option<String>,
}

impl Default for UpdateSettings {
    fn default() -> Self {
        Self { check_automatically: true, skipped_version: None }
    }
}

impl Settings {
    /// Loads the settings, falling back to defaults when the file is missing
    /// or unreadable (a broken file is kept aside as `settings.toml.bak`).
    pub fn load() -> Self {
        let Some(path) = paths::settings_file() else { return Self::default() };
        Self::load_from(&path)
    }

    pub fn load_from(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(text) => toml::from_str(&text).unwrap_or_else(|error| {
                tracing::warn!(%error, "the settings file is invalid; using defaults");
                let _ = std::fs::rename(path, path.with_extension("toml.bak"));
                Self::default()
            }),
            Err(_) => Self::default(),
        }
    }

    /// Saves atomically: a crash mid-write never leaves a truncated file.
    pub fn save(&self) {
        let Some(path) = paths::settings_file() else { return };
        if let Err(error) = self.save_to(&path) {
            tracing::warn!(%error, path = %path.display(), "could not save the settings");
        }
    }

    pub fn save_to(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let text = toml::to_string_pretty(self).map_err(std::io::Error::other)?;
        let staging = path.with_extension("toml.tmp");
        std::fs::write(&staging, text)?;
        std::fs::rename(staging, path)
    }

    /// Moves `archive` to the top of the recent list.
    pub fn remember(&mut self, archive: &Path) {
        self.recent.retain(|p| p != archive);
        self.recent.insert(0, archive.to_path_buf());
        self.recent.truncate(RECENT_LIMIT);
    }
}

/// Where Coffer keeps its files on each platform.
pub mod paths {
    use std::path::PathBuf;

    fn base() -> Option<directories::BaseDirs> {
        directories::BaseDirs::new()
    }

    /// `~/.config/coffer`, `~/Library/Application Support/io.github.damienreichhart.Coffer`
    /// or `%APPDATA%\DamienReichhart\Coffer\config`.
    pub fn config_dir() -> Option<PathBuf> {
        let base = base()?;
        Some(if cfg!(windows) {
            base.config_dir().join("DamienReichhart").join("Coffer").join("config")
        } else if cfg!(target_os = "macos") {
            base.config_dir().join("io.github.damienreichhart.Coffer")
        } else {
            base.config_dir().join("coffer")
        })
    }

    pub fn settings_file() -> Option<PathBuf> {
        Some(config_dir()?.join("settings.toml"))
    }

    /// Logs and downloaded updates.
    pub fn data_dir() -> Option<PathBuf> {
        let base = base()?;
        Some(if cfg!(windows) {
            base.data_local_dir().join("DamienReichhart").join("Coffer").join("data")
        } else if cfg!(target_os = "macos") {
            base.data_local_dir().join("io.github.damienreichhart.Coffer")
        } else {
            base.data_local_dir().join("coffer")
        })
    }

    pub fn log_dir() -> Option<PathBuf> {
        Some(data_dir()?.join("logs"))
    }

    pub fn update_dir() -> PathBuf {
        data_dir().unwrap_or_else(std::env::temp_dir).join("updates")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_toml() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("settings.toml");
        let mut settings = Settings { theme: ThemeChoice::Dark, ..Settings::default() };
        settings.compress.format = ArchiveFormat::Zip;
        settings.remember(Path::new("/a.7z"));
        settings.save_to(&path).expect("save");
        assert_eq!(Settings::load_from(&path), settings);
    }

    #[test]
    fn tolerates_partial_and_broken_files() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("settings.toml");
        std::fs::write(&path, "theme = \"light\"\nfuture_option = 3\n").expect("write");
        let settings = Settings::load_from(&path);
        assert_eq!(settings.theme, ThemeChoice::Light);
        assert!(settings.updates.check_automatically);

        std::fs::write(&path, "theme = [").expect("write");
        assert_eq!(Settings::load_from(&path), Settings::default());
        assert!(path.with_extension("toml.bak").exists());
    }

    #[test]
    fn recent_list_is_bounded_and_deduplicated() {
        let mut settings = Settings::default();
        for i in 0..20 {
            settings.remember(Path::new(&format!("/{i}.zip")));
        }
        settings.remember(Path::new("/5.zip"));
        assert_eq!(settings.recent.len(), RECENT_LIMIT);
        assert_eq!(settings.recent[0], Path::new("/5.zip"));
        assert_eq!(settings.recent.iter().filter(|p| p.ends_with("5.zip")).count(), 1);
    }
}
