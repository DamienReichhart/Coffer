/**
 * Build-time facts about the signing keys in the repository (read from disk in
 * the build, never shipped to the browser):
 *
 *   ../packaging/linux/7zip-gui.asc   the APT/RPM repositories' public GPG key.
 *                                     The repository instructions (and the key
 *                                     and .repo files) only exist when it does.
 *   ../packaging/keys/update.pub      the minisign key installers are signed with.
 */
import { existsSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';

/** The repository root; REPO_ROOT overrides it (tests). */
const repoRoot = resolve(process.cwd(), process.env.REPO_ROOT || '..');

function read(path: string): string | null {
  const full = resolve(repoRoot, path);
  return existsSync(full) ? readFileSync(full, 'utf8') : null;
}

/** The armored public key of the package repositories, or null. */
export const repositoryKey: string | null = read('packaging/linux/7zip-gui.asc');

/** The minisign public key (its base64 line), or null. */
export const minisignKey: string | null = (() => {
  const text = read('packaging/keys/update.pub');
  if (!text) return null;
  const line = text
    .split(/\r?\n/)
    .map((l) => l.trim())
    .find((l) => l && !l.startsWith('untrusted comment:'));
  return line ?? null;
})();

/** The package name in the deb/rpm repositories. */
export const PACKAGE = '7zip-gui';

/** The dnf/zypper repository definition served at `<site>/7zip-gui.repo`. */
export function repoFile(root: string): string {
  return `[7zip-gui]
name=7zip-GUI
baseurl=${root}/rpm/
enabled=1
gpgcheck=1
repo_gpgcheck=1
gpgkey=${root}/7zip-gui.asc
`;
}
