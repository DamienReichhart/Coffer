#!/usr/bin/env bash
# Installs what `cargo xtask package` needs on a Linux runner: dpkg-deb (always
# there), rpmbuild, desktop-file-utils / appstream for validation, and a
# pinned, checksum-verified appimagetool. Usage: install-linux-packaging-tools.sh x86_64|aarch64
set -euo pipefail

arch="${1:?usage: $0 x86_64|aarch64}"
version="1.9.1"
case "$arch" in
  x86_64)  sha256="ed4ce84f0d9caff66f50bcca6ff6f35aae54ce8135408b3fa33abfc3cb384eb0" ;;
  aarch64) sha256="f0837e7448a0c1e4e650a93bb3e85802546e60654ef287576f46c71c126a9158" ;;
  *) echo "unsupported architecture: $arch" >&2; exit 1 ;;
esac

sudo apt-get update
sudo apt-get install -y --no-install-recommends rpm desktop-file-utils appstream file

dest="$HOME/.local/bin"
mkdir -p "$dest"
curl -fsSL --retry 3 -o "$dest/appimagetool" \
  "https://github.com/AppImage/appimagetool/releases/download/${version}/appimagetool-${arch}.AppImage"
echo "${sha256}  $dest/appimagetool" | sha256sum --check --strict
chmod +x "$dest/appimagetool"
echo "$dest" >> "${GITHUB_PATH:-/dev/null}"
echo "APPIMAGETOOL=$dest/appimagetool" >> "${GITHUB_ENV:-/dev/null}"
# GitHub's runners have no FUSE: run AppImages by extracting them.
echo "APPIMAGE_EXTRACT_AND_RUN=1" >> "${GITHUB_ENV:-/dev/null}"
