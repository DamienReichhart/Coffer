//! Handing a verified installer to the platform, then restarting the app.
//!
//! | Kind | What happens |
//! | --- | --- |
//! | `msi` | A helper script waits for the app to quit, runs `msiexec /passive`, and starts the new version. |
//! | `portable` | The new executable replaces the running one in place; the app restarts. |
//! | `dmg` | A helper script waits for the app to quit, copies the new `.app` over the old one, and opens it. |
//! | `deb`, `rpm` | `pkexec` installs the package with apt, dnf, zypper or rpm; the app restarts. |
//! | `appimage` | The new AppImage replaces the old file; the app restarts. |

use std::path::Path;
#[cfg(any(unix, windows))]
use std::process::{Command, Stdio};

use crate::Error;
use crate::platform::InstallKind;

/// What the app must do once the installer has been started.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Handover {
    /// Quit now: the installer, or the restarted new version, takes over.
    pub quit: bool,
}

/// Installs the verified file `installer` of the given kind. `exe` is the path
/// of the running executable as it was at startup.
pub fn install(kind: InstallKind, installer: &Path, exe: &Path) -> Result<Handover, Error> {
    tracing::info!(?kind, installer = %installer.display(), "installing update");
    match kind {
        InstallKind::Msi => msi(installer, exe),
        InstallKind::Portable => portable(installer, exe),
        InstallKind::Dmg => dmg(installer, exe),
        InstallKind::Deb | InstallKind::Rpm => linux_package(kind, installer, exe),
        InstallKind::AppImage => appimage(installer),
    }
}

#[cfg(any(unix, windows))]
fn detach(command: &mut Command) -> Result<(), Error> {
    command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        const DETACHED_PROCESS: u32 = 0x0000_0008;
        command.creation_flags(CREATE_NO_WINDOW | DETACHED_PROCESS);
    }
    command.spawn().map(drop).map_err(|e| Error::Install(e.to_string()))
}

#[cfg(windows)]
fn msi(installer: &Path, exe: &Path) -> Result<Handover, Error> {
    // A batch file waits for msiexec, so the new version starts after it.
    let script = installer.with_extension("cmd");
    let body = format!(
        "@echo off\r\n\
         ping -n 3 127.0.0.1 >nul\r\n\
         start \"\" /wait msiexec.exe /i \"{msi}\" /passive /norestart\r\n\
         if %ERRORLEVEL% EQU 0 start \"\" \"{exe}\"\r\n\
         if %ERRORLEVEL% EQU 3010 start \"\" \"{exe}\"\r\n\
         del \"%~f0\"\r\n",
        msi = installer.display(),
        exe = exe.display(),
    );
    std::fs::write(&script, body)?;
    detach(Command::new("cmd.exe").arg("/C").arg(&script))?;
    Ok(Handover { quit: true })
}

#[cfg(windows)]
fn portable(installer: &Path, exe: &Path) -> Result<Handover, Error> {
    let file = std::fs::File::open(installer)?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| Error::Install(e.to_string()))?;
    let wanted = exe.file_name().and_then(|n| n.to_str()).unwrap_or("7zip-gui.exe").to_owned();
    let mut found = None;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| Error::Install(e.to_string()))?;
        let name = entry.enclosed_name().and_then(|p| p.file_name().map(std::ffi::OsStr::to_os_string));
        if name.as_deref() == Some(std::ffi::OsStr::new(&wanted)) {
            let target = installer.with_extension("exe");
            let mut out = std::fs::File::create(&target)?;
            std::io::copy(&mut entry, &mut out)?;
            found = Some(target);
            break;
        }
    }
    let new_exe = found.ok_or_else(|| Error::Install(format!("{wanted} is not in the archive")))?;
    self_replace::self_replace(&new_exe).map_err(|e| Error::Install(e.to_string()))?;
    let _ = std::fs::remove_file(&new_exe);
    detach(&mut Command::new(exe))?;
    Ok(Handover { quit: true })
}

#[cfg(target_os = "macos")]
fn dmg(installer: &Path, exe: &Path) -> Result<Handover, Error> {
    const SCRIPT: &str = r#"
pid="$1"; dmg="$2"; target="$3"
while kill -0 "$pid" 2>/dev/null; do sleep 0.3; done
mnt="$(mktemp -d)"
if ! hdiutil attach -nobrowse -quiet -mountpoint "$mnt" "$dmg"; then open "$dmg"; exit 1; fi
app="$(find "$mnt" -maxdepth 1 -name '*.app' | head -n 1)"
if [ -n "$app" ] && rm -rf "$target.old" && mv "$target" "$target.old" && ditto "$app" "$target"; then
  rm -rf "$target.old"
else
  [ -d "$target.old" ] && [ ! -d "$target" ] && mv "$target.old" "$target"
  hdiutil detach -quiet "$mnt"; open "$dmg"; exit 1
fi
hdiutil detach -quiet "$mnt"
rmdir "$mnt" 2>/dev/null
open "$target"
"#;
    // .../7zip-GUI.app/Contents/MacOS/7zip-gui → .../7zip-GUI.app
    let bundle = exe
        .ancestors()
        .nth(3)
        .filter(|p| p.extension().is_some_and(|e| e == "app"))
        .ok_or_else(|| Error::Install("the app is not running from an .app bundle".into()))?;
    detach(
        Command::new("/bin/sh")
            .arg("-c")
            .arg(SCRIPT)
            .arg("sh")
            .arg(std::process::id().to_string())
            .arg(installer)
            .arg(bundle),
    )?;
    Ok(Handover { quit: true })
}

#[cfg(all(unix, not(target_os = "macos")))]
fn linux_package(kind: InstallKind, installer: &Path, exe: &Path) -> Result<Handover, Error> {
    // apt resolves the dependencies (7-Zip is only recommended); dnf and zypper
    // likewise. rpm -U is the last resort.
    const SCRIPT: &str = r#"
file="$1"; kind="$2"
if [ "$kind" = deb ]; then
  if command -v apt-get >/dev/null; then exec apt-get install -y --allow-downgrades "$file"; fi
  exec dpkg -i "$file"
fi
if command -v dnf >/dev/null; then exec dnf install -y "$file"; fi
if command -v zypper >/dev/null; then exec zypper --non-interactive install --allow-unsigned-rpm "$file"; fi
exec rpm -U --replacepkgs "$file"
"#;
    let status = Command::new("pkexec")
        .arg("/bin/sh")
        .arg("-c")
        .arg(SCRIPT)
        .arg("sh")
        .arg(installer)
        .arg(kind.to_string())
        .stdin(Stdio::null())
        .status();
    match status {
        Ok(status) if status.success() => {
            detach(&mut Command::new(exe))?;
            Ok(Handover { quit: true })
        }
        Ok(status) if status.code() == Some(126) => {
            Err(Error::Install("the administrator password was not given".into()))
        }
        _ => {
            // No polkit agent, or the package manager failed: let the
            // desktop's software installer take over.
            Command::new("xdg-open")
                .arg(installer)
                .spawn()
                .map(drop)
                .map_err(|e| Error::Install(format!("could not install {}: {e}", installer.display())))?;
            Ok(Handover { quit: false })
        }
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
fn appimage(installer: &Path) -> Result<Handover, Error> {
    use std::os::unix::fs::PermissionsExt;
    let target = std::env::var_os("APPIMAGE")
        .map(std::path::PathBuf::from)
        .ok_or_else(|| Error::Install("not running as an AppImage".into()))?;
    let staged = target.with_extension("new");
    std::fs::copy(installer, &staged)
        .map_err(|e| Error::Install(format!("can't write next to {}: {e}", target.display())))?;
    std::fs::set_permissions(&staged, std::fs::Permissions::from_mode(0o755))?;
    // Renaming over a running AppImage is safe: the old inode stays mounted.
    std::fs::rename(&staged, &target)?;
    detach(&mut Command::new(&target))?;
    Ok(Handover { quit: true })
}

// Kinds that don't exist on this platform.

#[cfg(not(windows))]
fn msi(_: &Path, _: &Path) -> Result<Handover, Error> {
    Err(Error::Install("MSI packages only install on Windows".into()))
}

#[cfg(not(windows))]
fn portable(_: &Path, _: &Path) -> Result<Handover, Error> {
    Err(Error::Install("the portable build is for Windows".into()))
}

#[cfg(not(target_os = "macos"))]
fn dmg(_: &Path, _: &Path) -> Result<Handover, Error> {
    Err(Error::Install("disk images only install on macOS".into()))
}

#[cfg(not(all(unix, not(target_os = "macos"))))]
fn linux_package(_: InstallKind, _: &Path, _: &Path) -> Result<Handover, Error> {
    Err(Error::Install("Linux packages only install on Linux".into()))
}

#[cfg(not(all(unix, not(target_os = "macos"))))]
fn appimage(_: &Path) -> Result<Handover, Error> {
    Err(Error::Install("AppImages only run on Linux".into()))
}
