//! `cargo xtask package`: turns a release binary into installers named after
//! the release contract (`docs/ARCHITECTURE.md`):
//! `7zip-GUI-<version>-<os>-<arch>.<ext>`.
//!
//! | Target OS | Formats | Tools |
//! | --- | --- | --- |
//! | Linux | deb, rpm, AppImage | `dpkg-deb`, `rpmbuild`, `appimagetool` (`$APPIMAGETOOL`) |
//! | Windows | msi, portable zip | WiX 5 (`wix`) |
//! | macOS | dmg | `codesign`, `hdiutil` (and `xcrun notarytool` when Apple credentials are set) |

use std::fmt::Write as _;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail, ensure};

use crate::{run as exec, workspace_version};

pub const PRODUCT: &str = "7zip-GUI";
pub const BINARY: &str = "7zip-gui";
pub const APP_ID: &str = "io.github.damienreichhart.SevenZipGui";
const SUMMARY: &str = "A modern desktop interface for 7-Zip";
const DESCRIPTION: &str = "7zip-GUI opens, creates, extracts, tests and checksums archives with the 7-Zip \
command-line program installed on the computer. 7-Zip itself is not included.";
const HOMEPAGE: &str = "https://damienreichhart.github.io/7zip-GUI/";
const MAINTAINER: &str = "Damien Reichhart <damienreichhart@users.noreply.github.com>";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Os {
    Linux,
    Windows,
    Macos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Arch {
    X64,
    Arm64,
}

struct Target {
    os: Os,
    arch: Arch,
}

impl Target {
    fn parse(triple: &str) -> Result<Self> {
        let arch = if triple.starts_with("x86_64") {
            Arch::X64
        } else if triple.starts_with("aarch64") {
            Arch::Arm64
        } else {
            bail!("unsupported architecture in {triple}");
        };
        let os = if triple.contains("linux") {
            Os::Linux
        } else if triple.contains("windows") {
            Os::Windows
        } else if triple.contains("apple-darwin") {
            Os::Macos
        } else {
            bail!("unsupported OS in {triple}");
        };
        Ok(Self { os, arch })
    }

    fn os_name(&self) -> &'static str {
        match self.os {
            Os::Linux => "linux",
            Os::Windows => "windows",
            Os::Macos => "macos",
        }
    }

    fn arch_name(&self) -> &'static str {
        match self.arch {
            Arch::X64 => "x64",
            Arch::Arm64 => "arm64",
        }
    }

    fn default_formats(&self) -> &'static [&'static str] {
        match self.os {
            Os::Linux => &["deb", "rpm", "appimage"],
            Os::Windows => &["msi", "portable"],
            Os::Macos => &["dmg"],
        }
    }
}

struct Ctx<'a> {
    root: &'a Path,
    target: Target,
    version: semver::Version,
    binary: PathBuf,
    out: PathBuf,
    work: PathBuf,
}

impl Ctx<'_> {
    fn asset(&self, ext: &str) -> PathBuf {
        self.out.join(format!("{PRODUCT}-{}-{}-{}.{ext}", self.version, self.target.os_name(), self.target.arch_name()))
    }
}

pub fn run(root: &Path, triple: &str, formats: &[String], out: &Path, binary: Option<PathBuf>) -> Result<()> {
    let target = Target::parse(triple)?;
    let exe = if target.os == Os::Windows { format!("{BINARY}.exe") } else { BINARY.to_owned() };
    let binary = binary.unwrap_or_else(|| root.join("target").join(triple).join("release").join(&exe));
    ensure!(
        binary.is_file(),
        "{} does not exist: build it first with `cargo build --release --locked --target {triple} -p sevenzip-gui`",
        binary.display()
    );
    let formats: Vec<String> = if formats.is_empty() {
        target.default_formats().iter().map(|s| (*s).to_owned()).collect()
    } else {
        formats.to_vec()
    };
    std::fs::create_dir_all(out)?;
    let out = std::path::absolute(out)?;
    let work = root.join("target").join("package").join(triple);
    if work.exists() {
        std::fs::remove_dir_all(&work)?;
    }
    std::fs::create_dir_all(&work)?;
    let ctx = Ctx { root, target, version: workspace_version(root)?, binary, out, work };

    for format in &formats {
        let path = match (ctx.target.os, format.as_str()) {
            (Os::Linux, "deb") => deb(&ctx)?,
            (Os::Linux, "rpm") => rpm(&ctx)?,
            (Os::Linux, "appimage") => appimage(&ctx)?,
            (Os::Windows, "msi") => msi(&ctx)?,
            (Os::Windows, "portable") => portable(&ctx)?,
            (Os::Macos, "dmg") => dmg(&ctx)?,
            (_, other) => bail!("{other} can't be built for {triple}"),
        };
        let size = std::fs::metadata(&path)?.len();
        eprintln!("built {} ({:.1} MiB)", path.display(), size as f64 / 1_048_576.0);
    }
    Ok(())
}

// Linux ----------------------------------------------------------------------

/// Lays out the files of a Linux package under `root` (`/usr/...`).
fn linux_tree(ctx: &Ctx<'_>, root: &Path, kind: &str) -> Result<()> {
    let usr = root.join("usr");
    install(&ctx.binary, &usr.join("bin").join(BINARY), 0o755)?;
    write(&usr.join("lib").join(BINARY).join("install-kind"), format!("{kind}\n"), 0o644)?;
    let packaging = ctx.root.join("packaging/linux");
    install(
        &packaging.join(format!("{APP_ID}.desktop")),
        &usr.join("share/applications").join(format!("{APP_ID}.desktop")),
        0o644,
    )?;
    let metainfo = std::fs::read_to_string(packaging.join(format!("{APP_ID}.metainfo.xml")))?
        .replace("@VERSION@", &ctx.version.to_string())
        .replace("@DATE@", &release_date(ctx.root, &ctx.version));
    write(&usr.join("share/metainfo").join(format!("{APP_ID}.metainfo.xml")), metainfo, 0o644)?;
    for size in [16, 24, 32, 48, 64, 128, 256, 512] {
        let dest = usr.join(format!("share/icons/hicolor/{size}x{size}/apps/{APP_ID}.png"));
        install(&packaging.join(format!("icons/{size}.png")), &dest, 0o644)?;
    }
    install(
        &packaging.join("icons/scalable.svg"),
        &usr.join(format!("share/icons/hicolor/scalable/apps/{APP_ID}.svg")),
        0o644,
    )?;
    let license = std::fs::read_to_string(ctx.root.join("LICENSE"))?;
    write(
        &usr.join("share/doc").join(BINARY).join("copyright"),
        format!(
            "Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/\nUpstream-Name: {PRODUCT}\nSource: https://github.com/DamienReichhart/7zip-GUI\n\nFiles: *\nCopyright: Damien Reichhart\nLicense: MIT\n{}\n",
            license
                .lines()
                .map(|l| if l.is_empty() { " .".to_owned() } else { format!(" {l}") })
                .collect::<Vec<_>>()
                .join("\n")
        ),
        0o644,
    )?;
    Ok(())
}

fn deb(ctx: &Ctx<'_>) -> Result<PathBuf> {
    let root = ctx.work.join("deb");
    linux_tree(ctx, &root, "deb")?;
    let arch = match ctx.target.arch {
        Arch::X64 => "amd64",
        Arch::Arm64 => "arm64",
    };
    // Debian sorts `~` before anything, so 1.0.0~beta.1 < 1.0.0.
    let version = ctx.version.to_string().replacen('-', "~", 1);
    let installed_kb = dir_size(&root.join("usr"))? / 1024;
    let control = format!(
        "Package: {BINARY}\nVersion: {version}\nArchitecture: {arch}\nMaintainer: {MAINTAINER}\nInstalled-Size: {installed_kb}\n\
         Depends: libc6 (>= 2.35), libgcc-s1, libxkbcommon0\nRecommends: 7zip | p7zip-full, libxkbcommon-x11-0, libvulkan1 | libegl1, xdg-desktop-portal\n\
         Section: utils\nPriority: optional\nHomepage: {HOMEPAGE}\nDescription: {SUMMARY}\n {}\n",
        DESCRIPTION.split(". ").collect::<Vec<_>>().join(".\n ")
    );
    write(&root.join("DEBIAN/control"), control, 0o644)?;
    let out = ctx.asset("deb");
    exec(Command::new("dpkg-deb").args(["--root-owner-group", "-Zxz", "--build"]).arg(&root).arg(&out))?;
    Ok(out)
}

fn rpm(ctx: &Ctx<'_>) -> Result<PathBuf> {
    let top = ctx.work.join("rpmbuild");
    let staged = ctx.work.join("rpm-root");
    linux_tree(ctx, &staged, "rpm")?;
    let arch = match ctx.target.arch {
        Arch::X64 => "x86_64",
        Arch::Arm64 => "aarch64",
    };
    let (version, release) = if ctx.version.pre.is_empty() {
        (ctx.version.to_string(), "1".to_owned())
    } else {
        (
            format!("{}.{}.{}", ctx.version.major, ctx.version.minor, ctx.version.patch),
            format!("0.{}", ctx.version.pre.as_str().replace('-', ".")),
        )
    };
    let mut files = String::new();
    for entry in walk(&staged)? {
        if entry.is_file() {
            let rel = entry.strip_prefix(&staged)?;
            let _ = writeln!(files, "/{}", rel.display());
        }
    }
    let spec = format!(
        "Name: {BINARY}\nVersion: {version}\nRelease: {release}\nSummary: {SUMMARY}\nLicense: MIT\nURL: {HOMEPAGE}\n\
         BuildArch: {arch}\nRequires: libxkbcommon\nRecommends: 7zip\nRecommends: xdg-desktop-portal\n\
         %define _build_id_links none\n%define __strip /bin/true\n%define _binary_payload w19.zstdio\n\n\
         %description\n{DESCRIPTION}\n\n%install\ncp -a {staged}/. %{{buildroot}}/\n\n%files\n{files}\n",
        staged = staged.display()
    );
    let spec_path = top.join("SPECS").join(format!("{BINARY}.spec"));
    write(&spec_path, spec, 0o644)?;
    exec(
        Command::new("rpmbuild")
            .arg("-bb")
            .arg("--define")
            .arg(format!("_topdir {}", top.display()))
            .arg("--target")
            .arg(arch)
            .arg(&spec_path),
    )?;
    let built = walk(&top.join("RPMS"))?
        .into_iter()
        .find(|p| p.extension().is_some_and(|e| e == "rpm"))
        .context("rpmbuild produced no package")?;
    let out = ctx.asset("rpm");
    std::fs::copy(built, &out)?;
    Ok(out)
}

fn appimage(ctx: &Ctx<'_>) -> Result<PathBuf> {
    let tool = std::env::var("APPIMAGETOOL").unwrap_or_else(|_| "appimagetool".to_owned());
    let dir = ctx.work.join("AppDir");
    linux_tree(ctx, &dir, "appimage")?;
    // The marker is for system packages; an AppImage is found by $APPIMAGE.
    std::fs::remove_dir_all(dir.join("usr/lib"))?;
    let packaging = ctx.root.join("packaging/linux");
    std::fs::copy(packaging.join(format!("{APP_ID}.desktop")), dir.join(format!("{APP_ID}.desktop")))?;
    std::fs::copy(packaging.join("icons/256.png"), dir.join(format!("{APP_ID}.png")))?;
    std::fs::copy(packaging.join("icons/256.png"), dir.join(".DirIcon"))?;
    write(
        &dir.join("AppRun"),
        format!("#!/bin/sh\nhere=\"$(dirname \"$(readlink -f \"$0\")\")\"\nexec \"$here/usr/bin/{BINARY}\" \"$@\"\n"),
        0o755,
    )?;
    let arch = match ctx.target.arch {
        Arch::X64 => "x86_64",
        Arch::Arm64 => "aarch64",
    };
    let out = ctx.asset("AppImage");
    exec(
        Command::new(tool)
            .env("ARCH", arch)
            .env("APPIMAGE_EXTRACT_AND_RUN", "1")
            .arg("--no-appstream")
            .arg(&dir)
            .arg(&out),
    )?;
    Ok(out)
}

// Windows --------------------------------------------------------------------

fn msi(ctx: &Ctx<'_>) -> Result<PathBuf> {
    let stage = ctx.work.join("msi");
    std::fs::create_dir_all(&stage)?;
    write(&stage.join("install-kind"), "msi\n", 0o644)?;
    write(&stage.join("License.rtf"), rtf(&std::fs::read_to_string(ctx.root.join("LICENSE"))?), 0o644)?;
    let arch = match ctx.target.arch {
        Arch::X64 => "x64",
        Arch::Arm64 => "arm64",
    };
    // MSI versions are numeric: a prerelease installs as its base version.
    let version = format!("{}.{}.{}", ctx.version.major, ctx.version.minor, ctx.version.patch);
    let out = ctx.asset("msi");
    exec(
        Command::new("wix")
            .args(["build", "-nologo", "-arch", arch, "-ext", "WixToolset.UI.wixext"])
            .arg("-d")
            .arg(format!("Version={version}"))
            .arg("-d")
            .arg(format!("Exe={}", ctx.binary.display()))
            .arg("-d")
            .arg(format!("Stage={}", stage.display()))
            .arg("-d")
            .arg(format!("Packaging={}", ctx.root.join("packaging/windows").display()))
            .arg("-o")
            .arg(&out)
            .arg(ctx.root.join("packaging/windows/7zip-gui.wxs")),
    )?;
    Ok(out)
}

fn portable(ctx: &Ctx<'_>) -> Result<PathBuf> {
    let out = ctx.asset("zip");
    let file = std::fs::File::create(&out)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let folder = format!("{PRODUCT}-{}", ctx.version);
    zip.start_file(format!("{folder}/{BINARY}.exe"), options)?;
    zip.write_all(&std::fs::read(&ctx.binary)?)?;
    zip.start_file(format!("{folder}/install-kind"), options)?;
    zip.write_all(b"portable\n")?;
    zip.start_file(format!("{folder}/LICENSE.txt"), options)?;
    zip.write_all(std::fs::read_to_string(ctx.root.join("LICENSE"))?.replace('\n', "\r\n").as_bytes())?;
    zip.start_file(format!("{folder}/README.txt"), options)?;
    zip.write_all(
        format!(
            "{PRODUCT} {} — portable edition\r\n\r\nRun {BINARY}.exe; nothing is installed. Settings live in %APPDATA%\\DamienReichhart\\7zip-GUI.\r\n\
             7-Zip must be installed separately: winget install --id 7zip.7zip -e\r\n\r\nUpdates replace {BINARY}.exe in place.\r\n{HOMEPAGE}\r\n",
            ctx.version
        )
        .as_bytes(),
    )?;
    zip.finish()?;
    Ok(out)
}

/// The licence as a minimal RTF document for the MSI's licence page.
fn rtf(text: &str) -> String {
    let mut out = String::from("{\\rtf1\\ansi\\deff0{\\fonttbl{\\f0 Segoe UI;}}\\f0\\fs18 ");
    for c in text.chars() {
        match c {
            '\\' | '{' | '}' => {
                out.push('\\');
                out.push(c);
            }
            '\n' => out.push_str("\\par\n"),
            c if c.is_ascii() => out.push(c),
            c => {
                let _ = write!(out, "\\u{}?", c as i32 as i16);
            }
        }
    }
    out.push('}');
    out
}

// macOS ----------------------------------------------------------------------

#[cfg(not(unix))]
fn dmg(_: &Ctx<'_>) -> Result<PathBuf> {
    bail!("disk images are built on macOS")
}

#[cfg(unix)]
fn dmg(ctx: &Ctx<'_>) -> Result<PathBuf> {
    let stage = ctx.work.join("dmg");
    let app = stage.join(format!("{PRODUCT}.app"));
    let contents = app.join("Contents");
    install(&ctx.binary, &contents.join("MacOS").join(BINARY), 0o755)?;
    write(&contents.join("MacOS/install-kind"), "dmg\n", 0o644)?;
    install(&ctx.root.join("packaging/macos/AppIcon.icns"), &contents.join("Resources/AppIcon.icns"), 0o644)?;
    let plist = std::fs::read_to_string(ctx.root.join("packaging/macos/Info.plist"))?
        .replace("@VERSION@", &ctx.version.to_string())
        .replace("@SHORT_VERSION@", &format!("{}.{}.{}", ctx.version.major, ctx.version.minor, ctx.version.patch))
        .replace(
            "@MIN_MACOS@",
            match ctx.target.arch {
                Arch::Arm64 => "11.0",
                Arch::X64 => "10.15",
            },
        );
    write(&contents.join("Info.plist"), plist, 0o644)?;
    write(&contents.join("PkgInfo"), "APPL????", 0o644)?;

    let identity = std::env::var("APPLE_SIGNING_IDENTITY").ok().filter(|s| !s.is_empty());
    let entitlements = ctx.root.join("packaging/macos/entitlements.plist");
    let mut codesign = Command::new("codesign");
    codesign.args(["--force", "--timestamp", "--options", "runtime", "--entitlements"]).arg(&entitlements);
    match &identity {
        Some(identity) => codesign.args(["--sign", identity]),
        // Ad-hoc: Apple silicon refuses unsigned code; Gatekeeper still warns.
        None => codesign.args(["--sign", "-"]),
    };
    exec(codesign.arg(&app))?;
    exec(Command::new("codesign").args(["--verify", "--strict", "--verbose=2"]).arg(&app))?;

    std::os::unix::fs::symlink("/Applications", stage.join("Applications")).context("Applications link")?;
    let out = ctx.asset("dmg");
    if out.exists() {
        std::fs::remove_file(&out)?;
    }
    exec(
        Command::new("hdiutil")
            .args([
                "create",
                "-volname",
                PRODUCT,
                "-fs",
                "HFS+",
                "-format",
                "UDZO",
                "-imagekey",
                "zlib-level=9",
                "-ov",
                "-srcfolder",
            ])
            .arg(&stage)
            .arg(&out),
    )?;

    if identity.is_some() {
        notarize(&out)?;
    }
    Ok(out)
}

/// Notarizes and staples the disk image when Apple credentials are set.
#[cfg(unix)]
fn notarize(dmg: &Path) -> Result<()> {
    let (Ok(id), Ok(password), Ok(team)) =
        (std::env::var("APPLE_ID"), std::env::var("APPLE_PASSWORD"), std::env::var("APPLE_TEAM_ID"))
    else {
        eprintln!("APPLE_ID, APPLE_PASSWORD or APPLE_TEAM_ID is not set: the disk image is signed but not notarized.");
        return Ok(());
    };
    exec(
        Command::new("codesign")
            .args(["--force", "--timestamp", "--sign"])
            .arg(std::env::var("APPLE_SIGNING_IDENTITY")?)
            .arg(dmg),
    )?;
    exec(
        Command::new("xcrun")
            .args(["notarytool", "submit", "--wait", "--apple-id", &id, "--password", &password, "--team-id", &team])
            .arg(dmg),
    )?;
    exec(Command::new("xcrun").args(["stapler", "staple"]).arg(dmg))
}

// Helpers --------------------------------------------------------------------

fn release_date(root: &Path, version: &semver::Version) -> String {
    std::fs::read_to_string(root.join("CHANGELOG.md"))
        .ok()
        .and_then(|text| crate::changelog::section(&text, &version.to_string()))
        .and_then(|s| s.date)
        .unwrap_or_else(|| "2026-01-01".to_owned())
}

fn install(from: &Path, to: &Path, mode: u32) -> Result<()> {
    if let Some(dir) = to.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::copy(from, to).with_context(|| format!("copy {} to {}", from.display(), to.display()))?;
    set_mode(to, mode)
}

fn write(to: &Path, contents: impl AsRef<[u8]>, mode: u32) -> Result<()> {
    if let Some(dir) = to.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(to, contents)?;
    set_mode(to, mode)
}

#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))?;
    Ok(())
}

#[cfg(not(unix))]
#[allow(clippy::unnecessary_wraps)] // same signature as the Unix version
fn set_mode(_: &Path, _: u32) -> Result<()> {
    Ok(())
}

fn walk(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut found = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in std::fs::read_dir(&d)? {
            let path = entry?.path();
            if path.is_dir() {
                stack.push(path.clone());
            }
            found.push(path);
        }
    }
    found.sort();
    Ok(found)
}

fn dir_size(dir: &Path) -> Result<u64> {
    Ok(walk(dir)?.iter().filter(|p| p.is_file()).map(|p| std::fs::metadata(p).map_or(0, |m| m.len())).sum())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn targets() {
        let t = Target::parse("aarch64-unknown-linux-gnu").expect("linux");
        assert_eq!((t.os_name(), t.arch_name()), ("linux", "arm64"));
        let t = Target::parse("x86_64-pc-windows-msvc").expect("windows");
        assert_eq!((t.os_name(), t.arch_name()), ("windows", "x64"));
        let t = Target::parse("aarch64-apple-darwin").expect("macos");
        assert_eq!(t.default_formats(), ["dmg"]);
        assert!(Target::parse("riscv64gc-unknown-linux-gnu").is_err());
    }

    #[test]
    fn rtf_escapes() {
        assert_eq!(
            rtf("a{b}\\\né"),
            "{\\rtf1\\ansi\\deff0{\\fonttbl{\\f0 Segoe UI;}}\\f0\\fs18 a\\{b\\}\\\\\\par\n\\u233?}"
        );
    }
}
