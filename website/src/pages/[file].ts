/*
 * The package repositories' public key and the dnf/zypper repository file, at
 * the site's root. Only built when the key is in the repository
 * (packaging/linux/7zip-gui.asc): without it there are no repositories.
 */
import type { APIRoute, GetStaticPaths } from 'astro';
import { repoFile, repositoryKey } from '@/lib/packages';

export const getStaticPaths: GetStaticPaths = () =>
  repositoryKey ? [{ params: { file: '7zip-gui.asc' } }, { params: { file: '7zip-gui.repo' } }] : [];

export const GET: APIRoute = ({ params, site }) => {
  const root = new URL(import.meta.env.BASE_URL, site).href.replace(/\/$/, '');
  const body = params.file === '7zip-gui.asc' ? (repositoryKey ?? '') : repoFile(root);
  return new Response(body, { headers: { 'Content-Type': 'text/plain; charset=utf-8' } });
};
