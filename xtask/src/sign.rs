//! The update-signing key: creating it, checking it, and signing installers.
//!
//! The secret key lives only in the `UPDATE_SIGNING_KEY` secret (with
//! `UPDATE_SIGNING_PASSWORD` when encrypted). The public key is committed in
//! `packaging/keys/update.pub` and compiled into the app, which installs only
//! files signed by it.

use std::io::Cursor;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail, ensure};

const PUBLIC_KEY: &str = "packaging/keys/update.pub";

/// Must stay identical to `coffer_updater::download::trusted_comment`.
pub fn trusted_comment(version: &str, name: &str) -> String {
    format!("Coffer {version} {name}")
}

fn secret_key() -> Result<minisign::SecretKey> {
    let text = std::env::var("UPDATE_SIGNING_KEY")
        .ok()
        .filter(|k| !k.trim().is_empty())
        .context("UPDATE_SIGNING_KEY is not set: run scripts/setup-signing.sh to create the release keys")?;
    let password = std::env::var("UPDATE_SIGNING_PASSWORD").ok().filter(|p| !p.is_empty());
    let boxed =
        minisign::SecretKeyBox::from_string(&text).context("UPDATE_SIGNING_KEY is not a minisign secret key")?;
    match password {
        Some(password) => boxed.into_secret_key(Some(password)),
        None => boxed.into_unencrypted_secret_key(),
    }
    .context("could not unlock UPDATE_SIGNING_KEY (wrong UPDATE_SIGNING_PASSWORD?)")
}

fn public_key(root: &Path) -> Result<minisign::PublicKey> {
    let path = root.join(PUBLIC_KEY);
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("{PUBLIC_KEY} is missing: run scripts/setup-signing.sh and commit it"))?;
    let boxed = minisign::PublicKeyBox::from_string(&text).context("the public key file is invalid")?;
    boxed.into_public_key().context("the public key is invalid")
}

/// Fails unless the secret signs what the committed public key verifies.
pub fn check(root: &Path) -> Result<()> {
    let secret = secret_key()?;
    let public = public_key(root)?;
    let data = b"Coffer signing check";
    let signature = minisign::sign(None, &secret, Cursor::new(data), Some("check"), None)?;
    minisign::verify(&public, &signature, Cursor::new(data), true, false, false)
        .context("UPDATE_SIGNING_KEY does not match packaging/keys/update.pub")?;
    eprintln!("The update signing key matches {PUBLIC_KEY}.");
    Ok(())
}

pub fn sign_files(version: &str, files: &[PathBuf]) -> Result<()> {
    ensure!(!files.is_empty(), "no files to sign");
    let secret = secret_key()?;
    for file in files {
        let name = file.file_name().and_then(|n| n.to_str()).context("file without a name")?;
        let data = std::fs::File::open(file).with_context(|| format!("can't read {}", file.display()))?;
        let signature = minisign::sign(
            None,
            &secret,
            std::io::BufReader::new(data),
            Some(&trusted_comment(version, name)),
            Some("signature from the Coffer release key"),
        )?;
        let out = PathBuf::from(format!("{}.minisig", file.display()));
        std::fs::write(&out, signature.into_string())?;
        eprintln!("signed {}", file.display());
    }
    Ok(())
}

pub fn keygen(root: &Path, secret_out: &Path, encrypted: bool) -> Result<()> {
    if secret_out.starts_with(root) {
        bail!("write the secret key outside the repository");
    }
    let pair = if encrypted {
        let password =
            std::env::var("UPDATE_SIGNING_PASSWORD").context("set UPDATE_SIGNING_PASSWORD to encrypt the key")?;
        minisign::KeyPair::generate_encrypted_keypair(Some(password))?
    } else {
        minisign::KeyPair::generate_unencrypted_keypair()?
    };
    let public = pair.pk.to_box()?.into_string();
    let secret = pair.sk.to_box(Some("Coffer update signing key"))?.into_string();
    std::fs::create_dir_all(root.join("packaging/keys"))?;
    std::fs::write(root.join(PUBLIC_KEY), &public)?;
    std::fs::write(secret_out, secret)?;
    eprintln!(
        "Wrote {PUBLIC_KEY} (commit it) and {} (store it as the UPDATE_SIGNING_KEY secret, then delete it).",
        secret_out.display()
    );
    Ok(())
}
