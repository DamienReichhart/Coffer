# Changelog

All notable changes to Coffer are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project uses
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

To release, move the entries of **Unreleased** under a new `## [x.y.z] - YYYY-MM-DD`
heading, set the same version in `Cargo.toml` (`[workspace.package]`), run
`cargo update --workspace`, and merge to `main`. See [docs/RELEASING.md](docs/RELEASING.md).

## [Unreleased]

### Added

- Multi-volume archives open and extract from any of their volumes: numbered (`backup.zip.001` … `.011`), RAR (`.part1.rar` and `.rar`/`.r00`) and spanned ZIP (`.z01` … `.zip`). Dropping or opening every volume at once opens the archive once, the Extract screen shows how many volumes it found and their total size, and missing volumes are named instead of 7-Zip's "unexpected end of archive".
- Links to the documentation and to the GitHub repository in the sidebar.

### Changed

- A redesigned interface: flat outlined groups instead of shadowed cards, denser controls, a home screen built around the three main actions and the recent archives, a grouped sidebar, segmented pickers, and Create/Extract buttons that stay at the bottom of their forms.
- The archive browser keeps file names readable in narrow windows: the details panel, then the secondary columns and the toolbar labels give way first. A multiple selection shows its total in the details panel.
- The archive size of a split or spanned archive counts every volume.

## [0.1.1] - 2026-11-13

### Changed

- The APT repository now uses the standard Debian layout. If you added it with v0.1.0's instructions, replace the line in `/etc/apt/sources.list.d/coffer.list` with `deb [signed-by=/usr/share/keyrings/coffer.gpg] https://damienreichhart.github.io/Coffer/ stable main`.

### Fixed

- The APT and RPM repositories are published with every release; v0.1.0's were missing because the website deployment failed after the release.

The application itself is unchanged from 0.1.0.

## [0.1.0] - 2026-11-12

### Added

- Browse archives in every format the installed 7-Zip reads (7z, ZIP, RAR, TAR, ISO, CAB, WIM and more): folders, natural sorting, search across the whole archive, multi-selection with Ctrl and Shift, and a details panel.
- Open a file straight from an archive with a double-click.
- Create 7z, ZIP, TAR, GZip, BZip2, XZ and WIM archives: compression level and method, dictionary size, threads, solid mode, AES-256 encryption with encrypted file names, split volumes, exclusions and deleting the originals.
- Extract everything or a selection, into a folder named after the archive, with or without the folder structure, and a choice for existing files.
- Add, delete and rename entries of 7z, ZIP, TAR and WIM archives, and test any archive.
- Checksums (CRC32, CRC64, SHA-1, SHA-256, BLAKE2sp, and MD5, SHA-512 and XXH64 with 7-Zip 24 or newer) with verification against a published value.
- An Activity view with live progress, cancellation, retry and the log of every operation.
- Drag and drop everywhere, keyboard shortcuts, light and dark themes that follow the system.
- Automatic updates from the website: every installer is checked against its SHA-256 and a minisign signature before it runs.
- Installers for Windows (MSI and portable, x64 and ARM64), macOS (Apple silicon and Intel) and Linux (deb, rpm and AppImage, x64 and ARM64).

[Unreleased]: https://github.com/DamienReichhart/Coffer/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/DamienReichhart/Coffer/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/DamienReichhart/Coffer/releases/tag/v0.1.0
