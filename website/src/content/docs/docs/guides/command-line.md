---
title: Command line
description: Start Coffer on an archive, or straight into extracting or compressing.
---

The executable is `coffer` (`coffer.exe` on Windows; inside the app bundle on macOS, at
`/Applications/Coffer.app/Contents/MacOS/coffer`).

```text
coffer [ARCHIVE]
coffer --extract <ARCHIVE> [--to <DIR>]
coffer --compress <PATH>...
coffer --version
coffer --help
```

| Command | What it does |
| --- | --- |
| `coffer` | Opens the home screen. |
| `coffer photos.7z` | Opens the archive. This is what file associations and *Open with* use. |
| `coffer --extract photos.7z` | Opens the extract dialog for the archive. |
| `coffer --extract photos.7z --to ~/Pictures` | The same, with the destination filled in. |
| `coffer --compress report.pdf data/` | Opens Compress with these files and folders. |
| `coffer --version` | Prints the version and exits. |
| `coffer --help` | Prints the usage and exits. |

Coffer always shows its window: you review the options and press the button. For scripted,
unattended work, call 7-Zip directly — `7z x`, `7z a` and friends.

## File manager integration

- **Linux:** the `.deb`, `.rpm` and AppImage register Coffer for common archive types, so it
  appears under *Open with*.
- **Windows:** choose *Open with → Choose another app* on an archive and pick Coffer.
- **macOS:** *Get Info → Open with* on an archive, then **Change All…** to make it the default.
