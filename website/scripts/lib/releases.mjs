// @ts-check
/**
 * Pure functions behind scripts/sync-releases.mjs: reading release assets,
 * choosing which releases the site keeps and building the update manifest.
 * The contracts they implement are in docs/ARCHITECTURE.md ("Release assets",
 * "Update manifest", "Website data"). No I/O here, so it is all unit-tested.
 */

/** How many releases the site keeps. Older ones fall out on the next deployment. */
export const KEEP_VERSIONS = 5;

/** The update manifest's format version (docs/ARCHITECTURE.md). */
export const MANIFEST_SCHEMA = 1;

/** File extension → asset kind. */
const KINDS = /** @type {const} */ ({
  deb: 'deb',
  rpm: 'rpm',
  AppImage: 'appimage',
  msi: 'msi',
  zip: 'portable',
  dmg: 'dmg',
});

const SEMVER =
  '(?:0|[1-9]\\d*)\\.(?:0|[1-9]\\d*)\\.(?:0|[1-9]\\d*)(?:-[0-9A-Za-z-]+(?:\\.[0-9A-Za-z-]+)*)?(?:\\+[0-9A-Za-z-]+(?:\\.[0-9A-Za-z-]+)*)?';
const ASSET = new RegExp(
  `^7zip-GUI-(${SEMVER})-(linux|windows|macos)-(x64|arm64)\\.(deb|rpm|AppImage|msi|zip|dmg)$`,
);

/**
 * @typedef {'linux' | 'windows' | 'macos'} Os
 * @typedef {'x64' | 'arm64'} Arch
 * @typedef {'deb' | 'rpm' | 'appimage' | 'msi' | 'portable' | 'dmg'} Kind
 * @typedef {{ version: string, os: Os, arch: Arch, kind: Kind, key: string }} AssetInfo
 */

/**
 * Reads an installer's file name, e.g. `7zip-GUI-1.2.3-linux-x64.deb`.
 * Signatures, checksums and anything else return `null`.
 *
 * @param {string} name
 * @returns {AssetInfo | null}
 */
export function classifyAsset(name) {
  const match = ASSET.exec(name);
  if (!match) return null;
  const [, version, os, arch, ext] = match;
  const kind = KINDS[/** @type {keyof typeof KINDS} */ (ext)];
  return {
    version: /** @type {string} */ (version),
    os: /** @type {Os} */ (os),
    arch: /** @type {Arch} */ (arch),
    kind,
    key: `${os}-${arch}-${kind}`,
  };
}

/**
 * Parses a `sha256sum` listing (`<hex>  <name>` or `<hex> *<name>` per line).
 *
 * @param {string} text
 * @returns {Map<string, string>} file name → lowercase hex digest
 */
export function parseSha256Sums(text) {
  const sums = new Map();
  for (const line of text.split(/\r?\n/)) {
    const match = /^([0-9a-fA-F]{64})\s+\*?(.+?)\s*$/.exec(line);
    if (match) sums.set(/** @type {string} */ (match[2]), /** @type {string} */ (match[1]).toLowerCase());
  }
  return sums;
}

/**
 * @param {string} version
 * @returns {{ core: number[], pre: string[] } | null}
 */
function parseSemver(version) {
  const match = /^v?(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z.-]+))?(?:\+[0-9A-Za-z.-]+)?$/.exec(version);
  if (!match) return null;
  return {
    core: [Number(match[1]), Number(match[2]), Number(match[3])],
    pre: match[4] ? match[4].split('.') : [],
  };
}

/**
 * Compares two versions by semantic-versioning precedence (a leading `v` is ignored).
 * Strings that aren't versions sort before every version.
 *
 * @param {string} a
 * @param {string} b
 * @returns {number} negative if a < b, positive if a > b, 0 if equal
 */
export function compareSemver(a, b) {
  const x = parseSemver(a);
  const y = parseSemver(b);
  if (!x || !y) return (x ? 1 : 0) - (y ? 1 : 0);
  for (let i = 0; i < 3; i++) {
    const d = /** @type {number} */ (x.core[i]) - /** @type {number} */ (y.core[i]);
    if (d !== 0) return d;
  }
  // A version without prerelease identifiers ranks above one with them.
  if (!x.pre.length || !y.pre.length) return (x.pre.length ? 0 : 1) - (y.pre.length ? 0 : 1);
  const n = Math.max(x.pre.length, y.pre.length);
  for (let i = 0; i < n; i++) {
    const p = x.pre[i];
    const q = y.pre[i];
    if (p === undefined) return -1;
    if (q === undefined) return 1;
    if (p === q) continue;
    const pn = /^\d+$/.test(p);
    const qn = /^\d+$/.test(q);
    if (pn && qn) return Number(p) - Number(q);
    if (pn) return -1;
    if (qn) return 1;
    return p < q ? -1 : 1;
  }
  return 0;
}

/**
 * @typedef {{ name: string, size: number, browser_download_url: string }} GithubAsset
 * @typedef {{
 *   tag_name: string, name?: string | null, body?: string | null, draft: boolean,
 *   prerelease: boolean, published_at?: string | null, created_at?: string,
 *   html_url: string, assets: GithubAsset[] }} GithubRelease
 */

/**
 * The releases the site keeps: published (not drafts), tagged with a version,
 * newest first, at most `keep`.
 *
 * @param {GithubRelease[]} releases
 * @param {number} [keep]
 * @returns {GithubRelease[]}
 */
export function selectReleases(releases, keep = KEEP_VERSIONS) {
  return releases
    .filter((r) => !r.draft && parseSemver(r.tag_name))
    .sort((a, b) => compareSemver(b.tag_name, a.tag_name))
    .slice(0, keep);
}

/** `v1.2.3` → `1.2.3`. */
export const versionOf = (/** @type {string} */ tag) => tag.replace(/^v/, '');

/** The changelog page's anchor for a version: `1.2.3` → `v1-2-3`. */
export const changelogAnchor = (/** @type {string} */ version) =>
  `v${version.replace(/[^0-9A-Za-z]+/g, '-')}`;

/**
 * Joins the site's origin and base path: (`https://x.github.io`, `/7zip-GUI/`) →
 * `https://x.github.io/7zip-GUI`.
 *
 * @param {string} origin
 * @param {string} basePath
 */
export function siteRoot(origin, basePath) {
  return `${origin.replace(/\/$/, '')}${basePath.replace(/\/$/, '')}`;
}

/**
 * @typedef {AssetInfo & {
 *   name: string, size: number, sha256: string | null,
 *   url: string, githubUrl: string, signatureUrl: string | null, signature: string | null }} ReleaseAsset
 * @typedef {{
 *   version: string, tag: string, title: string, date: string | null, prerelease: boolean,
 *   htmlUrl: string, notes: string, checksumsUrl: string | null, assets: ReleaseAsset[] }} ReleaseData
 */

/**
 * The site's view of one release. With `hosted`, the installers are served by
 * the site under `downloads/v<version>/`; otherwise links go to GitHub.
 *
 * @param {GithubRelease} release
 * @param {{ root: string, hosted: boolean, sums: Map<string, string>,
 *           signatures: Map<string, string> }} options
 * @returns {ReleaseData}
 */
export function buildReleaseData(release, { root, hosted, sums, signatures }) {
  const version = versionOf(release.tag_name);
  const byName = new Map(release.assets.map((a) => [a.name, a]));
  const local = (/** @type {string} */ name) =>
    `${root}/downloads/v${version}/${encodeURIComponent(name)}`;
  const link = (/** @type {GithubAsset} */ asset) =>
    hosted ? local(asset.name) : asset.browser_download_url;

  /** @type {ReleaseAsset[]} */
  const assets = [];
  for (const asset of release.assets) {
    const info = classifyAsset(asset.name);
    if (!info || info.version !== version) continue;
    const sig = byName.get(`${asset.name}.minisig`);
    assets.push({
      ...info,
      name: asset.name,
      size: asset.size,
      sha256: sums.get(asset.name) ?? null,
      url: link(asset),
      githubUrl: asset.browser_download_url,
      signatureUrl: sig ? link(sig) : null,
      signature: signatures.get(asset.name) ?? null,
    });
  }
  assets.sort((a, b) => a.key.localeCompare(b.key));
  const sumsAsset = byName.get('SHA256SUMS.txt');

  return {
    version,
    tag: release.tag_name,
    title: release.name || `7zip-GUI ${release.tag_name}`,
    date: release.published_at ?? release.created_at ?? null,
    prerelease: release.prerelease,
    htmlUrl: release.html_url,
    notes: release.body ?? '',
    checksumsUrl: sumsAsset ? link(sumsAsset) : null,
    assets,
  };
}

/**
 * The update manifest (`updates/latest.json`) for the newest stable release, or
 * `null` when there is none. Only installers with a checksum and a signature are
 * listed: the updater refuses anything it can't verify, so offering it would
 * only produce a failed update.
 *
 * @param {ReleaseData[]} releases  newest first
 * @param {{ root: string }} options
 */
export function buildManifest(releases, { root }) {
  const latest = releases.find((r) => !r.prerelease);
  if (!latest) return null;
  /** @type {Record<string, { name: string, url: string, mirror: string, size: number, sha256: string, signature: string }>} */
  const assets = {};
  for (const a of latest.assets) {
    if (!a.sha256 || !a.signature) continue;
    assets[a.key] = {
      name: a.name,
      url: a.url,
      mirror: a.githubUrl,
      size: a.size,
      sha256: a.sha256,
      signature: a.signature,
    };
  }
  return {
    schema: MANIFEST_SCHEMA,
    version: latest.version,
    pub_date: latest.date,
    notes: latest.notes,
    notes_url: `${root}/changelog/#${changelogAnchor(latest.version)}`,
    assets,
  };
}
