---
title: Releasing
description: How a version is released — for maintainers.
---

Releases are automatic once the version is bumped. Nothing is released by accident: the release
workflow refuses to run for a version that already exists.

## Steps

1. **Bump the version** in the workspace `Cargo.toml` (`[workspace.package] version`), following
   [semantic versioning](https://semver.org), and run `cargo check` so `Cargo.lock` follows.
2. **Write the changelog**: rename **Unreleased** in `CHANGELOG.md` to the new version with today’s
   date (`## [1.2.0] - 2026-11-12`) and start a fresh, empty **Unreleased** section.
3. **Merge to `main`** (through `dev` and a pull request).

## What happens next

On every push to `main`, the **Release** workflow:

1. **Plans the release** and **fails before building anything** when:
   - the tag `v<version>` already exists — the version wasn’t bumped;
   - `CHANGELOG.md` has no section for the version;
   - the version files disagree, or the signing keys aren’t configured.
2. Runs the whole **CI** suite.
3. **Builds** every installer natively: Linux (deb, rpm, AppImage), Windows (MSI, portable zip) and
   macOS (dmg), each for x64 and ARM64, and signs each with minisign.
4. **Publishes** the GitHub Release — tag, release notes from the changelog, every installer,
   signatures, `SHA256SUMS.txt` and build provenance attestations — only once every build
   succeeded.

When the release is published, the **Pages** workflow rebuilds this website: the download page and
changelog show the new version, the last five releases’ installers are copied onto the site, the
APT/RPM repositories are regenerated, and `updates/latest.json` makes running apps offer the update.

A version with a prerelease suffix (`1.2.0-rc.1`) is published as a GitHub pre-release: it appears
on the download page’s previous versions but is never offered as an automatic update.

## Keys and secrets

The release needs repository secrets created once with `scripts/setup-signing.sh` (see
`docs/RELEASING.md` in the repository): the minisign key that signs updates, and optionally the GPG
key that signs the Linux repositories and the Apple and Windows code-signing certificates.
