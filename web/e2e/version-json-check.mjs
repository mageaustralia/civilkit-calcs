// The build writes dist/version.json with the version of Cargo.toml, and the service worker does not precache it.
//   node e2e/version-json-check.mjs [dist]   (after a build)
import { readFileSync, existsSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const dist = resolve(process.argv[2] ?? join(here, '..', 'calcpad-web', 'dist'));
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };
const toml = readFileSync(join(here, '..', '..', 'Cargo.toml'), 'utf8');
const want = /\[workspace\.package\][^\[]*?\nversion\s*=\s*"([^"]+)"/.exec(toml)?.[1];
check(!!want, `Cargo.toml has a workspace version (${want})`);
const f = join(dist, 'version.json');
check(existsSync(f), 'dist/version.json exists');
if (existsSync(f)) {
    const j = JSON.parse(readFileSync(f, 'utf8'));
    check(j.version === want, `version.json says ${j.version}, Cargo.toml says ${want}`);
    check(/^\d{2}\.([1-9]|1[0-2])\.[1-9]\d*$/.test(j.version), 'the version has the YY.M.N form');
}
const sw = existsSync(join(dist, 'sw.js')) ? readFileSync(join(dist, 'sw.js'), 'utf8') : '';
check(sw !== '' && !sw.includes('version.json'), 'the service worker does not precache version.json');
const headers = readFileSync(join(here, '..', 'calcpad-web', 'cloudflare', '_headers'), 'utf8');
check(/\/calcs\/version\.json\s+Cache-Control: public, max-age=\d+/.test(headers), 'version.json has a short cache header');
console.log(fails.length ? `${fails.length} FAIL` : 'all ok');
process.exit(fails.length ? 1 : 0);
