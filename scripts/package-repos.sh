#!/usr/bin/env bash
# Builds signed APT and RPM repositories over the installers the website hosts.
#
#   scripts/package-repos.sh <site dist folder> <site URL>
#
# The installers stay where the website serves them (downloads/v<version>/);
# the repositories only add indexes, so they cost almost no space:
#   APT: <site>/dists/stable/…        →  deb [signed-by=…] <site>/ stable main
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

# Start from nothing, so a rebuild never mixes old and new indexes.
rm -rf dists rpm

# --- APT (standard layout; the site root is the archive root) ---
#   deb [signed-by=/usr/share/keyrings/coffer.gpg] <site>/ stable main
# Packages indexes list Filename paths relative to the site root
# (downloads/v<version>/…), so the installers are not copied.
dist_dir=dists/stable
all_packages="$(mktemp)"
apt-ftparchive packages downloads > "$all_packages"
for arch in amd64 arm64; do
  mkdir -p "$dist_dir/main/binary-$arch"
  # Split by each package's own Architecture field: apt-ftparchive --arch
  # filters on *_<arch>.deb file names, which our installers don't use.
  awk -v arch="$arch" 'BEGIN { RS = ""; ORS = "\n\n" } $0 ~ "\nArchitecture: (" arch "|all)\n"' \
    "$all_packages" > "$dist_dir/main/binary-$arch/Packages"
  gzip -9 --keep --force "$dist_dir/main/binary-$arch/Packages"
done
rm -f "$all_packages"
apt-ftparchive \
  -o APT::FTPArchive::Release::Origin="Coffer" \
  -o APT::FTPArchive::Release::Label="Coffer" \
  -o APT::FTPArchive::Release::Suite="stable" \
  -o APT::FTPArchive::Release::Codename="stable" \
  -o APT::FTPArchive::Release::Components="main" \
  -o APT::FTPArchive::Release::Architectures="amd64 arm64" \
  -o APT::FTPArchive::Release::Description="Coffer packages" \
  release "$dist_dir" > "$dist_dir/Release.tmp"
mv "$dist_dir/Release.tmp" "$dist_dir/Release"
gpg --batch --yes --armor --detach-sign --output "$dist_dir/Release.gpg" "$dist_dir/Release"
gpg --batch --yes --clearsign --output "$dist_dir/InRelease" "$dist_dir/Release"

# --- RPM (metadata in rpm/, packages referenced where they are hosted) ---
if compgen -G "downloads/*/*.rpm" >/dev/null; then
  # createrepo_c writes into an existing output folder; it never creates one.
  mkdir -p rpm
  createrepo_c --quiet --baseurl "$site/downloads/" --outputdir rpm downloads
  gpg --batch --yes --armor --detach-sign --output rpm/repodata/repomd.xml.asc rpm/repodata/repomd.xml
fi

echo "Package repositories:"
du -sh dists rpm 2>/dev/null || true
