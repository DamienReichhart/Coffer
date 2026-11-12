#!/usr/bin/env bash
# Imports the Developer ID certificate (base64 .p12 in APPLE_CERTIFICATE) into a
# temporary keychain so `codesign` can use it. Does nothing when it is not set:
# the build is then ad-hoc signed.
set -euo pipefail

if [ -z "${APPLE_CERTIFICATE:-}" ]; then
  echo "::notice::APPLE_CERTIFICATE is not set: the macOS app will be ad-hoc signed and not notarized."
  exit 0
fi

keychain="$RUNNER_TEMP/signing.keychain-db"
password="$(openssl rand -hex 24)"
certificate="$RUNNER_TEMP/certificate.p12"

printf '%s' "$APPLE_CERTIFICATE" | base64 --decode > "$certificate"
security create-keychain -p "$password" "$keychain"
security set-keychain-settings -lut 21600 "$keychain"
security unlock-keychain -p "$password" "$keychain"
security import "$certificate" -P "${APPLE_CERTIFICATE_PASSWORD:-}" -A -t cert -f pkcs12 -k "$keychain"
security set-key-partition-list -S apple-tool:,apple: -k "$password" "$keychain" >/dev/null
# Keep the existing keychains searchable, with ours first.
existing=()
while IFS= read -r line; do existing+=("$(echo "$line" | tr -d '" ')"); done < <(security list-keychains -d user)
security list-keychains -d user -s "$keychain" "${existing[@]}"
rm -f "$certificate"
security find-identity -v -p codesigning "$keychain"
