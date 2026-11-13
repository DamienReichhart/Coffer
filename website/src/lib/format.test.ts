import assert from 'node:assert/strict';
import { describe, it } from 'node:test';
import { changelogAnchor, formatBytes, formatDate } from './format.ts';

describe('format', () => {
  it('formats sizes', () => {
    assert.equal(formatBytes(0), '0 B');
    assert.equal(formatBytes(1023), '1023 B');
    assert.equal(formatBytes(1536), '1.5 KB');
    assert.equal(formatBytes(12 * 1024 * 1024), '12.0 MB');
    assert.equal(formatBytes(-1), '—');
  });
  it('formats dates in UTC', () => {
    assert.equal(formatDate('2026-11-12T23:30:00Z'), '12 November 2026');
    assert.equal(formatDate(null), '');
  });
  it('matches the sync script anchors', () => {
    assert.equal(changelogAnchor('1.2.3'), 'v1-2-3');
  });
});
