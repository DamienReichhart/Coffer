---
title: Command line
description: Start 7zip-GUI on an archive, or straight into extracting or compressing.
---

The executable is `7zip-gui` (`7zip-gui.exe` on Windows; inside the app bundle on macOS, at
`/Applications/7zip-GUI.app/Contents/MacOS/7zip-gui`).

```text
7zip-gui [ARCHIVE]
7zip-gui --extract <ARCHIVE> [--to <DIR>]
7zip-gui --compress <PATH>...
7zip-gui --version
7zip-gui --help
```

| Command | What it does |
| --- | --- |
| `7zip-gui` | Opens the welcome screen. |
| `7zip-gui photos.7z` | Opens the archive. This is what file associations and *Open with* use. |
| `7zip-gui --extract photos.7z` | Opens the extract dialog for the archive. |
| `7zip-gui --extract photos.7z --to ~/Pictures` | The same, with the destination filled in. |
| `7zip-gui --compress report.pdf data/` | Opens Compress with these files and folders. |
| `7zip-gui --version` | Prints the version and exits. |
| `7zip-gui --help` | Prints the usage and exits. |

7zip-GUI always shows its window: you review the options and press the button. For scripted,
unattended work, call 7-Zip directly — `7z x`, `7z a` and friends.

## File manager integration

- **Linux:** the `.deb`, `.rpm` and AppImage register 7zip-GUI for common archive types, so it
  appears under *Open with*.
- **Windows:** choose *Open with → Choose another app* on an archive and pick 7zip-GUI.
- **macOS:** *Get Info → Open with* on an archive, then **Change All…** to make it the default.
