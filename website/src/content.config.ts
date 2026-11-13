import { defineCollection } from 'astro:content';
import { docsLoader, i18nLoader } from '@astrojs/starlight/loaders';
import { docsSchema, i18nSchema } from '@astrojs/starlight/schema';

export const collections = {
  docs: defineCollection({ loader: docsLoader(), schema: docsSchema() }),
  // Starlight's interface strings. English only; src/content/i18n/en.json
  // overrides any of them.
  i18n: defineCollection({ loader: i18nLoader(), schema: i18nSchema() }),
};
