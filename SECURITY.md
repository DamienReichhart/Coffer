# Security policy

## Supported versions

Only the latest release receives fixes. The app updates itself (unless you
turned that off), so staying current is one click.

## Reporting a vulnerability

Please **don't open a public issue**. Use GitHub's
[private vulnerability reporting](https://github.com/DamienReichhart/7zip-GUI/security/advisories/new).
You'll get an answer within a week; a fix for a confirmed issue ships as soon
as it's ready, credited to you unless you prefer otherwise.

## What 7zip-GUI does to stay safe

- **Updates** are downloaded over HTTPS and installed only after their size,
  SHA-256 and a minisign signature from the release key (compiled into the app)
  all match; the signature binds the version and file name, so an old signed
  installer can't be replayed.
- **Releases** are built by GitHub Actions from the tagged commit, with
  `SHA256SUMS.txt` and build provenance (`gh attestation verify <file> --repo DamienReichhart/7zip-GUI`).
- **Passwords** are never written to disk or logs. Note that 7-Zip only accepts
  a password as a command-line argument (`-p…`), which other processes of the
  same user can read while 7-Zip runs. This is a limitation of 7-Zip's command
  line on every platform.
- **Dependencies** are checked by cargo-deny on every change and weekly, and by
  CodeQL.
- 7zip-GUI runs the 7-Zip executable it finds (or the one you configure). Keep
  your 7-Zip up to date: archive-parsing vulnerabilities are 7-Zip's to fix.
