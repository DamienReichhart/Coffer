/** The docs sidebar. Slugs are relative to src/content/docs. */
export const sidebar = [
  {
    label: 'Getting started',
    items: [
      { label: 'Introduction', slug: 'docs' },
      { label: 'Installation', slug: 'docs/installation' },
      { label: 'Installing 7-Zip', slug: 'docs/installing-7zip' },
      { label: 'Quick start', slug: 'docs/quick-start' },
    ],
  },
  {
    label: 'Guides',
    items: [
      { label: 'Opening and browsing', slug: 'docs/guides/browsing' },
      { label: 'Extracting', slug: 'docs/guides/extracting' },
      { label: 'Creating archives', slug: 'docs/guides/creating' },
      { label: 'Testing archives', slug: 'docs/guides/testing' },
      { label: 'Checksums', slug: 'docs/guides/checksums' },
      { label: 'Activity and cancelling', slug: 'docs/guides/activity' },
      { label: 'Drag and drop', slug: 'docs/guides/drag-and-drop' },
      { label: 'Command line', slug: 'docs/guides/command-line' },
    ],
  },
  {
    label: 'Reference',
    items: [
      { label: 'Settings', slug: 'docs/reference/settings' },
      { label: 'Automatic updates', slug: 'docs/reference/updates' },
      { label: 'Verifying downloads', slug: 'docs/reference/verifying' },
      { label: 'Security and privacy', slug: 'docs/reference/security' },
      { label: 'Supported formats', slug: 'docs/reference/formats' },
      { label: 'Troubleshooting', slug: 'docs/reference/troubleshooting' },
      { label: 'FAQ', slug: 'docs/reference/faq' },
    ],
  },
  {
    label: 'Project',
    items: [
      { label: 'Building from source', slug: 'docs/project/building' },
      { label: 'Contributing', slug: 'docs/project/contributing' },
      { label: 'Releasing', slug: 'docs/project/releasing' },
      { label: 'Changelog', link: '/changelog/' },
    ],
  },
];
