# Releasing

Releases are cut by **merging to `main` with a new version**. Nothing else is
manual, and nothing is built when the version was not bumped.

## Branches

| Branch | What runs |
| --- | --- |
| any branch, any pull request | **CI** (`ci.yml`): fmt, clippy (pedantic, `-D warnings`) and tests on Linux x64/ARM64, Windows and macOS against a real 7-Zip, MSRV, cargo-deny, actionlint/shellcheck, a Linux packaging smoke test (deb installed and run), and the full website build with fresh screenshots. |
| `main` | **Release** (`release.yml`), which also runs CI on the released commit, then **Pages** (`pages.yml`). |
| weekly | **Audit** (`audit.yml`) and **CodeQL** (`codeql.yml`). |

Work on `dev` (or feature branches), open a pull request to `main`, require the
`ci-success` check in branch protection.

## Cutting a release

1. Move the **Unreleased** entries of `CHANGELOG.md` under `## [x.y.z] - YYYY-MM-DD`.
2. Set `version = "x.y.z"` in `[workspace.package]` of the root `Cargo.toml`.
3. `cargo update --workspace` (updates `Cargo.lock`), commit, merge to `main`.

The Release workflow then:

1. **plan** — `cargo xtask release-plan` fails within a minute, *before any build*, if:
   - `vx.y.z` is already tagged locally or on GitHub (you forgot to bump);
   - the version is not newer than every existing tag;
   - `CHANGELOG.md` has no dated, non-empty section for it;
   - `Cargo.lock` doesn't carry the version;
   - the update-signing secret is missing or doesn't match `packaging/keys/update.pub`.
2. **ci** and **build** run in parallel. The build matrix:

   | Name | Runner | Installers |
   | --- | --- | --- |
   | linux-x64 | ubuntu-22.04 (glibc 2.35) | deb, rpm, AppImage |
   | linux-arm64 | ubuntu-22.04-arm | deb, rpm, AppImage |
   | windows-x64 | windows-latest | msi, portable zip |
   | windows-arm64 | windows-latest (cross) | msi, portable zip |
   | macos-arm64 | macos-latest | dmg |
   | macos-x64 | macos-latest (cross) | dmg |

   Every installer is signed with minisign (`.minisig`, trusted comment binds
   the version and file name). RPMs are GPG-signed when the repository key exists.
3. **publish** — only if CI and all six builds passed: checks every expected
   file exists, writes `SHA256SUMS.txt`, records build provenance
   (`actions/attest-build-provenance`), and creates the tag and GitHub release
   in one step (a `-beta.1` style version becomes a prerelease).
4. **Pages** re-deploys the website: it downloads the installers of the **five
   most recent releases** into `downloads/v<version>/`, rebuilds the download
   page and changelog, regenerates the APT/RPM indexes, and publishes
   `updates/latest.json`, which every installed app reads.

A failed release can be retried with **Run workflow** on the Release workflow;
the version is untouched until `publish` succeeds.

## One-time set-up

```bash
scripts/setup-signing.sh
```

It creates the minisign update key and the GPG repository key, stores the
secrets in the repository (`UPDATE_SIGNING_KEY`, `UPDATE_SIGNING_PASSWORD`,
`PACKAGES_GPG_PRIVATE_KEY`, `PACKAGES_GPG_PASSPHRASE`) and writes the public
keys to `packaging/`. Commit them. Then, in the repository settings:

- **Pages** → Source: *GitHub Actions*.
- **Branches** → protect `main`, require the `ci-success` status.
- **Actions** → allow GitHub Actions to create releases (default `GITHUB_TOKEN` with `contents: write` in the publish job).

Optional secrets for trusted macOS builds: `APPLE_CERTIFICATE` (base64 .p12),
`APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_ID`,
`APPLE_PASSWORD` (app-specific), `APPLE_TEAM_ID`. Without them the app is
ad-hoc signed and macOS asks users to confirm the first launch. Windows code
signing (to avoid SmartScreen warnings) needs a certificate; add a signing step
to the windows build when you have one.

## Hosting limits

GitHub Pages sites are limited to about 1 GB. Five versions of 14 installers
fit comfortably; the Pages workflow fails before deploying above 960 MB and the
sync script refuses to download more than `MAX_DOWNLOAD_MB` (900). If the app
grows, lower `KEEP_VERSIONS` in `website/scripts/sync-releases.mjs` — GitHub
Releases keeps every version regardless, and the update manifest falls back to
the GitHub mirror.
