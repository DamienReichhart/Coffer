/**
 * Splits a Keep a Changelog file (https://keepachangelog.com) into versions:
 *
 *   ## [Unreleased]
 *   ## [1.2.3] - 2026-11-12
 *
 * Link reference definitions at the end (`[1.2.3]: https://…`) are dropped; the
 * page links each version to its GitHub release itself.
 */
export interface ChangelogEntry {
  /** `1.2.3`, or `Unreleased`. */
  version: string;
  /** `2026-11-12`, or null (Unreleased, or no date given). */
  date: string | null;
  /** Markdown body of the section, without its heading. */
  body: string;
  unreleased: boolean;
}

const HEADING = /^##\s+\[?([^\]\s]+)\]?(?:\s+[-–—]\s+(\d{4}-\d{2}-\d{2}))?\s*(?:\[YANKED\])?\s*$/i;
const LINK_DEF = /^\[[^\]]+\]:\s+\S+/;

export function parseChangelog(markdown: string): ChangelogEntry[] {
  const entries: ChangelogEntry[] = [];
  let current: ChangelogEntry | null = null;
  const lines: string[] = [];

  const flush = () => {
    if (!current) return;
    current.body = lines.join('\n').trim();
    entries.push(current);
    lines.length = 0;
  };

  for (const line of markdown.split(/\r?\n/)) {
    const heading = HEADING.exec(line);
    if (heading) {
      flush();
      const version = heading[1] ?? '';
      current = {
        version,
        date: heading[2] ?? null,
        body: '',
        unreleased: version.toLowerCase() === 'unreleased',
      };
      continue;
    }
    if (current && !LINK_DEF.test(line)) lines.push(line);
  }
  flush();
  return entries;
}

/** True when a section has content beyond headings (an empty `### Added` counts as empty). */
export function hasContent(entry: ChangelogEntry): boolean {
  return entry.body.split('\n').some((line) => line.trim() && !/^#{1,6}\s/.test(line.trim()));
}
