import { href } from '@/lib/href';

/** The legal pages every page links to. */
export function legalLinks() {
  return [
    { label: 'Legal notice', url: href('/legal-notice/') },
    { label: 'Privacy', url: href('/privacy/') },
  ];
}
