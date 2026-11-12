# Architecture

This document is the map of the repository and the contracts between its parts.
When you change a contract here, change every side of it in the same pull request.

## Repository layout

| Path | What it is |
| --- | --- |
| `crates/core` (`sevenzip-core`) | Everything that talks to 7-Zip: finding the executable, building command lines, running them, parsing listings and progress. No UI code. |
| `crates/updater` (`sevenzip-updater`) | The update manifest, version comparison, download, SHA-256 + minisign verification and handing over to the platform installer. No UI code. |
| `crates/app` (`sevenzip-gui`) | The iced desktop application (binary `7zip-gui`). Screens, widgets, theme, settings, the job queue. |
| `xtask` | Release and packaging tooling, run as `cargo xtask <command>`: release planning, packaging (deb, rpm, AppImage, msi, zip, dmg), signing, icons. |
| `packaging/` | Static packaging inputs: desktop entry, AppStream metadata, WiX source, macOS `Info.plist`, public keys. |
| `website/` | The Astro + Starlight website and documentation, deployed to GitHub Pages. |
| `assets/brand/` | The logo source (`icon.svg`). Every icon size is rendered from it by `cargo xtask icons`. |
| `.github/workflows/` | CI (`ci.yml`), release (`release.yml`), website (`pages.yml`), scheduled audit (`audit.yml`). |

The crates depend in one direction only: `app → updater`, `app → core`. `core` and
`updater` never depend on each other or on iced, so either can be reused by a
future CLI, a file-manager integration or a test harness.

## 7-Zip is never bundled

The application drives a 7-Zip command-line executable that the user installs
(`7zz`, `7z` or `7za`). We never ship, download or link 7-Zip. `sevenzip-core`
finds it (settings override → `PATH` → well-known install locations) and
reports a clear "not installed" state with per-platform install instructions.

## Release assets (contract: `xtask` ⇄ `release.yml` ⇄ website ⇄ updater)

Every release `vX.Y.Z` publishes these files on GitHub Releases:

```
7zip-GUI-<version>-linux-x64.deb          7zip-GUI-<version>-linux-arm64.deb
7zip-GUI-<version>-linux-x64.rpm          7zip-GUI-<version>-linux-arm64.rpm
7zip-GUI-<version>-linux-x64.AppImage     7zip-GUI-<version>-linux-arm64.AppImage
7zip-GUI-<version>-windows-x64.msi        7zip-GUI-<version>-windows-arm64.msi
7zip-GUI-<version>-windows-x64.zip        7zip-GUI-<version>-windows-arm64.zip   (portable)
7zip-GUI-<version>-macos-x64.dmg          7zip-GUI-<version>-macos-arm64.dmg
<each of the above>.minisig               (minisign signature, verified by the updater)
SHA256SUMS.txt                            (sha256sum format, every installer)
```

`<version>` has no leading `v`. The *asset key* of a file is `<os>-<arch>-<kind>`
with `os ∈ {linux, windows, macos}`, `arch ∈ {x64, arm64}` and
`kind ∈ {deb, rpm, appimage, msi, portable, dmg}`.

## Update manifest (contract: website ⇄ updater)

The website publishes `updates/latest.json` (latest stable release) at
`https://damienreichhart.github.io/7zip-GUI/updates/latest.json`:

```json
{
  "schema": 1,
  "version": "1.2.3",
  "pub_date": "2026-11-12T10:00:00Z",
  "notes": "Markdown release notes",
  "notes_url": "https://damienreichhart.github.io/7zip-GUI/changelog/#v1-2-3",
  "assets": {
    "linux-x64-deb": {
      "name": "7zip-GUI-1.2.3-linux-x64.deb",
      "url": "https://damienreichhart.github.io/7zip-GUI/downloads/v1.2.3/7zip-GUI-1.2.3-linux-x64.deb",
      "mirror": "https://github.com/DamienReichhart/7zip-GUI/releases/download/v1.2.3/7zip-GUI-1.2.3-linux-x64.deb",
      "size": 12345678,
      "sha256": "<hex>",
      "signature": "<full text of the .minisig file>"
    }
  }
}
```

The updater downloads `url`, falls back to `mirror`, checks `size`, `sha256` and
the minisign `signature` against the public key compiled into the app
(`packaging/keys/update.pub`), and only then runs the installer. Unknown fields
are ignored; a manifest with a `schema` greater than the app understands is
treated as "no update" so old clients never misinterpret a new format.

## Website data (contract: `website/scripts/sync-releases.mjs` ⇄ pages)

`sync-releases.mjs` reads the GitHub Releases API and keeps the **five** most
recent releases (`KEEP_VERSIONS`). It writes:

- `website/src/data/releases.json` — what the download page renders.
- `website/public/updates/latest.json` — the update manifest above.
- with `--download`: every asset of those releases into
  `website/public/downloads/v<version>/`, so the site serves the installers
  itself and the updater downloads from the website. Older versions fall out of
  the site on the next deployment.

## Screenshots (contract: app ⇄ website)

`cargo run -p sevenzip-gui --example screenshots -- <out-dir>` renders the real
views headlessly (iced's tiny-skia renderer, demo data, no 7-Zip needed) to
`<out-dir>/{light,dark}/<name>.png` at 1280×800 logical pixels and a scale
factor of 2. Names: `welcome`, `browse`, `compress`, `extract`, `activity`,
`checksums`, `settings`. The Pages workflow regenerates them on every
deployment into `website/src/assets/screenshots/`.

## Install kind (contract: packaging ⇄ updater)

Each package installs a one-word text file named `install-kind` next to the
executable (Windows, macOS `Contents/MacOS`) or at `/usr/lib/7zip-gui/install-kind`
(deb, rpm) whose content is the *kind* above. An AppImage is recognised by the
`APPIMAGE` environment variable. Without a marker (a `cargo run` build) the app
still checks for updates but offers to open the download page instead of
installing.
