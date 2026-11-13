#!/usr/bin/env bash
# Builds signed APT and RPM repositories over the installers the website hosts.
#
#   scripts/package-repos.sh <site dist folder> <site URL>
#
# The installers stay where the website serves them (downloads/v<version>/);
# the repositories only add indexes, so they cost almost no space:
#   APT: flat repository at <site>/apt/  →  deb [signed-by=…] <site>/ apt/
#   RPM: <site>/rpm/repodata             →  baseurl=<site>/rpm/
# Needs apt-ftparchive (apt-utils), createrepo_c and the signing key in gpg.
set -euo pipefail

dist="${1:?usage: $0 <dist> <site url>}"
site="${2:?usage: $0 <dist> <site url>}"
site="${site%/}"
cd "$dist"

if ! compgen -G "downloads/*/*.deb" >/dev/null; then
  echo "::notice::No installers are hosted yet; no package repositories to build."
  exit 0
fi

# --- APT (flat repository; Filename paths are relative to the site root) ---
mkdir -p apt
apt-ftparchive packages downloads > apt/Packages
gzip -9 --keep --force apt/Packages
apt-ftparchive \
  -o APT::FTPArchive::Release::Origin="Coffer" \
  -o APT::FTPArchive::Release::Label="Coffer" \
  -o APT::FTPArchive::Release::Suite="stable" \
  -o APT::FTPArchive::Release::Codename="stable" \
  -o APT::FTPArchive::Release::Architectures="amd64 arm64" \
  -o APT::FTPArchive::Release::Description="Coffer packages" \
  release apt > apt/Release
gpg --batch --yes --armor --detach-sign --output apt/Release.gpg apt/Release
gpg --batch --yes --clearsign --output apt/InRelease apt/Release

# --- RPM (metadata in rpm/, packages referenced where they are hosted) ---
if compgen -G "downloads/*/*.rpm" >/dev/null; then
  createrepo_c --quiet --baseurl "$site/downloads/" --outputdir rpm downloads
  gpg --batch --yes --armor --detach-sign --output rpm/repodata/repomd.xml.asc rpm/repodata/repomd.xml
fi

echo "Package repositories:"
du -sh apt rpm 2>/dev/null || true
