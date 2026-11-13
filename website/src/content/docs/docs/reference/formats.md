---
title: Supported formats
description: Which archive formats Coffer can create, and which it can open.
---

Coffer supports exactly what your 7-Zip supports. With a current 7-Zip:

## Create, open and update

| Format | Extensions | Notes |
| --- | --- | --- |
| 7z | `.7z` | Best ratio; solid compression; AES-256 with encrypted names. |
| ZIP | `.zip` | Universal; AES-256 encryption. |
| TAR | `.tar` | No compression on its own. |
| GZIP | `.gz`, `.tgz` | Single file. |
| BZIP2 | `.bz2`, `.tbz2` | Single file. |
| XZ | `.xz`, `.txz` | Single file. |
| WIM | `.wim` | Windows Imaging Format. |

## Open and extract only

APFS, AR, ARJ, CAB, CHM, CPIO, CramFS, DMG, EXT, FAT, GPT, HFS, IHEX, ISO, LZH, LZMA, MBR, MSI,
NSIS, NTFS, QCOW2, RAR, RAR5, RPM, SquashFS, UDF, UEFI, VDI, VHD, VHDX, VMDK, XAR, Z and more —
see the list 7-Zip prints with `7z i`.

RAR is extract-only in every 7-Zip; some Linux distributions build 7-Zip without RAR support (see
[Installing 7-Zip](/docs/installing-7zip/)).

## Split archives

Every format above can be read when it is split into volumes (`.001`, `.part1.rar`, `.r00`,
`.z01`), from any of the volumes. See
[Split and multi-part archives](/docs/guides/extracting/#split-and-multi-part-archives).

## Compressed TAR in one step

GZIP, BZIP2 and XZ compress a single file. To compress a folder with them, create a TAR first and
then compress the `.tar` — or use 7z or ZIP, which handle folders directly.
