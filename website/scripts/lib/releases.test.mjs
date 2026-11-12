import assert from 'node:assert/strict';
import { describe, it } from 'node:test';
import {
  buildManifest,
  buildReleaseData,
  changelogAnchor,
  classifyAsset,
  compareSemver,
  parseSha256Sums,
  selectReleases,
  siteRoot,
} from './releases.mjs';

const ROOT = 'https://damienreichhart.github.io/7zip-GUI';
const HEX = (c) => c.repeat(64);

describe('classifyAsset', () => {
  it('reads every installer the release publishes', () => {
    const cases = [
      ['7zip-GUI-1.2.3-linux-x64.deb', 'linux-x64-deb'],
      ['7zip-GUI-1.2.3-linux-arm64.deb', 'linux-arm64-deb'],
      ['7zip-GUI-1.2.3-linux-x64.rpm', 'linux-x64-rpm'],
      ['7zip-GUI-1.2.3-linux-arm64.AppImage', 'linux-arm64-appimage'],
      ['7zip-GUI-1.2.3-windows-x64.msi', 'windows-x64-msi'],
      ['7zip-GUI-1.2.3-windows-arm64.zip', 'windows-arm64-portable'],
      ['7zip-GUI-1.2.3-macos-arm64.dmg', 'macos-arm64-dmg'],
      ['7zip-GUI-1.2.3-macos-x64.dmg', 'macos-x64-dmg'],
    ];
    for (const [name, key] of cases) assert.equal(classifyAsset(name)?.key, key, name);
  });

  it('keeps prerelease versions intact', () => {
    const info = classifyAsset('7zip-GUI-2.0.0-rc.1-windows-x64.msi');
    assert.deepEqual(info, {
      version: '2.0.0-rc.1',
      os: 'windows',
      arch: 'x64',
      kind: 'msi',
      key: 'windows-x64-msi',
    });
  });

  it('ignores signatures, checksums and unknown files', () => {
    for (const name of [
      '7zip-GUI-1.2.3-linux-x64.deb.minisig',
      'SHA256SUMS.txt',
      '7zip-GUI-1.2.3-linux-x86.deb',
      '7zip-GUI-1.2.3-freebsd-x64.deb',
      'other-1.2.3-linux-x64.deb',
      '7zip-GUI-1.2-linux-x64.deb',
    ])
      assert.equal(classifyAsset(name), null, name);
  });
});

describe('parseSha256Sums', () => {
  it('reads text and binary mode lines and skips noise', () => {
    const sums = parseSha256Sums(
      `${HEX('A')}  a.deb\n${HEX('b')} *b.msi\r\n\nnot a line\n${HEX('c')}  name with spaces.zip\n`,
    );
    assert.equal(sums.get('a.deb'), HEX('a'));
    assert.equal(sums.get('b.msi'), HEX('b'));
    assert.equal(sums.get('name with spaces.zip'), HEX('c'));
    assert.equal(sums.size, 3);
  });
});

describe('compareSemver', () => {
  it('orders by precedence', () => {
    const sorted = ['1.0.0', '1.0.0-rc.1', '1.0.0-alpha', '0.9.10', '1.10.0', '1.2.0', '1.0.0-alpha.1', '1.0.0-beta.11', '1.0.0-beta.2'].sort(compareSemver);
    assert.deepEqual(sorted, [
      '0.9.10',
      '1.0.0-alpha',
      '1.0.0-alpha.1',
      '1.0.0-beta.2',
      '1.0.0-beta.11',
      '1.0.0-rc.1',
      '1.0.0',
      '1.2.0',
      '1.10.0',
    ]);
  });
  it('ignores a leading v and build metadata', () => {
    assert.equal(compareSemver('v1.2.3', '1.2.3+build.5'), 0);
  });
});

const release = (tag, extra = {}) => ({
  tag_name: tag,
  name: `7zip-GUI ${tag}`,
  body: `Notes for ${tag}`,
  draft: false,
  prerelease: false,
  published_at: '2026-11-12T10:00:00Z',
  html_url: `https://github.com/DamienReichhart/7zip-GUI/releases/tag/${tag}`,
  assets: [],
  ...extra,
});

describe('selectReleases', () => {
  it('keeps the five newest published versions', () => {
    const tags = ['v0.1.0', 'v0.10.0', 'v0.2.0', 'v0.3.0', 'v0.4.0', 'v0.5.0', 'v0.9.0'];
    const releases = [...tags.map((t) => release(t)), release('v9.9.9', { draft: true }), release('nightly')];
    assert.deepEqual(
      selectReleases(releases).map((r) => r.tag_name),
      ['v0.10.0', 'v0.9.0', 'v0.5.0', 'v0.4.0', 'v0.3.0'],
    );
  });
});

const gh = (name, size = 100) => ({
  name,
  size,
  browser_download_url: `https://github.com/DamienReichhart/7zip-GUI/releases/download/v1.2.3/${name}`,
});

function sample(extra = {}) {
  return release('v1.2.3', {
    assets: [
      gh('7zip-GUI-1.2.3-linux-x64.deb', 10),
      gh('7zip-GUI-1.2.3-linux-x64.deb.minisig', 1),
      gh('7zip-GUI-1.2.3-windows-x64.msi', 20),
      gh('7zip-GUI-1.2.3-windows-x64.msi.minisig', 1),
      gh('7zip-GUI-1.2.3-macos-arm64.dmg', 30),
      gh('SHA256SUMS.txt', 1),
    ],
    ...extra,
  });
}
const sums = new Map([
  ['7zip-GUI-1.2.3-linux-x64.deb', HEX('1')],
  ['7zip-GUI-1.2.3-windows-x64.msi', HEX('2')],
  ['7zip-GUI-1.2.3-macos-arm64.dmg', HEX('3')],
]);
const signatures = new Map([
  ['7zip-GUI-1.2.3-linux-x64.deb', 'untrusted comment: deb\nSIG'],
  ['7zip-GUI-1.2.3-windows-x64.msi', 'untrusted comment: msi\nSIG'],
]);

describe('buildReleaseData', () => {
  it('links to the site when the installers are hosted', () => {
    const data = buildReleaseData(sample(), { root: ROOT, hosted: true, sums, signatures });
    assert.equal(data.version, '1.2.3');
    assert.equal(data.assets.length, 3);
    const deb = data.assets.find((a) => a.key === 'linux-x64-deb');
    assert.equal(deb.url, `${ROOT}/downloads/v1.2.3/7zip-GUI-1.2.3-linux-x64.deb`);
    assert.equal(deb.githubUrl, gh('7zip-GUI-1.2.3-linux-x64.deb').browser_download_url);
    assert.equal(deb.signatureUrl, `${ROOT}/downloads/v1.2.3/7zip-GUI-1.2.3-linux-x64.deb.minisig`);
    assert.equal(deb.sha256, HEX('1'));
    assert.equal(data.checksumsUrl, `${ROOT}/downloads/v1.2.3/SHA256SUMS.txt`);
  });

  it('links to GitHub otherwise', () => {
    const data = buildReleaseData(sample(), { root: ROOT, hosted: false, sums, signatures });
    for (const a of data.assets) assert.equal(a.url, a.githubUrl);
  });
});

describe('buildManifest', () => {
  it('describes the newest stable release with verifiable installers only', () => {
    const stable = buildReleaseData(sample(), { root: ROOT, hosted: true, sums, signatures });
    const pre = { ...stable, version: '2.0.0-rc.1', prerelease: true };
    const manifest = buildManifest([pre, stable], { root: ROOT });
    assert.equal(manifest.schema, 1);
    assert.equal(manifest.version, '1.2.3');
    assert.equal(manifest.notes, 'Notes for v1.2.3');
    assert.equal(manifest.notes_url, `${ROOT}/changelog/#v1-2-3`);
    // The dmg has no signature, so it isn't offered.
    assert.deepEqual(Object.keys(manifest.assets).sort(), ['linux-x64-deb', 'windows-x64-msi']);
    assert.deepEqual(manifest.assets['windows-x64-msi'], {
      name: '7zip-GUI-1.2.3-windows-x64.msi',
      url: `${ROOT}/downloads/v1.2.3/7zip-GUI-1.2.3-windows-x64.msi`,
      mirror: gh('7zip-GUI-1.2.3-windows-x64.msi').browser_download_url,
      size: 20,
      sha256: HEX('2'),
      signature: 'untrusted comment: msi\nSIG',
    });
  });

  it('is null without a stable release', () => {
    const pre = buildReleaseData(sample({ prerelease: true }), { root: ROOT, hosted: false, sums, signatures });
    assert.equal(buildManifest([pre], { root: ROOT }), null);
    assert.equal(buildManifest([], { root: ROOT }), null);
  });
});

describe('helpers', () => {
  it('builds anchors and roots', () => {
    assert.equal(changelogAnchor('1.2.3'), 'v1-2-3');
    assert.equal(changelogAnchor('2.0.0-rc.1'), 'v2-0-0-rc-1');
    assert.equal(siteRoot('https://x.github.io/', '/7zip-GUI/'), 'https://x.github.io/7zip-GUI');
    assert.equal(siteRoot('https://example.com', '/'), 'https://example.com');
  });
});
