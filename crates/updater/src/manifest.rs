//! The update manifest the website publishes (`updates/latest.json`).
//!
//! See `docs/ARCHITECTURE.md` for the contract with the website.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::platform::{InstallKind, Target};
use crate::{Error, Update};

/// The newest manifest format this app understands. A manifest with a higher
/// `schema` is ignored, so an old app never misreads a new format.
pub const SCHEMA: u32 = 1;

/// The update manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub schema: u32,
    pub version: semver::Version,
    #[serde(default)]
    pub pub_date: Option<String>,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub notes_url: Option<String>,
    /// Installers by asset key (`linux-x64-deb`, `macos-arm64-dmg`…).
    #[serde(default)]
    pub assets: BTreeMap<String, Asset>,
}

/// One downloadable installer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Asset {
    pub name: String,
    /// The copy on the website.
    pub url: String,
    /// The same file on GitHub Releases, tried when the website fails.
    #[serde(default)]
    pub mirror: Option<String>,
    pub size: u64,
    pub sha256: String,
    /// The full text of the `.minisig` file.
    pub signature: String,
}

impl Manifest {
    pub fn parse(json: &str) -> Result<Self, Error> {
        #[derive(Deserialize)]
        struct Probe {
            schema: u32,
        }
        let probe: Probe = serde_json::from_str(json).map_err(|e| Error::Manifest(e.to_string()))?;
        if probe.schema > SCHEMA {
            return Err(Error::Manifest(format!(
                "format {} is newer than this app understands ({SCHEMA})",
                probe.schema
            )));
        }
        let manifest: Self = serde_json::from_str(json).map_err(|e| Error::Manifest(e.to_string()))?;
        for (key, asset) in &manifest.assets {
            if !asset.url.starts_with("https://") {
                return Err(Error::Manifest(format!("{key} is not served over HTTPS")));
            }
            if asset.sha256.len() != 64 || !asset.sha256.chars().all(|c| c.is_ascii_hexdigit()) {
                return Err(Error::Manifest(format!("{key} has an invalid SHA-256")));
            }
        }
        Ok(manifest)
    }

    /// The update this manifest offers to an app at `current` on `target`
    /// installed as `kind`, or `None` when it is not newer.
    ///
    /// Prereleases are only offered to prereleases, so stable users never
    /// receive a beta by accident.
    pub fn update_for(
        &self,
        current: &semver::Version,
        target: Option<Target>,
        kind: Option<InstallKind>,
    ) -> Option<Update> {
        if self.version <= *current {
            return None;
        }
        if !self.version.pre.is_empty() && current.pre.is_empty() {
            return None;
        }
        let asset = match (target, kind) {
            (Some(target), Some(kind)) => self.assets.get(&target.asset_key(kind)).cloned(),
            _ => None,
        };
        Some(Update {
            version: self.version.clone(),
            notes: self.notes.clone(),
            notes_url: self.notes_url.clone(),
            asset,
            kind,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::{Arch, Os};

    fn manifest(version: &str) -> String {
        format!(
            r#"{{
              "schema": 1,
              "version": "{version}",
              "pub_date": "2026-11-12T10:00:00Z",
              "notes": "Faster listing.",
              "notes_url": "https://example.org/changelog/",
              "future_field": true,
              "assets": {{
                "linux-x64-deb": {{
                  "name": "Coffer-{version}-linux-x64.deb",
                  "url": "https://example.org/downloads/v{version}/Coffer-{version}-linux-x64.deb",
                  "mirror": "https://github.com/x/y/releases/download/v{version}/Coffer-{version}-linux-x64.deb",
                  "size": 1234,
                  "sha256": "{sha}",
                  "signature": "untrusted comment: x\nRUQ=\ntrusted comment: y\nAA=="
                }}
              }}
            }}"#,
            sha = "a".repeat(64)
        )
    }

    const LINUX: Option<Target> = Some(Target { os: Os::Linux, arch: Arch::X64 });

    #[test]
    fn offers_newer_versions_only() {
        let m = Manifest::parse(&manifest("1.2.0")).expect("parse");
        let older: semver::Version = "1.1.9".parse().expect("version");
        let same: semver::Version = "1.2.0".parse().expect("version");
        let update = m.update_for(&older, LINUX, Some(InstallKind::Deb)).expect("an update");
        assert_eq!(update.version.to_string(), "1.2.0");
        assert_eq!(update.asset.expect("asset").size, 1234);
        assert!(m.update_for(&same, LINUX, Some(InstallKind::Deb)).is_none());
    }

    #[test]
    fn no_asset_for_other_kinds_or_dev_builds() {
        let m = Manifest::parse(&manifest("1.2.0")).expect("parse");
        let older: semver::Version = "1.0.0".parse().expect("version");
        assert!(m.update_for(&older, LINUX, Some(InstallKind::Rpm)).expect("update").asset.is_none());
        assert!(m.update_for(&older, LINUX, None).expect("update").asset.is_none());
    }

    #[test]
    fn prereleases_reach_prereleases_only() {
        let m = Manifest::parse(&manifest("2.0.0-beta.1")).expect("parse");
        let stable: semver::Version = "1.0.0".parse().expect("version");
        let beta: semver::Version = "2.0.0-alpha.3".parse().expect("version");
        assert!(m.update_for(&stable, LINUX, None).is_none());
        assert!(m.update_for(&beta, LINUX, None).is_some());
    }

    #[test]
    fn refuses_future_schemas_and_bad_assets() {
        let future = manifest("1.0.0").replace("\"schema\": 1", "\"schema\": 2");
        assert!(Manifest::parse(&future).is_err());
        let http = manifest("1.0.0").replace("\"url\": \"https://", "\"url\": \"http://");
        assert!(Manifest::parse(&http).is_err());
        let sha = manifest("1.0.0").replace(&"a".repeat(64), "abc");
        assert!(Manifest::parse(&sha).is_err());
        assert!(Manifest::parse("<html>").is_err());
    }
}
