# 7zip-GUI website

The website and documentation at <https://damienreichhart.github.io/7zip-GUI/>, built with
[Astro](https://astro.build) and [Starlight](https://starlight.astro.build) and deployed to GitHub
Pages by `.github/workflows/pages.yml`.

## Develop

```sh
npm ci
npm run screenshots   # optional: renders the app's screenshots (needs Rust)
npm run sync          # optional: reads the GitHub releases (set GITHUB_TOKEN to avoid rate limits)
npm run dev
```

The site builds without screenshots (placeholders are shown), without release data (the download
page points to GitHub) and without `../CHANGELOG.md`.

| Script | What it does |
| --- | --- |
| `npm run dev` / `preview` | Dev server / serve the built site. |
| `npm run build` | `astro check` (types) then `astro build` into `dist/`, validating every docs link. |
| `npm run lint` | ESLint (TypeScript, Astro). |
| `npm test` | Unit tests (`node --test`): release sync, changelog parser, platform detection. |
| `npm run sync [-- --download]` | `scripts/sync-releases.mjs`, see below. |
| `npm run screenshots` | Renders `src/assets/screenshots/{light,dark}/*.png` with the app’s `screenshots` example. |
| `npm run brand` | Re-renders `public/` icons and `og.png` from `../assets/brand/icon.svg`. |

## Layout

```
src/
  components/      site chrome, home and download sections, Starlight overrides
  content/docs/    the documentation (Markdown/MDX); sidebar in src/lib/sidebar.ts
  layouts/         the marketing pages' layout
  lib/             site facts, release data, changelog parser, platform detection
  pages/           home, download, changelog, privacy, legal notice, 404, robots.txt,
                   [file].ts (repository key + .repo file, when the key exists)
  styles/          theme tokens (shared with the app), site and docs styles
scripts/
  sync-releases.mjs  GitHub Releases → src/data/releases.json, public/updates/latest.json,
                     public/downloads/ (with --download)
  lib/releases.mjs   its pure, unit-tested logic
```

## Release data

`scripts/sync-releases.mjs` implements the contracts in `../docs/ARCHITECTURE.md`. It keeps the
**five** newest published releases. With `--download` it copies their installers, signatures and
`SHA256SUMS.txt` into `public/downloads/v<version>/` (checking each file’s SHA-256), so the site
serves them and the update manifest points to the site, with GitHub as the mirror.

Environment: `GITHUB_TOKEN`, `GITHUB_REPOSITORY`, `SITE_URL` and `BASE_PATH` (as Astro gets them),
`RELEASES_REQUIRED=1` (fail when GitHub can’t be read), `MAX_DOWNLOAD_MB` (default 900, under the
1 GB GitHub Pages limit).

Generated files (`src/data/releases.json`, `public/updates/`, `public/downloads/`,
`src/assets/screenshots/`) are git-ignored.
