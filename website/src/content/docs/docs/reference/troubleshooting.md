---
title: Troubleshooting
description: Fixes for the problems people run into most.
---

## “7-Zip was not found”

Coffer needs 7-Zip installed separately. [Install it](/docs/installing-7zip/), then press
**Detect again** in **Settings → 7-Zip** (or on the welcome screen). If it is installed somewhere
unusual, point **Settings → 7-Zip → Executable** at it.

On macOS, apps started from Finder don’t see your shell’s `PATH`; Coffer looks in Homebrew’s and
MacPorts’ folders for that reason. A copy elsewhere needs the Executable setting.

## An archive won’t open

- **“Can not open the file as archive”** — the file is not an archive, is damaged or incomplete
  (an interrupted download), or its format isn’t supported by your 7-Zip. Try **Test**.
- **RAR archives on Linux** — your distribution’s 7-Zip may lack RAR support. See
  [Installing 7-Zip](/docs/installing-7zip/).
- **Split archives** — open the *first* volume, with all the others in the same folder.

## “Wrong password”

7-Zip can’t tell a wrong password from a damaged encrypted file. Check the keyboard layout and
Caps Lock; passwords are case-sensitive.

## Extraction or compression fails part-way

Open the job’s **View log** in [Activity](/docs/guides/activity/). Common causes: not enough disk
space, a destination you can’t write to, a file locked by another program (Windows), or a path too
long for the file system.

## The window is blank or flickers (Linux)

Coffer draws with the GPU through Vulkan or OpenGL and falls back to a software renderer. On
virtual machines or old drivers, force the software renderer:

```sh
ICED_BACKEND=tiny-skia coffer
```

## macOS says the app is damaged

The build isn’t notarized. See [Installation → macOS](/docs/installation/).

## Updates fail

- **“Signature verification failed”** — the download did not match the release. Nothing was
  installed; try again later, or download from the [download page](/download/) and
  [verify it](/docs/reference/verifying/).
- **Linux .deb/.rpm** — installation needs `pkexec` (polkit) and administrator rights. Without
  them, install the downloaded package with `sudo apt install ./file.deb` or `sudo dnf install
  ./file.rpm`.
- **Behind a proxy** — the app uses your system’s certificate store; set `HTTPS_PROXY` if your
  network requires a proxy.

## Logs

**Settings → Advanced → Open log folder** opens the app’s log files. Attach the latest one, and the
job’s 7-Zip log, when you [open an issue](https://github.com/DamienReichhart/Coffer/issues). Set
`RUST_LOG=debug` before starting the app for more detail.
