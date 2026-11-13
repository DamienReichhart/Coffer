#!/usr/bin/env bash
# End-to-end test of the APT and RPM repositories the website publishes:
# lays out the packages the way the site hosts them, builds the signed
# repositories with scripts/package-repos.sh, then installs Coffer from them
# with apt (on this runner) and dnf (in a Fedora container), verifying every
# signature on the way, exactly as a user's machine would.
#
#   scripts/ci/test-package-repos.sh <dist folder> <armored public key>
set -euo pipefail

dist="$(realpath "${1:?usage: $0 <dist> <public key>}")"
key="$(realpath "${2:?usage: $0 <dist> <public key>}")"
root="$(git rev-parse --show-toplevel)"
# apt reads repositories as the unprivileged _apt user: the test site must be
# world-readable, which a runner's private temp folder is not.
site="$(mktemp -d /tmp/coffer-repo-site.XXXXXX)"
version="$(cd "$root" && cargo metadata --no-deps --format-version 1 --locked \
  | python3 -c 'import json,sys; print(next(p["version"] for p in json.load(sys.stdin)["packages"] if p["name"] == "coffer"))')"

trap 'rm -rf "$site"' EXIT
mkdir -p "$site/downloads/v$version"
cp "$dist"/*.deb "$dist"/*.rpm "$site/downloads/v$version/"

# The container mounts the site at /site, so the RPM metadata points there.
"$root/scripts/package-repos.sh" "$site" "file:///site"
chmod -R a+rX "$site"

echo "::group::APT: install from the repository"
sudo install -d -m 755 /etc/apt/keyrings
sudo gpg --batch --yes --dearmor -o /etc/apt/keyrings/coffer-ci.gpg "$key"
echo "deb [signed-by=/etc/apt/keyrings/coffer-ci.gpg] file://$site/ stable main" \
  | sudo tee /etc/apt/sources.list.d/coffer-ci.list >/dev/null
# Refresh only this repository; apt refuses unsigned or badly signed indexes.
# Any warning fails the test too: users would see it on every update.
update_log="$(mktemp)"
sudo apt-get update \
  -o Dir::Etc::sourcelist=/etc/apt/sources.list.d/coffer-ci.list \
  -o Dir::Etc::sourceparts=- -o APT::Get::List-Cleanup=0 2>&1 | tee "$update_log"
if grep -E '^(W|E):' "$update_log"; then
  echo "::error::apt reported problems with the repository"
  exit 1
fi
apt-cache policy coffer | tee /dev/stderr | grep -q "file:$site"
sudo apt-get install -y coffer
coffer --version
sudo apt-get remove -y coffer
sudo rm /etc/apt/sources.list.d/coffer-ci.list /etc/apt/keyrings/coffer-ci.gpg
echo "::endgroup::"

echo "::group::DNF: install from the repository (Fedora)"
docker run --rm \
  -v "$site:/site:ro" -v "$key:/key.asc:ro" \
  quay.io/fedora/fedora:42 bash -euo pipefail -c '
    cat > /etc/yum.repos.d/coffer.repo <<REPO
[coffer]
name=Coffer
baseurl=file:///site/rpm/
enabled=1
gpgcheck=1
repo_gpgcheck=1
gpgkey=file:///key.asc
REPO
    dnf -y --setopt=install_weak_deps=False --disablerepo="*" --enablerepo=coffer makecache
    dnf -y --setopt=install_weak_deps=False install coffer
    rpm -q coffer
    coffer --version
  '
echo "::endgroup::"

echo "Both repositories install Coffer $version with verified signatures."
