<p align="center">
  <img src="assets/brand/icon.svg" width="96" height="96" alt="">
</p>

<h1 align="center">7zip-GUI</h1>

<p align="center">
  A modern, native desktop interface for 7-Zip — for Windows, macOS and Linux, on x64 and ARM.<br>
  <a href="https://damienreichhart.github.io/7zip-GUI/">Website</a> ·
  <a href="https://damienreichhart.github.io/7zip-GUI/download/">Download</a> ·
  <a href="https://damienreichhart.github.io/7zip-GUI/docs/">Documentation</a> ·
  <a href="CHANGELOG.md">Changelog</a>
</p>

<p align="center">
  <a href="https://github.com/DamienReichhart/7zip-GUI/actions/workflows/ci.yml"><img src="https://github.com/DamienReichhart/7zip-GUI/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/DamienReichhart/7zip-GUI/actions/workflows/release.yml"><img src="https://github.com/DamienReichhart/7zip-GUI/actions/workflows/release.yml/badge.svg" alt="Release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue" alt="MIT licence"></a>
</p>

---

7zip-GUI opens, creates, extracts, tests and checksums archives by driving the
**7-Zip command-line program you install yourself**. It is written in Rust with
[iced](https://iced.rs): one small native binary per platform, no web view, no
runtime.

> **7-Zip is not included.** 7zip-GUI never bundles, downloads or links 7-Zip.
> Install it with `winget install 7zip.7zip`, `brew install sevenzip` or
> `sudo apt install 7zip` (more in the [docs](https://damienreichhart.github.io/7zip-GUI/docs/installing-7zip/)).
> 7zip-GUI is an independent project, not affiliated with 7-Zip or Igor Pavlov.

## Features

- **Browse** 7z, ZIP, RAR, TAR, ISO, CAB, WIM and every other format your 7-Zip reads — folders, natural sorting, search across the archive, multi-selection, details panel, open a file with a double-click.
- **Create** 7z, ZIP, TAR, GZip, BZip2, XZ and WIM archives — level, method, dictionary, threads, solid mode, AES-256 with encrypted file names, split volumes, exclusions.
- **Extract** everything or a selection, into a folder named after the archive, with your choice for existing files.
- **Edit** archives: add, delete and rename entries; **test** integrity.
- **Checksums** (CRC32/64, SHA-1, SHA-256, BLAKE2sp, and MD5/SHA-512/XXH64 with 7-Zip 24+) with verification.
- **Activity** view with live progress, cancel, retry and logs. Drag and drop, keyboard shortcuts, light and dark themes.
- **Automatic updates** from the website, verified with SHA-256 and a minisign signature before anything runs.
- No account, no telemetry: the only network request is the update check, and it can be turned off.

## Install

| Platform | Packages |
| --- | --- |
| Windows 10/11 (x64, ARM64) | `.msi` installer, portable `.zip` |
| macOS 11+ (Apple silicon), 10.15+ (Intel) | `.dmg` |
| Linux (x64, ARM64) | `.deb`, `.rpm`, `.AppImage`, APT and DNF repositories |

Everything is on the [download page](https://damienreichhart.github.io/7zip-GUI/download/) and on
[GitHub Releases](https://github.com/DamienReichhart/7zip-GUI/releases), with `SHA256SUMS.txt`,
minisign signatures and build provenance.

## Build from source

Rust is pinned in `rust-toolchain.toml` (rustup installs it automatically).

```bash
cargo run -p sevenzip-gui
```

```bash
cargo test --workspace
```

The live tests run against the 7-Zip on your `PATH` and are skipped when there is none.
On Linux no system libraries are needed to build; at run time the window needs `libxkbcommon`.

## Repository

| Path | Contents |
| --- | --- |
| `crates/core` | Finding and driving 7-Zip: command lines, listings, progress, checksums. No UI. |
| `crates/updater` | Update manifest, verified downloads, platform installers. No UI. |
| `crates/app` | The iced application (binary `7zip-gui`) and the screenshot generator. |
| `xtask` | Release tooling: `cargo xtask release-plan`, `package`, `sign`, `icons`. |
| `packaging/` | Desktop entry, AppStream, WiX, `Info.plist`, public keys. |
| `website/` | Astro + Starlight site and documentation (GitHub Pages). |

[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) describes the contracts between them,
[docs/RELEASING.md](docs/RELEASING.md) how releases work, and
[CONTRIBUTING.md](CONTRIBUTING.md) how to contribute.

## Licence

MIT — see [LICENSE](LICENSE). Bundled: the Inter typeface (SIL OFL 1.1) and Lucide icons (ISC).
7-Zip is © Igor Pavlov and distributed under its own licence; it is not part of this project.
