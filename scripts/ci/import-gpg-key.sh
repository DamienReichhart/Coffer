#!/usr/bin/env bash
# Imports the package repository key (PACKAGES_GPG_PRIVATE_KEY, armored, with
# PACKAGES_GPG_PASSPHRASE in $PASSPHRASE) for non-interactive signing.
set -euo pipefail

: "${KEY:?the armored private key is expected in KEY}"
install -d -m 700 ~/.gnupg
printf '%s' "${PASSPHRASE:-}" > "$RUNNER_TEMP/passphrase"
chmod 600 "$RUNNER_TEMP/passphrase"
printf 'pinentry-mode loopback\npassphrase-file %s\n' "$RUNNER_TEMP/passphrase" >> ~/.gnupg/gpg.conf
echo "allow-loopback-pinentry" >> ~/.gnupg/gpg-agent.conf
gpgconf --kill gpg-agent || true
printf '%s\n' "$KEY" | gpg --batch --import
gpg --list-secret-keys --keyid-format long
