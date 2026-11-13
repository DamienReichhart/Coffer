---
title: Automatic updates
description: How Coffer finds, verifies and installs new versions — and how to turn it off.
---

Coffer keeps itself up to date. Every step is designed so that a compromised download can’t be
installed.

## How it works

1. **Check.** When the app starts, and every few hours while it runs, it downloads a small file from
   this website: `updates/latest.json`. It lists the latest version, its release notes and, for each
   platform and package format, the installer’s address, size, SHA-256 checksum and
   [minisign](https://jedisct1.github.io/minisign/) signature.
2. **Offer.** If that version is newer than yours, a banner shows it with its release notes. Choose
   **Install**, **Later**, or **Skip this version**.
3. **Download.** The installer for *your* platform, architecture and package format is downloaded
   from this website (or from GitHub Releases if the website can’t be reached).
4. **Verify.** The app checks the file’s size, its SHA-256 checksum and its minisign signature
   against the public key built into the app. If any check fails, the file is deleted and nothing is
   installed.
5. **Install.** The app hands the verified installer to your system and restarts:

| You installed | The update |
| --- | --- |
| Windows installer (.msi) | Runs `msiexec` on the new MSI, which upgrades in place, then restarts the app. |
| Windows portable (.zip) | Replaces its own executable with the new one, then restarts. |
| macOS (.dmg) | Replaces the app bundle with the one from the new disk image, then restarts. |
| Linux .deb | Installs the package with `apt` through `pkexec`, so you are asked for your password. |
| Linux .rpm | Installs the package with `dnf` or `zypper` through `pkexec`. |
| AppImage | Replaces the AppImage file in place, then restarts. |

The app knows how it was installed from a small marker that each package installs next to it. A
build without one (one you compiled yourself) still tells you about new versions but opens the
[download page](/download/) instead of installing.

If you installed from the APT or DNF repository, your package manager updates Coffer with the rest
of your system as well; both ways work.

## Turning it off

**Settings → Updates → Check for updates automatically.** With it off, the app never contacts the
network on its own; **Check now** still works when you want it.

Administrators can also block `damienreichhart.github.io` — the app works fully offline.

## What the check reveals

Only what any HTTPS request does: your IP address and that a Coffer app asked for the latest
version. No identifier, no usage data, nothing about your files. See [Privacy](/privacy/).

## Which versions are offered

Only stable releases. The website keeps the last five releases; older ones remain on
[GitHub Releases](https://github.com/DamienReichhart/Coffer/releases).
