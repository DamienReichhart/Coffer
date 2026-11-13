// @ts-check
import { satteri } from '@astrojs/markdown-satteri';
import sitemap from '@astrojs/sitemap';
import starlight from '@astrojs/starlight';
import { defineConfig, sharpImageService } from 'astro/config';
import starlightLinksValidator from 'starlight-links-validator';
import baseLinks from './src/lib/markdown/base-links.mjs';
import { sidebar } from './src/lib/sidebar.ts';
import { NAME, REPO_URL, DESCRIPTION } from './src/lib/site.ts';

/*
 * The site is served from GitHub Pages. The Pages workflow passes the origin and
 * base path that `actions/configure-pages` reports (SITE_URL, BASE_PATH), so a
 * custom domain (empty base path) works without editing this file. Elsewhere —
 * local builds and pull requests — it builds for the project site.
 */
const site = process.env.SITE_URL || 'https://damienreichhart.github.io';
const base = process.env.SITE_URL ? process.env.BASE_PATH || '/' : '/Coffer';
/** The base without a trailing slash, for building paths: `/Coffer` or ``. */
const root = base.replace(/\/$/, '');

export default defineConfig({
  site,
  base,
  trailingSlash: 'always',
  markdown: { processor: satteri({ hastPlugins: [baseLinks(base)] }) },
  // Screenshots become AVIF + WebP at several widths; effort 3 keeps builds fast.
  image: { service: sharpImageService({ avif: { effort: 3 } }) },
  // The brand assets and the changelog live at the repo root; let Vite read them.
  vite: { server: { fs: { allow: ['..'] } } },
  integrations: [
    starlight({
      title: NAME,
      description: DESCRIPTION,
      logo: { src: '../assets/brand/icon.svg', alt: NAME },
      favicon: '/favicon.svg',
      social: [{ icon: 'github', label: 'GitHub', href: REPO_URL }],
      editLink: { baseUrl: `${REPO_URL}/edit/main/website/` },
      lastUpdated: false,
      credits: false,
      // The site has its own 404 page (src/pages/404.astro) in the site layout.
      disable404Route: true,
      customCss: [
        '@fontsource-variable/inter',
        '@fontsource-variable/jetbrains-mono',
        './src/styles/theme.css',
        './src/styles/docs.css',
      ],
      components: {
        Header: './src/components/starlight/Header.astro',
        Footer: './src/components/starlight/Footer.astro',
      },
      head: [
        { tag: 'meta', attrs: { property: 'og:image', content: `${site}${root}/og.png` } },
        { tag: 'meta', attrs: { property: 'og:image:width', content: '1200' } },
        { tag: 'meta', attrs: { property: 'og:image:height', content: '630' } },
        { tag: 'meta', attrs: { name: 'twitter:card', content: 'summary_large_image' } },
        { tag: 'link', attrs: { rel: 'manifest', href: `${root}/site.webmanifest` } },
        { tag: 'link', attrs: { rel: 'apple-touch-icon', href: `${root}/apple-touch-icon.png` } },
      ],
      sidebar,
      plugins: [
        // The marketing pages aren't Starlight pages, so the validator can't see them.
        starlightLinksValidator({
          exclude: [
            `${root}/`,
            `${root}/download/**`,
            `${root}/changelog/**`,
            `${root}/privacy/**`,
            `${root}/legal-notice/**`,
          ],
        }),
      ],
    }),
    sitemap(),
  ],
});
