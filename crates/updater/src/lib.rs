//! Automatic updates for 7zip-GUI.
//!
//! 1. [`check`] fetches the update manifest the website publishes
//!    (`updates/latest.json`) and decides whether it describes a newer version
//!    with an installer for this platform and install kind.
//! 2. [`download()`] fetches that installer from the website (falling back to the
//!    GitHub mirror) and verifies its size, SHA-256 and minisign signature
//!    against the key compiled into the app.
//! 3. [`install()`] hands the verified file to the platform's installer and tells
//!    the app to quit, so the new version starts in its place.
//!
//! Nothing here is UI code; the app runs these on a background thread.

pub mod download;
pub mod install;
pub mod manifest;
pub mod platform;

pub use download::{Progress, download};
pub use install::{Handover, install};
pub use manifest::{Asset, Manifest};
pub use platform::{Arch, InstallKind, Os, Target};

use std::time::Duration;

mod key {
    include!(concat!(env!("OUT_DIR"), "/update_key.rs"));
}

/// The minisign public key releases are signed with, when this build has one.
pub const PUBLIC_KEY: Option<&str> = key::PUBLIC_KEY;

/// Where the website publishes the manifest. `SEVENZIP_GUI_UPDATE_URL`
/// overrides it, for testing against a local server.
pub const DEFAULT_MANIFEST_URL: &str = "https://damienreichhart.github.io/7zip-GUI/updates/latest.json";

/// The manifest URL to use.
pub fn manifest_url() -> String {
    std::env::var("SEVENZIP_GUI_UPDATE_URL").unwrap_or_else(|_| DEFAULT_MANIFEST_URL.to_owned())
}

/// Errors of the update process.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not reach the update server: {0}")]
    Network(String),
    #[error("the update information is invalid: {0}")]
    Manifest(String),
    #[error("the download is corrupt: {0}")]
    Integrity(String),
    #[error("the update is not signed by the 7zip-GUI release key")]
    Signature,
    #[error("this build has no update signing key, so it can't install updates")]
    NoPublicKey,
    #[error("there is no {0} installer in this release")]
    NoAsset(String),
    #[error("could not start the installer: {0}")]
    Install(String),
    #[error("cancelled")]
    Cancelled,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// A newer version than the running one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Update {
    pub version: semver::Version,
    pub notes: String,
    pub notes_url: Option<String>,
    /// The installer for this platform and install kind, when there is one
    /// and this build can verify it. Without one the app links to the website.
    pub asset: Option<Asset>,
    pub kind: Option<InstallKind>,
}

/// The HTTP client used for checks and downloads: the platform's trust store
/// (so corporate TLS inspection works), a user agent, and timeouts.
pub fn agent() -> ureq::Agent {
    let tls = ureq::tls::TlsConfig::builder().root_certs(ureq::tls::RootCerts::PlatformVerifier).build();
    ureq::Agent::config_builder()
        .tls_config(tls)
        .user_agent(concat!("7zip-GUI/", env!("CARGO_PKG_VERSION"), " (+https://github.com/DamienReichhart/7zip-GUI)"))
        .timeout_connect(Some(Duration::from_secs(15)))
        // Bodies are installers of tens of megabytes on slow links.
        .timeout_recv_body(Some(Duration::from_mins(30)))
        .build()
        .into()
}

/// Fetches the manifest and returns the update it offers, if any.
///
/// `current` is the running version. Versions the user chose to skip are the
/// caller's business.
pub fn check(agent: &ureq::Agent, url: &str, current: &semver::Version) -> Result<Option<Update>, Error> {
    let body = agent
        .get(url)
        .call()
        .map_err(|e| Error::Network(e.to_string()))?
        .body_mut()
        .with_config()
        .limit(1024 * 1024)
        .read_to_string()
        .map_err(|e| Error::Network(e.to_string()))?;
    let manifest = Manifest::parse(&body)?;
    Ok(manifest.update_for(current, Target::current(), InstallKind::detect()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_url_is_on_the_website() {
        assert!(DEFAULT_MANIFEST_URL.starts_with("https://"));
        assert!(DEFAULT_MANIFEST_URL.ends_with("/updates/latest.json"));
    }
}
