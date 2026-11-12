/** Prefixes a site path with the configured base, e.g. `/docs/` → `/7zip-GUI/docs/`. */
export function href(path: string): string {
  const base = import.meta.env.BASE_URL.replace(/\/$/, '');
  return `${base}${path.startsWith('/') ? path : `/${path}`}`;
}

/** The site's absolute root URL without a trailing slash, e.g. `https://x.github.io/7zip-GUI`. */
export function siteRoot(site: URL | undefined): string {
  const base = import.meta.env.BASE_URL.replace(/\/$/, '');
  return `${(site?.origin ?? '').replace(/\/$/, '')}${base}`;
}
