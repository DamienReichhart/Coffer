//! Embeds the icon and version information into the Windows executable.

fn main() {
    let ico = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packaging/windows/coffer.ico");
    println!("cargo:rerun-if-changed={}", ico.display());
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    if ico.exists() {
        res.set_icon(&ico.to_string_lossy());
    }
    res.set("ProductName", "Coffer")
        .set("FileDescription", "Coffer — a desktop interface for 7-Zip")
        .set("CompanyName", "Damien Reichhart")
        .set("LegalCopyright", "© Damien Reichhart. MIT licence.")
        .set("OriginalFilename", "coffer.exe")
        .set("InternalName", "coffer");
    if let Err(error) = res.compile() {
        // A missing resource compiler must not break development builds;
        // release builds check the icon in `cargo xtask package`.
        println!("cargo:warning=could not embed Windows resources: {error}");
    }
}
