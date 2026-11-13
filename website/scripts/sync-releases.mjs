#!/usr/bin/env node
// @ts-check
/**
 * Reads the GitHub Releases of the repository and writes what the website needs
 * (docs/ARCHITECTURE.md, "Website data"):
 *
 *   src/data/releases.json        the KEEP_VERSIONS newest releases, for the pages
 *   public/updates/latest.json    the update manifest the app polls
 *   public/downloads/v<version>/  with --download: every installer, signature and
 *                                 SHA256SUMS.txt of those releases, so the site
 *                                 serves them itself (checked against SHA256SUMS)
 *
 * Usage: node scripts/sync-releases.mjs [--download]
 *
 * Environment:
 *   GITHUB_TOKEN        optional; raises the API rate limit
 *   GITHUB_REPOSITORY   owner/name (default DamienReichhart/Coffer)
 *   SITE_URL, BASE_PATH the site's origin and base path, as Astro gets them
 *   RELEASES_REQUIRED=1 fail instead of writing empty data when GitHub can't be read
 *   MAX_DOWNLOAD_MB     refuse to host more than this (default 900: Pages allows 1 GB)
 */
import { createHash } from 'node:crypto';
import { createWriteStream } from 'node:fs';
import { mkdir, rm, writeFile, stat } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { Readable } from 'node:stream';
import { pipeline } from 'node:stream/promises';
import { fileURLToPath } from 'node:url';
import {
  KEEP_VERSIONS,
  buildManifest,
  buildReleaseData,
  parseSha256Sums,
  selectReleases,
  siteRoot,
  versionOf,
} from './lib/releases.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const website = join(here, '..');
const DATA_FILE = join(website, 'src/data/releases.json');
const UPDATES_DIR = join(website, 'public/updates');
const DOWNLOADS_DIR = join(website, 'public/downloads');

const repository = process.env.GITHUB_REPOSITORY || 'DamienReichhart/Coffer';
const token = process.env.GITHUB_TOKEN || '';
const required = process.env.RELEASES_REQUIRED === '1';
const download = process.argv.includes('--download');
const maxBytes = Number(process.env.MAX_DOWNLOAD_MB || 900) * 1024 * 1024;
const origin = process.env.SITE_URL || 'https://damienreichhart.github.io';
const basePath = process.env.SITE_URL ? process.env.BASE_PATH || '/' : '/Coffer';
const root = siteRoot(origin, basePath);

const headers = {
  Accept: 'application/vnd.github+json',
  'X-GitHub-Api-Version': '2022-11-28',
  'User-Agent': 'coffer-website-build',
  ...(token ? { Authorization: `Bearer ${token}` } : {}),
};

/** @param {string} url */
async function get(url, accept = headers.Accept) {
  const response = await fetch(url, { headers: { ...headers, Accept: accept }, redirect: 'follow' });
  if (!response.ok) throw new Error(`GET ${url}: ${response.status} ${response.statusText}`);
  return response;
}

/** @returns {Promise<import('./lib/releases.mjs').GithubRelease[]>} */
async function listReleases() {
  /** @type {import('./lib/releases.mjs').GithubRelease[]} */
  const all = [];
  for (let page = 1; page <= 10; page++) {
    const response = await get(`https://api.github.com/repos/${repository}/releases?per_page=100&page=${page}`);
    const batch = await response.json();
    all.push(...batch);
    if (batch.length < 100) break;
  }
  return all;
}

/** @param {import('./lib/releases.mjs').GithubAsset} asset */
const assetText = async (asset) => (await get(asset.browser_download_url, 'application/octet-stream')).text();

/**
 * Downloads a file, returning its SHA-256.
 * @param {string} url
 * @param {string} path
 */
async function downloadTo(url, path) {
  const response = await get(url, 'application/octet-stream');
  if (!response.body) throw new Error(`GET ${url}: empty body`);
  const hash = createHash('sha256');
  const source = Readable.fromWeb(/** @type {import('node:stream/web').ReadableStream} */ (response.body));
  source.on('data', (chunk) => hash.update(chunk));
  await pipeline(source, createWriteStream(path));
  return hash.digest('hex');
}

/**
 * @param {string} path
 * @param {unknown} value
 */
async function writeJson(path, value) {
  await mkdir(dirname(path), { recursive: true });
  await writeFile(path, `${JSON.stringify(value, null, 2)}\n`);
}

async function main() {
  // Start clean: a version that drops out of the last KEEP_VERSIONS is no longer served.
  await rm(UPDATES_DIR, { recursive: true, force: true });
  if (download) await rm(DOWNLOADS_DIR, { recursive: true, force: true });

  /** @type {import('./lib/releases.mjs').GithubRelease[]} */
  let kept = [];
  try {
    kept = selectReleases(await listReleases(), KEEP_VERSIONS);
  } catch (error) {
    if (required) throw error;
    console.warn(`::warning::Could not read the releases of ${repository}: ${error instanceof Error ? error.message : error}. Building without them.`);
  }

  /** @type {import('./lib/releases.mjs').ReleaseData[]} */
  const releases = [];
  let hostedBytes = 0;
  for (const release of kept) {
    const version = versionOf(release.tag_name);
    const byName = new Map(release.assets.map((a) => [a.name, a]));
    const sumsAsset = byName.get('SHA256SUMS.txt');
    const sums = sumsAsset ? parseSha256Sums(await assetText(sumsAsset)) : new Map();
    if (!sumsAsset) console.warn(`::warning::${release.tag_name} has no SHA256SUMS.txt`);

    const signatures = new Map();
    for (const asset of release.assets) {
      if (!asset.name.endsWith('.minisig')) continue;
      signatures.set(asset.name.slice(0, -'.minisig'.length), (await assetText(asset)).trim() + '\n');
    }

    if (download) {
      const dir = join(DOWNLOADS_DIR, `v${version}`);
      await mkdir(dir, { recursive: true });
      for (const asset of release.assets) {
        hostedBytes += asset.size;
        if (hostedBytes > maxBytes)
          throw new Error(
            `Hosting the last ${kept.length} releases needs more than ${process.env.MAX_DOWNLOAD_MB || 900} MB; ` +
              'GitHub Pages sites are limited to 1 GB. Lower KEEP_VERSIONS or shrink the installers.',
          );
        const path = join(dir, asset.name);
        const digest = await downloadTo(asset.browser_download_url, path);
        const expected = sums.get(asset.name);
        if (expected && expected !== digest)
          throw new Error(`${asset.name}: SHA-256 ${digest} does not match SHA256SUMS.txt (${expected})`);
        const { size } = await stat(path);
        if (size !== asset.size) throw new Error(`${asset.name}: downloaded ${size} bytes, expected ${asset.size}`);
      }
      console.log(`Hosted ${release.assets.length} files of ${release.tag_name}`);
    }

    releases.push(buildReleaseData(release, { root, hosted: download, sums, signatures }));
  }

  await writeJson(DATA_FILE, {
    generatedAt: new Date().toISOString(),
    repository,
    keep: KEEP_VERSIONS,
    hosted: download,
    releases,
  });

  const manifest = buildManifest(releases, { root });
  if (manifest) {
    await writeJson(join(UPDATES_DIR, 'latest.json'), manifest);
    console.log(`Update manifest: ${manifest.version} with ${Object.keys(manifest.assets).length} installers`);
  } else {
    console.log('No stable release yet: no update manifest.');
  }
  console.log(
    `Kept ${releases.length} release(s): ${releases.map((r) => r.version).join(', ') || 'none'}` +
      (download ? ` (${(hostedBytes / 1048576).toFixed(1)} MB hosted)` : ''),
  );
}

main().catch((/** @type {unknown} */ error) => {
  console.error(`::error::${error instanceof Error ? error.message : error}`);
  process.exit(1);
});
