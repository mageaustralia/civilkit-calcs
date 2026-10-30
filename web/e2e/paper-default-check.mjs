// The default PDF paper by browser region, without a browser: Letter for US, CA, MX, PH, CL, CO, VE and CR, A4 elsewhere,
// and a saved paper size always wins.
//   node e2e/paper-default-check.mjs
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const web = resolve(dirname(fileURLToPath(import.meta.url)), '..');
let fails = 0;
const eq = (got, want, what) => {
    const ok = got === want;
    if (!ok) fails++;
    console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}${ok ? '' : ` (got ${got}, want ${want})`}`);
};

const { tsImport } = await import(pathToFileURL(join(web, 'node_modules', 'tsx', 'dist', 'esm', 'api', 'index.mjs')).href);
const m = await tsImport(pathToFileURL(join(web, 'calcpad-frontend', 'src', 'types', 'pdf-settings.ts')).href, import.meta.url);

const letter = ['en-US', 'en-CA', 'fr-CA', 'es-MX', 'en-PH', 'es-CL', 'es-CO', 'es-VE', 'es-CR', 'en', 'en-us'];
const a4 = ['en-AU', 'en-GB', 'en-NZ', 'de', 'de-DE', 'fr', 'fr-FR', 'es', 'es-ES', 'pt-BR', 'zh-Hans', 'zh-CN', 'ja', 'en-IN'];
for (const l of letter) eq(m.defaultPaperFormat(l), 'Letter', `${l} is Letter`);
for (const l of a4) eq(m.defaultPaperFormat(l), 'A4', `${l} is A4`);
eq(m.defaultPaperFormat('not a tag!'), 'Letter', 'an unreadable tag falls back to Letter');

Object.defineProperty(globalThis, 'navigator', { value: { language: 'en-AU' }, configurable: true });
eq(m.resolveStoredPdfSettings(undefined).format, 'A4', 'nothing saved, en-AU browser: A4');
eq(m.resolveStoredPdfSettings({}).format, 'A4', 'an empty saved object: A4');
eq(m.resolveStoredPdfSettings({ format: 'Letter' }).format, 'Letter', 'a saved Letter wins over the region');
eq(m.resolveStoredPdfSettings({ format: 'A3' }).format, 'A3', 'a saved A3 wins over the region');
Object.defineProperty(globalThis, 'navigator', { value: { language: 'en-US' }, configurable: true });
eq(m.resolveStoredPdfSettings({ format: 'A4' }).format, 'A4', 'a saved A4 wins in a US browser');
eq(m.resolveStoredPdfSettings({}).format, 'Letter', 'nothing saved, en-US browser: Letter');
eq(m.resolveEffectivePdfSettings({}, { format: 'A5' }).format, 'A5', 'the document\'s own paper wins');

console.log(fails ? `${fails} failed` : 'all passed');
process.exit(fails ? 1 : 0);
