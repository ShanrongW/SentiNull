import { readFile } from 'node:fs/promises';
import { extname, join } from 'node:path';

const contentTypes = new Map([
  ['.html', 'text/html; charset=utf-8'],
  ['.css', 'text/css; charset=utf-8'],
  ['.mjs', 'text/javascript; charset=utf-8'],
]);

export async function findAsset(root, pathname) {
  const filename = pathname === '/' ? 'index.html' : pathname.slice(1);
  const segments = filename.split('/');
  if (segments.some((part) => !/^[a-z\d][a-z\d._-]*$/i.test(part))) return null;
  const contentType = contentTypes.get(extname(filename));
  if (!contentType) return null;
  try {
    return { contentType, body: await readFile(join(root, ...segments)) };
  } catch (error) {
    if (error.code === 'ENOENT' || error.code === 'ENOTDIR' || error.code === 'EISDIR') return null;
    throw error;
  }
}
