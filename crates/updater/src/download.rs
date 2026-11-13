//! Downloading an installer and proving it is the one we released.

use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use sha2::{Digest, Sha256};

use crate::Error;
use crate::manifest::Asset;

/// Download progress, reported as bytes received out of the expected total.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Progress {
    pub received: u64,
    pub total: u64,
}

/// Downloads `asset` into `dir` and verifies it, trying the website first and
/// the GitHub mirror second.
///
/// `public_key` is the base64 minisign key (normally [`crate::PUBLIC_KEY`]).
/// `version` is the version the manifest announced: the signature's trusted
/// comment must name it, so an old signed installer can't be replayed as new.
pub fn download(
    agent: &ureq::Agent,
    asset: &Asset,
    version: &semver::Version,
    dir: &Path,
    public_key: Option<&str>,
    cancel: &AtomicBool,
    mut on_progress: impl FnMut(Progress),
) -> Result<PathBuf, Error> {
    let public_key = public_key.ok_or(Error::NoPublicKey)?;
    // The name comes from the network: keep only a plain file name.
    let name = Path::new(&asset.name)
        .file_name()
        .filter(|n| *n == std::ffi::OsStr::new(&asset.name))
        .ok_or_else(|| Error::Manifest(format!("invalid file name {:?}", asset.name)))?;
    std::fs::create_dir_all(dir)?;
    let path = dir.join(name);
    let partial = dir.join(format!("{}.part", asset.name));

    let mut last_error = None;
    for url in std::iter::once(&asset.url).chain(asset.mirror.as_ref()) {
        match fetch(agent, url, asset.size, &partial, cancel, &mut on_progress) {
            Ok(()) => {
                last_error = None;
                break;
            }
            Err(Error::Cancelled) => {
                let _ = std::fs::remove_file(&partial);
                return Err(Error::Cancelled);
            }
            Err(error) => {
                tracing::warn!(%url, %error, "download failed");
                last_error = Some(error);
            }
        }
    }
    if let Some(error) = last_error {
        let _ = std::fs::remove_file(&partial);
        return Err(error);
    }
    if let Err(error) = verify(&partial, asset, version, public_key) {
        let _ = std::fs::remove_file(&partial);
        return Err(error);
    }
    std::fs::rename(&partial, &path)?;
    Ok(path)
}

fn fetch(
    agent: &ureq::Agent,
    url: &str,
    expected: u64,
    to: &Path,
    cancel: &AtomicBool,
    on_progress: &mut impl FnMut(Progress),
) -> Result<(), Error> {
    if !url.starts_with("https://") {
        return Err(Error::Manifest(format!("{url} is not HTTPS")));
    }
    let mut response = agent.get(url).call().map_err(|e| Error::Network(e.to_string()))?;
    let mut reader = response.body_mut().with_config().limit(expected + 1).reader();
    let mut file = File::create(to)?;
    let mut buffer = vec![0_u8; 64 * 1024];
    let mut received = 0_u64;
    loop {
        if cancel.load(Ordering::Relaxed) {
            return Err(Error::Cancelled);
        }
        let n = reader.read(&mut buffer).map_err(|e| Error::Network(e.to_string()))?;
        if n == 0 {
            break;
        }
        received += n as u64;
        if received > expected {
            return Err(Error::Integrity("the file is larger than announced".into()));
        }
        file.write_all(&buffer[..n])?;
        on_progress(Progress { received, total: expected });
    }
    file.sync_all()?;
    if received != expected {
        return Err(Error::Integrity(format!("received {received} of {expected} bytes")));
    }
    Ok(())
}

/// Checks the size, SHA-256 and minisign signature of a downloaded file.
pub fn verify(path: &Path, asset: &Asset, version: &semver::Version, public_key: &str) -> Result<(), Error> {
    let key = minisign_verify::PublicKey::from_base64(public_key).map_err(|_| Error::NoPublicKey)?;
    let signature = minisign_verify::Signature::decode(&asset.signature).map_err(|_| Error::Signature)?;
    let comment = signature.trusted_comment();
    let expected_comment = trusted_comment(&version.to_string(), &asset.name);
    if comment != expected_comment {
        return Err(Error::Integrity(format!("the signature is for {comment:?}, not {expected_comment:?}")));
    }
    let mut verifier = key.verify_stream(&signature).map_err(|_| Error::Signature)?;
    let mut sha = Sha256::new();
    let mut reader = BufReader::new(File::open(path)?);
    let mut buffer = vec![0_u8; 64 * 1024];
    let mut size = 0_u64;
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        size += n as u64;
        sha.update(&buffer[..n]);
        verifier.update(&buffer[..n]);
    }
    if size != asset.size {
        return Err(Error::Integrity(format!("size {size}, expected {}", asset.size)));
    }
    let digest = hex(&sha.finalize());
    if !digest.eq_ignore_ascii_case(&asset.sha256) {
        return Err(Error::Integrity("SHA-256 mismatch".into()));
    }
    verifier.finalize().map_err(|_| Error::Signature)
}

/// The trusted comment the release pipeline signs with (`cargo xtask sign`).
/// Binding the version and file name into the signature prevents replaying
/// an older, validly signed installer as an update.
pub fn trusted_comment(version: &str, name: &str) -> String {
    format!("Coffer {version} {name}")
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    bytes.iter().fold(String::with_capacity(bytes.len() * 2), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Signed {
        dir: tempfile::TempDir,
        path: PathBuf,
        asset: Asset,
        public_key: String,
    }

    fn signed(content: &[u8], version: &str, name: &str) -> Signed {
        let pair = minisign::KeyPair::generate_unencrypted_keypair().expect("keypair");
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join(name);
        std::fs::write(&path, content).expect("write");
        let signature = minisign::sign(
            Some(&pair.pk),
            &pair.sk,
            content,
            Some(&trusted_comment(version, name)),
            Some("signature from the Coffer release key"),
        )
        .expect("sign")
        .into_string();
        let asset = Asset {
            name: name.to_owned(),
            url: "https://example.org/x".into(),
            mirror: None,
            size: content.len() as u64,
            sha256: hex(&Sha256::digest(content)),
            signature,
        };
        Signed { dir, path, asset, public_key: pair.pk.to_base64() }
    }

    fn v(text: &str) -> semver::Version {
        text.parse().expect("version")
    }

    #[test]
    fn accepts_a_correctly_signed_file() {
        let s = signed(b"installer bytes", "1.2.3", "Coffer-1.2.3-linux-x64.deb");
        verify(&s.path, &s.asset, &v("1.2.3"), &s.public_key).expect("valid");
        drop(s.dir);
    }

    #[test]
    fn rejects_tampering() {
        let s = signed(b"installer bytes", "1.2.3", "a.deb");
        std::fs::write(&s.path, b"installer byteZ").expect("write");
        let err = verify(&s.path, &s.asset, &v("1.2.3"), &s.public_key).expect_err("tampered");
        assert!(matches!(err, Error::Integrity(_)), "{err:?}");
    }

    #[test]
    fn rejects_a_matching_hash_with_a_foreign_signature() {
        let s = signed(b"installer bytes", "1.2.3", "a.deb");
        let other = signed(b"installer bytes", "1.2.3", "a.deb");
        let err = verify(&s.path, &s.asset, &v("1.2.3"), &other.public_key).expect_err("wrong key");
        assert!(matches!(err, Error::Signature), "{err:?}");
    }

    #[test]
    fn rejects_replayed_old_versions() {
        let s = signed(b"old installer", "1.0.0", "a.deb");
        let err = verify(&s.path, &s.asset, &v("2.0.0"), &s.public_key).expect_err("replay");
        assert!(matches!(err, Error::Integrity(_)), "{err:?}");
    }

    #[test]
    fn refuses_without_a_key_and_with_bad_names() {
        let s = signed(b"x", "1.0.0", "a.deb");
        let agent = crate::agent();
        let cancel = AtomicBool::new(false);
        let err = download(&agent, &s.asset, &v("1.0.0"), s.dir.path(), None, &cancel, |_| {});
        assert!(matches!(err, Err(Error::NoPublicKey)));
        let mut evil = s.asset.clone();
        evil.name = "../../etc/passwd".into();
        let err = download(&agent, &evil, &v("1.0.0"), s.dir.path(), Some(&s.public_key), &cancel, |_| {});
        assert!(matches!(err, Err(Error::Manifest(_))));
    }
}
