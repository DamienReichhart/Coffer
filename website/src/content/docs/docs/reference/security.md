---
title: Security and privacy
description: What 7zip-GUI does with your files, passwords and network — honestly.
---

## Your files

7zip-GUI only reads and writes the files you point it at, through 7-Zip. It never uploads anything.
Archives are listed without being extracted, and extraction writes only into the destination you
choose.

7-Zip refuses to write outside the destination for entries with absolute paths or `..` components
(“zip slip”), and 7zip-GUI passes destinations to it explicitly.

## Passwords

- Passwords you type are kept **in memory only**, for the archive or job they belong to, and are
  never written to the settings file or the logs.
- 7-Zip’s command-line interface accepts a password only as an argument (`-p<password>`). That
  means **while a job runs, the password is visible to other programs running under your own user
  account** that list processes with their arguments (such as `ps` or Task Manager’s command-line
  column). Other users on a properly configured system can’t see it, and it disappears when the
  job ends. This is a limitation of how 7-Zip is driven, and the same applies to any script calling
  `7z -p`. On a shared or untrusted account, prefer typing the password into 7-Zip directly.
- Logs and error messages shown in **Activity** are cleaned of the password before they are
  displayed.

## Encryption strength

Encryption is 7-Zip’s: AES-256 for 7z and ZIP. A 7z archive with **Encrypt file names** hides
names and sizes as well. Its security is only as good as your password: use a long, unique one.

## Network

The app makes no network requests except the update check and, when you accept an update, the
download. See [Automatic updates](/docs/reference/updates/) for how updates are verified (SHA-256 +
minisign signature against a key built into the app) and how to turn the check off.

## Releases

Installers are built by GitHub Actions from the public repository, with checksums, minisign
signatures and build provenance attestations — see [Verifying downloads](/docs/reference/verifying/).

## Reporting a vulnerability

Please report security issues privately through
[GitHub Security Advisories](https://github.com/DamienReichhart/7zip-GUI/security/advisories/new)
rather than a public issue. Vulnerabilities in 7-Zip itself should go to the
[7-Zip project](https://www.7-zip.org/).
