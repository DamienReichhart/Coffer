import assert from 'node:assert/strict';
import { describe, it } from 'node:test';
import { hasContent, parseChangelog } from './changelog.ts';

const SAMPLE = `# Changelog

All notable changes to this project are documented in this file.

## [Unreleased]

### Added

## [1.1.0] - 2026-12-01

### Added
- Checksums screen.

### Fixed
- A crash.

## [1.0.0] - 2026-11-12

First release.

[Unreleased]: https://github.com/o/r/compare/v1.1.0...HEAD
[1.1.0]: https://github.com/o/r/compare/v1.0.0...v1.1.0
`;

describe('parseChangelog', () => {
  const entries = parseChangelog(SAMPLE);

  it('reads every version in order', () => {
    assert.deepEqual(
      entries.map((e) => [e.version, e.date, e.unreleased]),
      [
        ['Unreleased', null, true],
        ['1.1.0', '2026-12-01', false],
        ['1.0.0', '2026-11-12', false],
      ],
    );
  });

  it('keeps the body and drops link definitions', () => {
    assert.match(entries[1]!.body, /### Fixed\n- A crash\./);
    assert.equal(entries[2]!.body, 'First release.');
  });

  it('knows an empty section', () => {
    assert.equal(hasContent(entries[0]!), false);
    assert.equal(hasContent(entries[1]!), true);
  });
});
