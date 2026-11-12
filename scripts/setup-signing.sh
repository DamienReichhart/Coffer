#!/usr/bin/env bash
# One-time set-up of the release keys. Run it once, from a clone of the
# repository, on a trusted machine with the GitHub CLI (`gh auth login`) and gpg:
#
#   scripts/setup-signing.sh
#
# It creates:
#   1. The update-signing key (minisign). The public half goes to
#      packaging/keys/update.pub and is compiled into the app: the updater
#      installs nothing that this key didn't sign. The secret half becomes the
#      UPDATE_SIGNING_KEY and UPDATE_SIGNING_PASSWORD repository secrets.
#   2. The package-repository key (GPG, for APT and RPM). The public half goes
#      to packaging/linux/7zip-gui.asc; the secret half becomes the
#      PACKAGES_GPG_PRIVATE_KEY and PACKAGES_GPG_PASSPHRASE secrets.
#
# Secrets only travel from this machine to GitHub; the temporary copies are
# shredded. Commit the two public keys afterwards. Losing the update key means
# installed apps can no longer update themselves: keep an offline backup of
# the files this script prints before it deletes them, if you want one.
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

command -v gh >/dev/null || { echo "Install the GitHub CLI (https://cli.github.com) and run gh auth login." >&2; exit 1; }
command -v gpg >/dev/null || { echo "Install gpg." >&2; exit 1; }
gh auth status >/dev/null

workdir="$(mktemp -d)"
cleanup() { find "$workdir" -type f -exec shred -u {} + 2>/dev/null || rm -rf "$workdir"; rm -rf "$workdir"; }
trap cleanup EXIT

random() { openssl rand -base64 33 | tr -d '\n'; }

if [ -f packaging/keys/update.pub ]; then
  echo "packaging/keys/update.pub already exists: keeping the current update key."
  echo "(Replacing it would stop every installed copy from updating itself.)"
else
  echo "==> Creating the update-signing key"
  export UPDATE_SIGNING_PASSWORD
  UPDATE_SIGNING_PASSWORD="$(random)"
  cargo xtask keygen --encrypted --secret "$workdir/update.key"
  gh secret set UPDATE_SIGNING_KEY < "$workdir/update.key"
  printf '%s' "$UPDATE_SIGNING_PASSWORD" | gh secret set UPDATE_SIGNING_PASSWORD
  echo "    UPDATE_SIGNING_KEY and UPDATE_SIGNING_PASSWORD are set."
fi

if [ -f packaging/linux/7zip-gui.asc ]; then
  echo "packaging/linux/7zip-gui.asc already exists: keeping the current repository key."
else
  echo "==> Creating the package-repository key"
  export GNUPGHOME="$workdir/gnupg"
  install -d -m 700 "$GNUPGHOME"
  passphrase="$(random)"
  gpg --batch --pinentry-mode loopback --passphrase "$passphrase" \
    --quick-gen-key "7zip-GUI packages <damienreichhart@users.noreply.github.com>" rsa4096 sign 5y
  fingerprint="$(gpg --list-secret-keys --with-colons | awk -F: '/^fpr:/ { print $10; exit }')"
  gpg --armor --export "$fingerprint" > packaging/linux/7zip-gui.asc
  gpg --batch --pinentry-mode loopback --passphrase "$passphrase" --armor --export-secret-keys "$fingerprint" > "$workdir/packages.asc"
  gh secret set PACKAGES_GPG_PRIVATE_KEY < "$workdir/packages.asc"
  printf '%s' "$passphrase" | gh secret set PACKAGES_GPG_PASSPHRASE
  echo "    PACKAGES_GPG_PRIVATE_KEY and PACKAGES_GPG_PASSPHRASE are set (key $fingerprint, valid 5 years)."
fi

echo
echo "Done. Commit the public keys:"
echo "  git add packaging/keys/update.pub packaging/linux/7zip-gui.asc && git commit -m 'Add the release public keys'"
