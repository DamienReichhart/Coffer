/**
 * A Sätteri hast plugin that prefixes root-relative links in Markdown and MDX
 * with the site's base path, so docs can link to `/docs/installation/` and work
 * whether the site is served from `/Coffer/` (a GitHub Pages project site) or
 * from the root of a custom domain. Covers plain links and <LinkCard href>.
 *
 * @param {string} base  Astro's `base`, e.g. `/Coffer`.
 * Untyped on purpose: Starlight and @astrojs/markdown-satteri depend on two
 * different satteri versions, whose plugin types don't match each other.
 *
 * @returns {any}
 */
export default function baseLinks(base) {
  const prefix = base.replace(/\/$/, '');

  /** @param {unknown} href */
  const rewrite = (href) => {
    if (typeof href !== 'string' || !href.startsWith('/') || href.startsWith('//'))
      return undefined;
    if (!prefix || href === prefix || href.startsWith(`${prefix}/`)) return undefined;
    return `${prefix}${href}`;
  };

  return {
    name: 'coffer-base-links',
    element: {
      filter: ['a'],
      visit(node, ctx) {
        const href = rewrite(node.properties?.href);
        if (href) ctx.setProperty(node, 'href', href);
      },
    },
    mdxJsxFlowElement: {
      filter: ['LinkCard'],
      visit(node) {
        let changed = false;
        const attributes = node.attributes.map((attribute) => {
          if (attribute.type !== 'mdxJsxAttribute' || attribute.name !== 'href') return attribute;
          const href = rewrite(attribute.value);
          if (!href) return attribute;
          changed = true;
          return { ...attribute, value: href };
        });
        return changed ? { ...node, attributes } : undefined;
      },
    },
  };
}
