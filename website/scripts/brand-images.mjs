#!/usr/bin/env node
// @ts-check
/**
 * Renders the site's icons and social card from the logo, assets/brand/icon.svg:
 *
 *   public/favicon.svg, public/apple-touch-icon.png (180), public/icon-192.png,
 *   public/icon-512.png, public/og.png (1200×630)
 *
 * The outputs are committed; run `npm run brand` after changing the logo.
 */
import { readFile, writeFile } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import sharp from 'sharp';

const website = join(dirname(fileURLToPath(import.meta.url)), '..');
const logoPath = join(website, '../assets/brand/icon.svg');
const out = (/** @type {string} */ name) => join(website, 'public', name);

const logo = await readFile(logoPath);
await writeFile(out('favicon.svg'), logo);

/** @type {[string, number][]} */
const sizes = [
  ['apple-touch-icon.png', 180],
  ['icon-192.png', 192],
  ['icon-512.png', 512],
];
for (const [name, size] of sizes) {
  await sharp(logo, { density: 384 }).resize(size, size).png({ compressionLevel: 9 }).toFile(out(name));
}

// The social card: the logo and the name on the dark background, with the tagline.
const card = `<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="630" viewBox="0 0 1200 630">
  <defs>
    <radialGradient id="glow" cx="0.82" cy="0.1" r="0.9">
      <stop offset="0" stop-color="#7C8CFF" stop-opacity="0.35"/>
      <stop offset="1" stop-color="#121316" stop-opacity="0"/>
    </radialGradient>
  </defs>
  <rect width="1200" height="630" fill="#121316"/>
  <rect width="1200" height="630" fill="url(#glow)"/>
  <text x="96" y="330" font-family="Inter, 'DejaVu Sans', sans-serif" font-size="96" font-weight="700" fill="#ECEDEF" letter-spacing="-3">Coffer</text>
  <text x="98" y="402" font-family="Inter, 'DejaVu Sans', sans-serif" font-size="38" font-weight="500" fill="#9A9DA8">A modern desktop app for 7-Zip.</text>
  <text x="98" y="520" font-family="Inter, 'DejaVu Sans', sans-serif" font-size="26" font-weight="500" fill="#7C8CFF">Windows · macOS · Linux — free and open source</text>
</svg>`;
const icon = await sharp(logo, { density: 384 }).resize(300, 300).png().toBuffer();
await sharp(Buffer.from(card))
  .composite([{ input: icon, left: 820, top: 165 }])
  .png({ compressionLevel: 9 })
  .toFile(out('og.png'));

console.log('Rendered favicon.svg, apple-touch-icon.png, icon-192.png, icon-512.png and og.png');
