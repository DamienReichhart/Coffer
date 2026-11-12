import assert from 'node:assert/strict';
import { describe, it } from 'node:test';
import { archFrom, osFrom, platformKey } from './platform.ts';

describe('platform', () => {
  it('reads the OS', () => {
    assert.equal(osFrom('Win32 Mozilla/5.0 (Windows NT 10.0; Win64; x64)'), 'windows');
    assert.equal(osFrom('MacIntel Mozilla/5.0 (Macintosh; Intel Mac OS X 14_0)'), 'macos');
    assert.equal(osFrom('Linux x86_64 Mozilla/5.0 (X11; Linux x86_64)'), 'linux');
    assert.equal(osFrom('Linux armv8l Mozilla/5.0 (Linux; Android 14)'), 'other');
  });
  it('reads the architecture when the user agent tells', () => {
    assert.equal(archFrom('Mozilla/5.0 (X11; Linux aarch64)', 'linux'), 'arm64');
    assert.equal(archFrom('Mozilla/5.0 (Windows NT 10.0; Win64; x64)', 'windows'), 'x64');
    assert.equal(archFrom('Mozilla/5.0 (Macintosh; Intel Mac OS X 14_0)', 'macos'), undefined);
  });
  it('guesses the likeliest architecture', () => {
    assert.equal(platformKey({ os: 'macos', arch: undefined }), 'macos-arm64');
    assert.equal(platformKey({ os: 'windows', arch: undefined }), 'windows-x64');
    assert.equal(platformKey({ os: 'linux', arch: 'arm64' }), 'linux-arm64');
    assert.equal(platformKey({ os: 'other', arch: undefined }), undefined);
  });
});
