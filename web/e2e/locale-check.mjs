// End-to-end check of the number format setting (docs/localisation.md), headless Chromium.
//   node e2e/locale-check.mjs [--url http://127.0.0.1:4872/] [--shots DIR]
//   (after npm run build, with a static server on calcpad-web/dist, e.g. python3 -m http.server)
//
//   1. the setting is a labelled control in the Settings dialog, Automatic by default;
//   2. each format (point, comma, space and comma, with and without digit groups) shows the same worksheet's
//      results in that format, and nothing but digits and separators differs from the point format;
//   3. the source of the worksheet and the saved input values stay canonical;
//   4. a number box takes 6,5 and 6.5 in a comma format, the engine gets 6.5, and the range message follows the format;
//   5. ?numfmt=comma applies without being stored; the stored setting survives a reload;
//   6. Automatic follows the browser language (de-DE comma, en-US point, fr-FR space and comma);
//   7. the HTML, Word and Excel exports follow the format and keep real numbers where there are cells.
import { execFileSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium, devices } from 'playwright';

const here = dirname(fileURLToPath(import.meta.url));
const web = resolve(here, '..');
const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const origin = arg('--url', 'http://127.0.0.1:4872/').replace(/\/$/, '');
const shots = arg('--shots', '');
if (shots) mkdirSync(shots, { recursive: true });
const failures = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) failures.push(what); };
let total = 0;
const ck = (ok, what) => { total++; check(ok, what); };

const NB = '\u00a0';
const browser = await chromium.launch();

async function openPage(opts = {}, query = '', viewport = { width: 1400, height: 900 }) {
    const context = await browser.newContext({ viewport, acceptDownloads: true, ...opts });
    await context.addInitScript(() => { if (window === window.top) { delete window.showSaveFilePicker; delete window.showOpenFilePicker; } });
    // This check drives the sheet's own controls in the input form, so the inputs are not docked (input-dock-check.mjs covers the dock).
    await context.addInitScript(() => { try { sessionStorage.setItem('calcpad.inputDock', 'off'); } catch { /* storage refused */ } });
    const page = await context.newPage();
    const errors = [];
    page.on('pageerror', (e) => { if (e.message !== 'Canceled' && e.message !== 'Event') errors.push(e.message); });
    await page.goto(`${origin}/${query}`);
    await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 60000 });
    return { page, context, errors };
}
const newTab = (page, text) => page.evaluate((t) => window.calcpadTabs.newUntitled(t), text);
const editorText = (page) => page.evaluate(() => window.calcpadActiveEditor.getValue());
const spaced = (s) => s.replace(/[ \t\r\n\u2000-\u200a]+/g, ' ');

/** Text of every result frame, newest first is not guaranteed (the preview is double buffered). */
async function frameTexts(page) {
    const out = [];
    for (const f of page.frames()) {
        if (f === page.mainFrame()) continue;
        out.push(await f.evaluate(() => document.body?.innerText ?? '').catch(() => ''));
    }
    return out;
}
/** Waits until some result frame holds `marker` and matches `pattern`; its text, or '' on a timeout. */
async function until(page, marker, pattern, tries = 80) {
    for (let i = 0; i < tries; i++) {
        for (const t of await frameTexts(page)) {
            // Hair, thin and em-fraction spaces round the operators and units read as plain spaces; no-break spaces are kept (digit groups).
            const n = t.replace(/[ \t\r\n\u2000-\u200a]+/g, ' ');
            if (t.includes(marker) && pattern.test(n)) return n;
        }
        await page.waitForTimeout(250);
    }
    return '';
}
async function openSettings(page) {
    const b = page.locator('[data-ribbon-btn=k-settings]').first();
    if (!await b.isVisible()) await page.locator('.ribbon-tab, [role=tab]', { hasText: /^\s*Calculation\s*$/ }).first().click();
    await b.click();
    // Calculation settings opens the Calculation tab of the dialog; Number format is on General.
    await page.locator('.tab-content.as-dialog [role=tab][data-settings-tab=general]').click();
    await page.locator('#cprs-numfmt-choice').waitFor({ timeout: 10000 });
}
async function closeSettings(page) {
    await page.keyboard.press('Escape');
    await page.locator('.tab-content.as-dialog').waitFor({ state: 'detached', timeout: 5000 }).catch(() => {});
}
/** Sets the format through the Settings dialog, the way a user does. */
async function choose(page, choice, grouping) {
    await openSettings(page);
    await page.selectOption('#cprs-numfmt-choice', choice);
    const box = page.locator('#cprs-numfmt-group');
    if ((await box.isChecked()) !== grouping) await box.click();
    await closeSettings(page);
}

const SOURCE = `"Prose with 1.5 and 12345.678 here
'<h3>Heading 2</h3>
a = 1234.5678
b = -0.00001234
c = 12345678.9
v = [1.5; 2.25; -3000.125]
M = [1.5; 2|3; 4.75]
L = 6.5m
F = 12.5kN
x1 = a*2 + b^2
y = a + 1000.5
a_1 = 3.14159
`;

// ---- 1 and 2. the control, and each format ----------------------------------------------------------------------
{
    const { page, errors } = await openPage({ locale: 'en-US' });
    await newTab(page, SOURCE);
    const base = await until(page, 'Prose with', /a = 1234\.57/);
    ck(/a = 1234\.57/.test(base) && /v = \[1\.5; 2\.25; -3000\.12\]/.test(base), `Automatic in en-US shows the point format (${base.slice(0, 70)})`);

    await openSettings(page);
    const label = await page.locator('label[for=cprs-numfmt-choice]').innerText();
    ck(/Number format/.test(label), `the control has a real label (${label})`);
    ck((await page.inputValue('#cprs-numfmt-choice')) === 'auto', 'the default choice is Automatic');
    ck(!(await page.isChecked('#cprs-numfmt-group')), 'digit grouping is off by default');
    const opts = await page.locator('#cprs-numfmt-choice option').allInnerTexts();
    ck(opts.length === 4 && /Automatic/.test(opts[0]) && /Point/.test(opts[1]) && /Comma/.test(opts[2]) && /Space and comma/.test(opts[3]), `four choices (${opts.join(' | ')})`);
    if (shots) await page.screenshot({ path: join(shots, 'settings-light-desktop.png') });
    await closeSettings(page);

    const strip = (t) => t.replace(/[\d.,\s\u00a0\u202f]/g, '');
    const baseText = base;
    const cases = [
        ['point', false, ['a = 1234.57', 'c = 12345679', 'v = [1.5; 2.25; -3000.12]', 'b = -1.23×10-5']],
        ['point', true, ['a = 1,234.57', 'c = 12,345,679', 'v = [1.5; 2.25; -3,000.12]', 'y = a + 1,000.5 = 1,234.57 + 1,000.5 = 2,235.07']],
        ['comma', false, ['a = 1234,57', 'c = 12345679', 'v = [1,5; 2,25; -3000,12]', 'b = -1,23×10-5', 'L = 6,5', 'F = 12,5']],
        ['comma', true, ['a = 1.234,57', 'c = 12.345.679', 'v = [1,5; 2,25; -3.000,12]', 'y = a + 1.000,5 = 1.234,57 + 1.000,5 = 2.235,07']],
        ['space', false, ['a = 1234,57', 'v = [1,5; 2,25; -3000,12]']],
        ['space', true, [`a = 1${NB}234,57`, `c = 12${NB}345${NB}679`, `v = [1,5; 2,25; -3${NB}000,12]`]],
    ];
    for (const [choice, grouping, wants] of cases) {
        await choose(page, choice, grouping);
        const want = wants[0];
        const t = await until(page, 'Prose with', new RegExp(want.replace(/[.*+?^${}()|[\]\\]/g, '\\$&').replace(/ /g, '[ \\u00a0]')));
        const tt = t.replace(/\u00a0/g, NB);
        ck(wants.every((w) => spaced(tt).includes(spaced(w))), `${choice}${grouping ? ' with digit groups' : ''}: ${wants.length} expected texts shown (${wants.filter((w) => !spaced(tt).includes(spaced(w))).join(' ; ') || 'all'})`);
        ck(strip(tt) === strip(baseText), `${choice}${grouping ? ' with groups' : ''}: only digits and separators differ from the point format`);
        ck(/Prose with 1\.5 and 12345\.678 here/.test(tt) && /Heading 2/.test(tt), `${choice}${grouping ? ' with groups' : ''}: authored text keeps its numbers as typed`);
        ck(/x1 = /.test(tt) && /a1 = 3[.,]14/.test(tt.replace('a_1', 'a1')) || /a1/.test(tt), `${choice}${grouping ? ' with groups' : ''}: names and subscripts are untouched`);
        const src = await editorText(page);
        ck(src === SOURCE, `${choice}${grouping ? ' with groups' : ''}: the worksheet source is unchanged`);
    }
    await choose(page, 'comma', true);
    await until(page, 'Prose with', /1\.234,57/);
    if (shots) await page.screenshot({ path: join(shots, 'results-comma-light.png') });
    // The value-only mode writes a bare number into the page.
    await newTab(page, '#val\nx = 1 + 2.5\n');
    const bare = await until(page, '3', /3,5/, 60);
    ck(/3,5/.test(bare), `a bare value-only result follows the format (${bare.slice(0, 30)})`);
    ck(errors.length === 0, `no page errors (${errors.slice(0, 2).join('; ')})`);
    await page.context().close();
}

// ---- 3 and 4. the number boxes ----------------------------------------------------------------------------------------
const BOX = `<!--sheet-->
# Box check

x: ? 1.5 m [0.5 .. 2.5 step 0.25] | the entry
y: x * 2 -> m | twice
k: ? 4.25 | a plain number
z: k + 1
`;
{
    const { page, errors } = await openPage({ locale: 'en-US' });
    const mode = (name) => page.locator('.preview-mode-group .toolbar-btn', { hasText: new RegExp(`^${name}$`) }).click({ timeout: 10000 });
    const form = async (tries = 60) => {
        for (let i = 0; i < tries; i++) {
            for (const f of page.frames()) {
                if (f === page.mainFrame()) continue;
                if (await f.$('input.calcpad-ui-input').catch(() => null)) return f;
            }
            await page.waitForTimeout(250);
        }
        return null;
    };
    // What a box SHOWS: the page's own value property is canonical on purpose, so the native getter is used.
    const boxes = async () => (await form())?.evaluate(() => {
        const get = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').get;
        return [...document.querySelectorAll('input.calcpad-ui-input')].map((e) => get.call(e));
    });
    /** Types into box `n`, Enter, then waits for the page to show `pattern`. */
    const type = async (n, text, pattern, tries = 4) => {
        for (let a = 0; a < tries; a++) {
            const f = await form();
            if (!f) continue;
            await page.waitForTimeout(500);
            await f.fill(`input.calcpad-ui-input >> nth=${n}`, text).catch(() => {});
            await f.press(`input.calcpad-ui-input >> nth=${n}`, 'Enter').catch(() => {});
            const t = await until(page, 'Box check', pattern, 24);
            if (t) return t;
        }
        console.log('  (last texts)', JSON.stringify((await frameTexts(page)).map((x) => spaced(x).slice(0, 200))));
        return '';
    };

    await newTab(page, BOX);
    await mode('Input');
    let b = await boxes();
    ck(JSON.stringify(b) === '["1.5","4.25"]', `Point format: the boxes show the canonical values (${JSON.stringify(b)})`);
    const pointT = await type(0, '1.75', /y = x · 2 = 1\.75 m · 2 = 3\.5 m/);
    ck(/= 3\.5 m/.test(pointT), `Point format: typing 1.75 gives y = 3.5 m (${pointT.slice(pointT.indexOf('twice') - 50, pointT.indexOf('twice') + 5)})`);

    await choose(page, 'comma', false);
    await mode('Input');
    b = await boxes();
    ck(JSON.stringify(b) === '["1,75","4,25"]', `Comma format: the boxes show 1,75 and 4,25 (${JSON.stringify(b)})`);
    const f1 = await form();
    const sl = await f1.evaluate(() => [...document.querySelectorAll('input.calcpad-ui-slider')].map((s) => [s.min, s.max, s.step, s.value]));
    ck(JSON.stringify(sl) === '[["0.5","2.5","0.25","1.75"]]', `the slider keeps its canonical numbers and follows the box (${JSON.stringify(sl)})`);
    if (shots) await page.screenshot({ path: join(shots, 'input-comma-light.png') });

    let t = await type(0, '1,25', /y = x · 2 = 1,25 m · 2 = 2,5 m/);
    ck(/= 2,5 m/.test(t), `typing 1,25 gives y = 2,5 m, so the engine got 1.25 (${t.slice(t.indexOf('twice') - 50, t.indexOf('twice') + 5)})`);
    b = await boxes();
    ck(b[0] === '1,25', `the box shows its value in the format after the re-run (${b[0]})`);
    t = await type(0, '1.5', /y = x · 2 = 1,5 m · 2 = 3 m/);
    ck(/= 3 m/.test(t), 'typing 1.5 is accepted as well and gives y = 3 m');
    t = await type(1, '6,5', /z = k \+ 1 = 6,5 \+ 1 = 7,5/);
    ck(/= 7,5/.test(t), `a plain number box takes 6,5 (${t.slice(t.indexOf('a plain'), t.indexOf('a plain') + 5) || t.slice(-60)})`);
    // The range message uses the format.
    {
        const f = await form();
        await f.fill('input.calcpad-ui-input >> nth=0', '9');
        await f.press('input.calcpad-ui-input >> nth=0', 'Enter');
        await page.waitForTimeout(800);
        const msg = await (await form()).evaluate(() => document.querySelector('.calcpad-ui-error')?.textContent ?? '');
        ck(msg === 'must be from 0,5 to 2,5', `the out-of-range message follows the format (${msg})`);
        if (shots) await page.screenshot({ path: join(shots, 'input-range-error-comma.png') });
        await f.fill('input.calcpad-ui-input >> nth=0', '1,25');
        await f.press('input.calcpad-ui-input >> nth=0', 'Enter');
        await page.waitForTimeout(600);
    }
    // A key press on the slider moves it by its step and the box shows the new value.
    {
        t = await until(page, 'Box check', /1,25 m/);
        const f = await form();
        await f.locator('input.calcpad-ui-slider').first().focus();
        await page.keyboard.press('ArrowRight');
        const t2 = await until(page, 'Box check', /y = x · 2 = 1,5 m · 2 = 3 m/);
        ck(/= 3 m/.test(t2), 'one slider key press moves 1,25 to 1,5 (step 0,25) and the result follows');
    }
    // Saved values and the source stay canonical.
    await mode('Input');
    await page.locator('.toolbar-btn', { hasText: /^Save values$/ }).click({ timeout: 10000 });
    await page.waitForTimeout(800);
    const saved = await editorText(page);
    const comment = saved.split('\n').slice(0, 2).join('\n');
    ck(/"x:1":"1\.5"/.test(comment) && /"k:1":"6\.5"/.test(comment) && !/\d,\d/.test(comment), `Save values writes canonical numbers (${comment.slice(0, 90)})`);
    ck(/x: \? 1\.5 m \[0\.5 \.\. 2\.5 step 0\.25\]/.test(saved), 'the sheet line keeps its decimal points and bounds');
    // The Export tab's HTML, Word and Excel, from the same render path.
    const out = await page.evaluate(async () => {
        const content = window.calcpadActiveEditor.getValue();
        const post = (ep, extra) => fetch(`/api/calcpad/${ep}`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ content, forPrint: true, ...extra }) });
        const uiOverrides = { 'x:1': '1.5', 'k:1': '6.5' };
        const html = await (await post('convert', { uiOverrides })).text();
        const doc = await post('docx', { uiOverrides });
        const xl = await post('xlsx', { uiOverrides });
        const bytes = async (r) => Array.from(new Uint8Array(await r.arrayBuffer()));
        return { html, docx: [doc.status, await bytes(doc)], xlsx: [xl.status, await bytes(xl)] };
    });
    const txt = spaced(out.html.replace(/<[^>]+>/g, ' '));
    ck(/z = k \+ 1 = 6,5 \+ 1 = 7,5/.test(txt), `the HTML export follows the format (${txt.slice(txt.indexOf('z ='), txt.indexOf('z =') + 30)})`);
    const dir = mkdtempSync(join(tmpdir(), 'ck-locale-'));
    writeFileSync(join(dir, 'a.docx'), Buffer.from(out.docx[1]));
    writeFileSync(join(dir, 'a.xlsx'), Buffer.from(out.xlsx[1]));
    ck(out.docx[0] === 200 && out.xlsx[0] === 200, 'the Word and Excel exports are produced');
    const unz = (f, p) => execFileSync('unzip', ['-p', join(dir, f), p], { maxBuffer: 1 << 26 }).toString('utf8');
    const docXml = unz('a.docx', 'word/document.xml');
    ck(/7,5/.test(docXml) && !/>7\.5</.test(docXml), 'the Word document shows 7,5 (the equations and the text)');
    const names = execFileSync('unzip', ['-Z1', join(dir, 'a.xlsx')]).toString();
    const sheets = names.split('\n').filter((n) => /xl\/worksheets\/sheet\d+\.xml/.test(n));
    const cells = sheets.map((s) => unz('a.xlsx', s)).join('\n');
    const numeric = [...cells.matchAll(/<c [^>]*(?<!t="[a-z]+")>\s*<v>([^<]*)<\/v>/g)].map((m) => m[1]);
    ck(numeric.includes('6.5') && numeric.every((v) => !v.includes(',')), `the Excel numeric cells stay real numbers with a point (${numeric.slice(0, 8).join(' ')})`);
    ck(errors.length === 0, `no page errors (${errors.slice(0, 2).join('; ')})`);
    await page.context().close();
}

// ---- 5. the link, and the stored setting across a reload -------------------------------------------------------------------
{
    const { page, errors } = await openPage({ locale: 'en-US' }, '?numfmt=comma');
    await newTab(page, SOURCE);
    const t = await until(page, 'Prose with', /a = 1234,57/);
    ck(/a = 1234,57/.test(t), '?numfmt=comma shows the comma format');
    const stored = await page.evaluate(() => localStorage.getItem('calcpad-number-format'));
    ck(stored === null, `the link does not store anything (${stored})`);
    await page.context().close();
}
{
    const { page, errors } = await openPage({ locale: 'en-US' }, '?numfmt=space&numgroup=1');
    await newTab(page, SOURCE);
    const t = await until(page, 'Prose with', /a = 1.234,57/);
    ck(t.includes(`a = 1${NB}234,57`), '?numfmt=space&numgroup=1 shows the space format with digit groups');
    await openSettings(page);
    ck((await page.locator('.cprs-override').count()) === 1, 'the Settings page says the link overrides the choice');
    await page.context().close();
}
{
    const { page, context } = await openPage({ locale: 'en-US' });
    await newTab(page, SOURCE);
    await choose(page, 'comma', true);
    const raw = await page.evaluate(() => localStorage.getItem('calcpad-number-format'));
    ck(raw === '{"choice":"comma","grouping":true}', `the choice is stored (${raw})`);
    await page.reload();
    await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 60000 });
    await newTab(page, SOURCE);
    const t = await until(page, 'Prose with', /a = 1\.234,57/);
    ck(/a = 1\.234,57/.test(t), 'after a reload the comma format with groups is still in force');
    await openSettings(page);
    ck((await page.inputValue('#cprs-numfmt-choice')) === 'comma' && (await page.isChecked('#cprs-numfmt-group')), 'and the Settings page shows it');
    await context.close();
}

// ---- 6. Automatic follows the browser language ------------------------------------------------------------------------------------
for (const [locale, pattern, what] of [['de-DE', /a = 1234,57/, 'de-DE: comma'], ['en-US', /a = 1234\.57/, 'en-US: point'], ['fr-FR', /a = 1234,57/, 'fr-FR: comma'], ['pt-BR', /a = 1234,57/, 'pt-BR: comma']]) {
    const { page, context } = await openPage({ locale });
    await newTab(page, SOURCE);
    const t = await until(page, 'Prose with', pattern);
    ck(pattern.test(t), `Automatic with the browser language ${what}`);
    if (locale === 'de-DE') {
        await openSettings(page);
        const opt = await page.locator('#cprs-numfmt-choice option').first().innerText();
        ck(/comma/.test(opt), `the Automatic entry names what it resolves to (${opt})`);
        await page.selectOption('#cprs-numfmt-choice', 'point');
        await page.locator('#cprs-numfmt-group').check();
        await closeSettings(page);
        const t2 = await until(page, 'Prose with', /a = 1,234\.57/);
        ck(/a = 1,234\.57/.test(t2), 'an explicit Point beats the browser language');
    }
    await context.close();
}

// ---- 7. dark theme and phone layout (screenshots) ---------------------------------------------------------------------------------------
if (shots) {
    {
        const { page, context } = await openPage({ locale: 'en-US', colorScheme: 'dark' });
        await newTab(page, SOURCE);
        await choose(page, 'comma', true);
        await openSettings(page);
        await page.screenshot({ path: join(shots, 'settings-dark-desktop.png') });
        await closeSettings(page);
        await until(page, 'Prose with', /1\.234,57/);
        await page.screenshot({ path: join(shots, 'results-comma-dark.png') });
        await context.close();
    }
    {
        const { page, context } = await openPage({ ...devices['iPhone 14'], locale: 'en-US' });
        await openSettings(page);
        const bb = await page.locator('#cprs-numfmt-choice').boundingBox();
        ck(bb != null && bb.x >= 0 && bb.x + bb.width <= 390 + 1, `phone: the control fits the screen (${bb && Math.round(bb.x)}..${bb && Math.round(bb.x + bb.width)} of 390)`);
        await page.screenshot({ path: join(shots, 'settings-phone.png') });
        await context.close();
    }
}

await browser.close();
console.log(`\n${total - failures.length}/${total} checks passed`);
if (failures.length) {
    console.log('FAILED:\n  ' + failures.join('\n  '));
    process.exit(1);
}
