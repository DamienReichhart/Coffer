//! Compiles the update-signing public key into the updater.
//!
//! The key lives in `packaging/keys/update.pub` (written by
//! `scripts/setup-signing.sh`). Without it the build still works, but the
//! updater refuses to install anything: it can only point at the download page.

use std::path::Path;

fn main() {
    let key = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packaging/keys/update.pub");
    println!("cargo:rerun-if-changed={}", key.display());
    println!("cargo:rerun-if-env-changed=SEVENZIP_GUI_UPDATE_KEY");
    println!("cargo:rerun-if-env-changed=SEVENZIP_GUI_REQUIRE_UPDATE_KEY");
    let contents = std::env::var("SEVENZIP_GUI_UPDATE_KEY")
        .ok()
        .or_else(|| std::fs::read_to_string(&key).ok())
        .map(|text| {
            // The base64 line of a minisign public key file (the other is a comment).
            text.lines()
                .map(str::trim)
                .find(|line| !line.is_empty() && !line.starts_with("untrusted comment:"))
                .unwrap_or_default()
                .to_owned()
        })
        .filter(|line| !line.is_empty());
    // Release builds set this, so an installer can never ship unable to update.
    assert!(
        contents.is_some()
            || !std::env::var("SEVENZIP_GUI_REQUIRE_UPDATE_KEY").is_ok_and(|v| !v.is_empty() && v != "0"),
        "SEVENZIP_GUI_REQUIRE_UPDATE_KEY is set but packaging/keys/update.pub is missing: run scripts/setup-signing.sh"
    );
    let out = Path::new(&std::env::var("OUT_DIR").expect("OUT_DIR")).join("update_key.rs");
    let code = match contents {
        Some(line) => format!("pub const PUBLIC_KEY: Option<&str> = Some({line:?});\n"),
        None => "pub const PUBLIC_KEY: Option<&str> = None;\n".to_owned(),
    };
    std::fs::write(out, code).expect("write update_key.rs");
}
