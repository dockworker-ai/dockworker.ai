import { cp, mkdir, readFile, writeFile } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const dist = join(root, 'dist');
const outFrontend = join(root, '..', 'frontend', 'dist');

const GITHUB_OLD = 'https://github.com/lornu-ai/dockworker.ai-design';
const GITHUB_NEW = 'https://github.com/dockworker-ai/dockworker.ai';

await mkdir(dist, { recursive: true });
await cp(join(root, 'index.html'), join(dist, 'index.html'));
await cp(join(root, 'assets'), join(dist, 'assets'), { recursive: true });

const jsPath = join(dist, 'assets', 'index-BMT5qsGB.js');
let js = await readFile(jsPath, 'utf8');
if (!js.includes(GITHUB_OLD) && !js.includes(GITHUB_NEW)) {
  console.warn('warning: expected GitHub URL not found in landing bundle');
}
js = js.split(GITHUB_OLD).join(GITHUB_NEW);
await writeFile(jsPath, js);

await writeFile(
  join(dist, '_redirects'),
  [
    '# Console routes belong on a future app host — apex is marketing only.',
    '/login / 302',
    '/dashboard / 302',
    '/settings / 302',
    '/builds/* / 302',
    '/auth/* / 302',
  ].join('\n') + '\n',
);

await mkdir(outFrontend, { recursive: true });
await cp(dist, outFrontend, { recursive: true });
console.log('landing → frontend/dist (Cloudflare Pages root_dir=frontend)');
