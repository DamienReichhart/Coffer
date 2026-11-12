---
title: Drag and drop
description: Drop archives to open them and files to compress them.
---

Drop files from your file manager onto the 7zip-GUI window:

| You drop | 7zip-GUI |
| --- | --- |
| One archive | Opens it in the archive view. |
| Files or folders (not an archive) | Opens **Compress** with them added. |
| Files onto the **Compress** screen | Adds them to the list. |
| Files onto the **Checksums** screen | Adds them to the list. |

While you drag files over the window, a highlight shows where they will go.

An “archive” here is a file whose extension 7-Zip recognises (`.7z`, `.zip`, `.rar`, `.tar`,
`.gz`, `.xz`, `.iso`, `.001`…). To compress an archive into another one, add it with **Add files**
on the Compress screen.

:::note[Linux]
Drag and drop works on X11 and, with a recent compositor, on Wayland. If it doesn’t on your
desktop, use the file pickers instead.
:::
