#!/usr/bin/env bash
# Signs RPM packages with the first secret key in gpg's keyring, then checks
# the signature is there. Used by the release (the repository key) and by CI
# (a throwaway key), so CI exercises exactly what a release runs.
#
#   scripts/ci/sign-rpm.sh dist/*.rpm
#
# The key's passphrase, if any, is read from $RUNNER_TEMP/passphrase
# (written by import-gpg-key.sh).
set -euo pipefail

[ "$#" -gt 0 ] || { echo "usage: $0 <package.rpm>..." >&2; exit 2; }

fingerprint="$(gpg --list-secret-keys --with-colons | awk -F: '/^fpr:/ { print $10; exit }')"
[ -n "$fingerprint" ] || { echo "no secret key in gpg's keyring" >&2; exit 1; }

passphrase="${RUNNER_TEMP:-/tmp}/passphrase"
[ -f "$passphrase" ] || : > "$passphrase"

rpm --define "_gpg_name $fingerprint" \
    --define "_gpg_sign_cmd_extra_args --batch --pinentry-mode loopback --passphrase-file $passphrase" \
    --addsign "$@"

for package in "$@"; do
  if ! rpm -qp --qf '%{SIGPGP:pgpsig}%{RSAHEADER:pgpsig}\n' "$package" | grep -q "Key ID"; then
    echo "::error::$package is not signed"
    exit 1
  fi
  echo "signed $(basename "$package")"
done
