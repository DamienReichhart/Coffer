/**
 * The releases the site knows about, from src/data/releases.json, written by
 * scripts/sync-releases.mjs. The file is absent in a fresh checkout: the site
 * then builds with no releases and points visitors to GitHub.
 */
export type Os = 'linux' | 'windows' | 'macos';
export type Arch = 'x64' | 'arm64';
export type Kind = 'deb' | 'rpm' | 'appimage' | 'msi' | 'portable' | 'dmg';

export interface ReleaseAsset {
  name: string;
  key: string;
  version: string;
  os: Os;
  arch: Arch;
  kind: Kind;
  size: number;
  sha256: string | null;
  url: string;
  githubUrl: string;
  signatureUrl: string | null;
  signature: string | null;
}

export interface Release {
  version: string;
  tag: string;
  title: string;
  date: string | null;
  prerelease: boolean;
  htmlUrl: string;
  notes: string;
  checksumsUrl: string | null;
  assets: ReleaseAsset[];
}

interface ReleasesFile {
  generatedAt: string;
  repository: string;
  keep: number;
  hosted: boolean;
  releases: Release[];
}

const files = import.meta.glob<{ default: ReleasesFile }>('../data/releases.json', { eager: true });
const data: ReleasesFile | undefined = Object.values(files)[0]?.default;

/** Every kept release, newest first. */
export const releases: Release[] = data?.releases ?? [];

/** The newest stable release, or the newest prerelease when there is no stable one yet. */
export const latest: Release | undefined = releases.find((r) => !r.prerelease) ?? releases[0];

/** Kept releases other than `latest`. */
export const previous: Release[] = releases.filter((r) => r !== latest);

export const OS_LABEL: Record<Os, string> = { windows: 'Windows', macos: 'macOS', linux: 'Linux' };
export const ARCH_LABEL: Record<Arch, string> = { x64: 'x64', arm64: 'ARM64' };
export const KIND_LABEL: Record<Kind, string> = {
  msi: 'Installer (.msi)',
  portable: 'Portable (.zip)',
  dmg: 'Disk image (.dmg)',
  deb: 'Debian / Ubuntu (.deb)',
  rpm: 'Fedora / openSUSE (.rpm)',
  appimage: 'AppImage',
};
/** The order platforms and formats are listed in. */
export const OS_ORDER: Os[] = ['windows', 'macos', 'linux'];
export const KIND_ORDER: Kind[] = ['msi', 'portable', 'dmg', 'deb', 'rpm', 'appimage'];

/** A release's assets grouped by OS, in display order. */
export function byOs(release: Release): { os: Os; assets: ReleaseAsset[] }[] {
  return OS_ORDER.map((os) => ({
    os,
    assets: release.assets
      .filter((a) => a.os === os)
      .sort(
        (a, b) =>
          KIND_ORDER.indexOf(a.kind) - KIND_ORDER.indexOf(b.kind) ||
          (a.arch === b.arch ? 0 : a.arch === 'x64' ? -1 : 1),
      ),
  })).filter((g) => g.assets.length > 0);
}

/** The installer recommended for a platform, keyed `<os>-<arch>` for the download button. */
export function recommended(release: Release | undefined): Record<string, string> {
  const pick: Record<string, Kind[]> = {
    windows: ['msi', 'portable'],
    macos: ['dmg'],
    linux: ['appimage', 'deb', 'rpm'],
  };
  const out: Record<string, string> = {};
  if (!release) return out;
  for (const os of OS_ORDER)
    for (const arch of ['x64', 'arm64'] as Arch[]) {
      for (const kind of pick[os] ?? []) {
        const asset = release.assets.find((a) => a.os === os && a.arch === arch && a.kind === kind);
        if (asset) {
          out[`${os}-${arch}`] = asset.url;
          break;
        }
      }
    }
  return out;
}
