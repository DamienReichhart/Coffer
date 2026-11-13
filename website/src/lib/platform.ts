/**
 * Best-effort detection of the visitor's OS and CPU architecture, in the browser.
 * User-Agent Client Hints give the architecture in Chromium browsers; elsewhere
 * it is only known when the user agent says so (Safari on Apple silicon still
 * reports "Intel", so a Mac without hints is assumed to be Apple silicon, which
 * every Mac sold since 2023 is).
 */
export type DetectedOs = 'windows' | 'macos' | 'linux' | 'other';
export type DetectedArch = 'x64' | 'arm64' | undefined;

export interface Platform {
  os: DetectedOs;
  arch: DetectedArch;
}

export function osFrom(text: string): DetectedOs {
  const s = text.toLowerCase();
  if (/android|iphone|ipad|ipod|cros/.test(s)) return 'other';
  if (s.includes('win')) return 'windows';
  if (s.includes('mac')) return 'macos';
  if (s.includes('linux') || s.includes('x11')) return 'linux';
  return 'other';
}

export function archFrom(ua: string, os: DetectedOs): DetectedArch {
  if (/aarch64|arm64|armv8/i.test(ua)) return 'arm64';
  if (/x86_64|x64|win64|wow64|amd64/i.test(ua) && os !== 'macos') return 'x64';
  return undefined;
}

/** The key `recommended()` in lib/releases.ts uses, filling in the likeliest architecture. */
export function platformKey({ os, arch }: Platform): string | undefined {
  if (os === 'other') return undefined;
  return `${os}-${arch ?? (os === 'macos' ? 'arm64' : 'x64')}`;
}

interface UAData {
  platform: string;
  getHighEntropyValues(hints: string[]): Promise<{ platform?: string; architecture?: string }>;
}

export async function detectPlatform(): Promise<Platform> {
  const ua = navigator.userAgent;
  const isIpad = /Macintosh/.test(ua) && navigator.maxTouchPoints > 1;
  const uaData = (navigator as Navigator & { userAgentData?: UAData }).userAgentData;
  if (uaData) {
    try {
      const values = await uaData.getHighEntropyValues(['architecture', 'platform']);
      const os = osFrom(values.platform || uaData.platform);
      const arch =
        values.architecture === 'arm' ? 'arm64' : values.architecture === 'x86' ? 'x64' : undefined;
      return { os: isIpad ? 'other' : os, arch };
    } catch {
      // Fall through to the user agent string.
    }
  }
  const os = isIpad ? 'other' : osFrom(`${navigator.platform} ${ua}`);
  return { os, arch: archFrom(ua, os) };
}
