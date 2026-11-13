import js from '@eslint/js';
import { defineConfig } from 'eslint/config';
import astro from 'eslint-plugin-astro';
import globals from 'globals';
import tseslint from 'typescript-eslint';

const sources = '**/*.{js,mjs,ts,astro}';

export default defineConfig(
  { ignores: ['dist', '.astro', 'node_modules', 'public'] },
  {
    files: [sources],
    extends: [js.configs.recommended, ...tseslint.configs.recommended],
    rules: {
      '@typescript-eslint/consistent-type-imports': 'error',
    },
  },
  {
    // Build-time code (configs, scripts, tests) runs on Node.
    files: ['*.{js,mjs,ts}', 'scripts/**', 'src/**/*.test.ts'],
    languageOptions: { globals: globals.node },
  },
  ...astro.configs.recommended,
  {
    // Past 300 lines a file usually holds more than one thing: split it.
    files: [`src/${sources}`],
    rules: { 'max-lines': ['warn', { max: 300, skipBlankLines: true, skipComments: true }] },
  },
);
