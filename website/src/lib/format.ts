/** `12345678` → `11.8 MB` (binary units, as file managers show them). */
export function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes < 0) return '—';
  const units = ['B', 'KB', 'MB', 'GB'];
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit++;
  }
  return `${unit === 0 ? value : value.toFixed(1)} ${units[unit]}`;
}

/** An ISO date as `12 November 2026` (UTC, so builds are reproducible). */
export function formatDate(iso: string | null | undefined): string {
  if (!iso) return '';
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return iso;
  return date.toLocaleDateString('en-GB', {
    day: 'numeric',
    month: 'long',
    year: 'numeric',
    timeZone: 'UTC',
  });
}

/** The changelog's anchor for a version: `1.2.3` → `v1-2-3` (same rule as scripts/lib/releases.mjs). */
export function changelogAnchor(version: string): string {
  return `v${version.replace(/[^0-9A-Za-z]+/g, '-')}`;
}
