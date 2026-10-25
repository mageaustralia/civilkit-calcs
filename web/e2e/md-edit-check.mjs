// End-to-end check of rich-text editing of a `#md on` block: double-click a Markdown paragraph in the
// preview, edit it in the popover (Milkdown), apply, and look at the source. Equations still edit as lines.
//
//   node e2e/md-edit-check.mjs [--port 4181] [--dist DIR]
import { createServer } from 'node:http';
import { existsSync, readFileSync, statSync } from 'node:fs';
import { dirname, extname, join, normalize, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright';

const here = dirname(fileURLToPath(import.meta.url));
const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const port = Number(arg('--port', '4181'));
const dist = resolve(arg('--dist', join(here, '..', 'calcpad-web', 'dist')));
const failures = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) failures.push(what); };

const TYPES = { '.html': 'text/html', '.js': 'text/javascript', '.mjs': 'text/javascript', '.wasm': 'application/wasm', '.css': 'text/css', '.json': 'application/json', '.png': 'image/png', '.svg': 'image/svg+xml', '.woff2': 'font/woff2', '.ttf': 'font/ttf', '.otf': 'font/otf' };
const server = createServer((req, res) => {
    let p = decodeURIComponent(new URL(req.url, `http://127.0.0.1:${port}`).pathname);
    if (p.endsWith('/')) p += 'index.html';
    const f = normalize(join(dist, p));
    if (!f.startsWith(dist) || !existsSync(f) || !statSync(f).isFile()) { res.writeHead(404); return res.end(); }
    res.writeHead(200, { 'Content-Type': TYPES[extname(f)] ?? 'application/octet-stream' });
    res.end(readFileSync(f));
});
await new Promise(ok => server.listen(port, '127.0.0.1', ok));

const browser = await chromium.launch();
const page = await (await browser.newContext({ viewport: { width: 1500, height: 900 } })).newPage();
const errors = [];
page.on('pageerror', e => errors.push(e.message));
await page.goto(`http://127.0.0.1:${port}/`);
await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 60000 });

const src = ["'before", '#markdown', '## Design basis', '', 'The **load** is _applied_ at f~c~ midspan, x^2^.', '#end markdown', 'a = 2', 'b = a + 3', 'b'].join('\n') + '\n';
await page.evaluate(v => window.calcpadActiveEditor.setValue(v), src);
const line = n => page.evaluate(i => window.calcpadActiveEditor.getModel().getLineContent(i), n);
const all = () => page.evaluate(() => window.calcpadActiveEditor.getValue());
const findFrame = async (re) => {
    for (let i = 0; i < 40; i++) {
        for (const f of page.frames().slice(1)) {
            try { if (await f.evaluate(s => new RegExp(s).test(document.body.innerText) && !!document.querySelector('[data-source-line]'), re)) return f; } catch { /* frame swapping */ }
        }
        await page.waitForTimeout(500);
    }
    return null;
};
let fr = await findFrame('Design basis');
check(!!fr, 'the Markdown block renders in the preview');
const lines = fr ? await fr.evaluate(() => [...document.querySelectorAll('[data-source-line]')].map(e => e.getAttribute('data-source-line') + ':' + e.textContent.slice(0, 20).replace(/\s+/g, ' '))) : [];
console.log('  tagged lines:', lines.join(' | '));
check(fr && await fr.evaluate(() => !!document.querySelector('sub') && !!document.querySelector('sup')), 'f~c~ and x^2^ render as subscript and superscript');

// 1. Double-click the paragraph: the rich-text editor opens with the block's text.
const geo = () => fr.evaluate(() => { const r = n => document.querySelector(`[data-source-line="${n}"]`).getBoundingClientRect(); return { top: Math.round(r(3).top), left: Math.round(r(3).left), next: Math.round(r(7).top) }; });
const g0 = await geo();
await fr.locator('[data-source-line="5"]').first().dblclick();
await page.waitForSelector('.md-edit .ProseMirror', { timeout: 15000 });
check(true, 'double-click on a Markdown paragraph opens the rich-text editor');
const shown = await page.locator('.md-edit .ProseMirror').innerText();
check(/Design basis/.test(shown) && /applied at/.test(shown), `the editor shows the whole block (${JSON.stringify(shown.slice(0, 60))})`);
check(await page.locator('.md-edit .ProseMirror h2').count() === 1 && await page.locator('.md-edit .ProseMirror strong').count() === 1, 'heading and bold are shown as formatting, not as symbols');
await page.screenshot({ path: join(process.env.TMPDIR ?? '/tmp', 'md-edit-open.png') });
// The editor opens in place of the text, and the preview makes room for it.
await page.waitForTimeout(500);
const eb = await page.locator('.md-edit').boundingBox();
const fb = await page.locator('iframe').evaluateAll(l => l.filter(f => f.getBoundingClientRect().width > 0).map(f => { const r = f.getBoundingClientRect(); return { left: r.left, top: r.top }; })[0]);
check(Math.abs(eb.y - (fb.top + g0.top)) < 3 && Math.abs(eb.x - (fb.left + g0.left)) < 3, `the editor opens over the rendered text (editor ${Math.round(eb.x)},${Math.round(eb.y)} vs text ${Math.round(fb.left + g0.left)},${Math.round(fb.top + g0.top)})`);
const g1 = await geo();
check(fb.top + g1.next >= eb.y + eb.height - 2 && g1.next > g0.next, `the rest of the page starts below the editor, not under it (next line at ${Math.round(fb.top + g1.next)}, editor ends ${Math.round(eb.y + eb.height)})`);
await page.keyboard.press('Escape');
await page.waitForTimeout(400);
const g2 = await geo();
check(g2.next === g0.next && g2.top === g0.top, 'cancelling gives the page back exactly as it was');
fr = await findFrame('Design basis');
await fr.locator('[data-source-line="5"]').first().dblclick();
await page.waitForSelector('.md-edit .ProseMirror', { timeout: 15000 });

// 2. Apply without a change: the source is untouched.
await page.locator('.md-edit button.primary').click();
await page.waitForTimeout(300);
check(await all() === src, 'applying without a change leaves the source exactly as it was');

// 3. Edit: type at the end of the paragraph, apply.
fr = await findFrame('Design basis');
await fr.locator('[data-source-line="5"]').first().dblclick();
await page.waitForSelector('.md-edit .ProseMirror');
await page.locator('.md-edit .ProseMirror p').first().click();
await page.keyboard.press('End');
await page.keyboard.type(' Check the deflection.');
await page.locator('.md-edit button.primary').click();
await page.waitForTimeout(300);
check(await line(3) === '## Design basis' && await line(4) === '', 'the untouched heading keeps its source line');
check(await line(5) === 'The **load** is _applied_ at f~c~ midspan, x^2^. Check the deflection.', `the edited paragraph keeps ~sub~ and ^sup^ as raw Markdown (${JSON.stringify(await line(5))})`);
check(await line(2) === '#markdown' && await line(6) === '#end markdown' && await line(7) === 'a = 2', 'the block markers and the rest of the sheet are unchanged');
fr = await findFrame('Check the deflection');
check(!!fr, 'the preview shows the edited text');
await page.evaluate(() => { window.calcpadActiveEditor.focus(); window.calcpadActiveEditor.trigger('t', 'undo', null); });
check(await all() === src, 'one undo restores the block');

// 4. Tables: insert one with the toolbar, fill a cell, apply; the source is a plain pipe table.
fr = await findFrame('Design basis');
await fr.locator('[data-source-line="5"]').first().dblclick();
await page.waitForSelector('.md-edit .ProseMirror');
await page.locator('.md-edit .ProseMirror p').first().click();
await page.keyboard.press('End');
await page.locator('.md-edit .md-block-bar button', { hasText: 'Table' }).click();
await page.waitForSelector('.md-edit .ProseMirror table');
check(await page.locator('.md-edit .ProseMirror table tr').count() === 3, 'the Table button inserts a 3 x 3 table');
await page.locator('.md-edit .ProseMirror table th, .md-edit .ProseMirror table td').first().click();
await page.keyboard.type('Member');
await page.keyboard.press('Tab');
await page.keyboard.type('Load');
await page.locator('.md-edit .md-block-bar button', { hasText: '+ Row' }).click();
check(await page.locator('.md-edit .ProseMirror table tr').count() === 4, '+ Row adds a row');
await page.locator('.md-edit button.primary').click();
await page.waitForTimeout(400);
const text = await all();
console.log(text.split('\n').slice(0, 14).map(l => '    ' + l).join('\n'));
check(/^\| Member +\| Load +\| *\|$/m.test(text) && /^\| *-{3,}[ |:-]*$/m.test(text), 'the table is written as a plain Markdown pipe table, no HTML');
check(!/<table|<tr|<td/.test(text), 'no HTML in the file');
fr = await findFrame('Member');
check(!!fr && await fr.evaluate(() => document.querySelectorAll('table tr').length) === 4, 'the preview renders it as a table');
const rules = await fr.evaluate(() => { const td = document.querySelector('table td'); const th = document.querySelector('table th'); const c = getComputedStyle(td); return { td: c.borderTopWidth + ' ' + c.borderTopStyle, pad: c.paddingLeft, thBg: getComputedStyle(th).backgroundColor }; });
check(rules.td === '1px solid' && rules.pad !== '0px' && rules.thBg !== 'rgba(0, 0, 0, 0)', `a table has rules, padding and a shaded header (${JSON.stringify(rules)})`);
await page.evaluate(() => { window.calcpadActiveEditor.focus(); window.calcpadActiveEditor.trigger('t', 'undo', null); });
check(await all() === src, 'undo removes the table in one step');

// 5. Esc cancels; an equation still edits as a source line.
fr = await findFrame('Design basis');
await fr.locator('[data-source-line="5"]').first().dblclick();
await page.waitForSelector('.md-edit .ProseMirror');
await page.keyboard.press('Escape');
await page.waitForTimeout(200);
check(await page.locator('.md-edit').count() === 0 && await all() === src, 'Esc cancels without a change');
await fr.locator('[data-source-line="8"]').first().dblclick();
await page.waitForSelector('.line-edit-input', { timeout: 5000 });
check(await page.locator('.md-edit').count() === 0 && (await page.locator('.line-edit-input').inputValue()) === 'b = a + 3', 'an equation line still opens the source-line editor');
await page.keyboard.press('Escape');

// 6. The older `#md on` form still opens, and keeps inline HTML.
await page.evaluate(v => window.calcpadActiveEditor.setValue(v), ["#md on", "'**f**<sub>c</sub> = 4 ksi at 28 d, a\\_b and x<sup>2</sup>", '#md off', 'q = 1'].join('\n') + '\n');
fr = await findFrame('ksi');
await fr.locator('[data-source-line="2"]').first().dblclick();
await page.waitForSelector('.md-edit .ProseMirror');
await page.locator('.md-edit .ProseMirror p').first().click();
await page.keyboard.press('End');
await page.keyboard.type(' ok');
await page.locator('.md-edit button.primary').click();
await page.waitForTimeout(300);
const rich = await line(2);
console.log('  older-form line:', rich);
check(/<sub>c<\/sub>/.test(rich) || /~c~/.test(rich), 'the older #md on form still edits, and keeps its subscript');
check(/<sup>2<\/sup>/.test(rich) || /\^2\^/.test(rich), '... and its superscript');
check(/ok$/.test(rich), '... and the typed text');

// A real double-click at the element's place: the preview keeps two frames, and the one that holds the
// element may be the hidden one, so the click goes to whatever is on top, as a person's would.
const dbl = async (frame, n) => {
    const box = await frame.locator(`[data-source-line="${n}"]`).first().boundingBox();
    await page.mouse.dblclick(box.x + Math.min(20, box.width / 2), box.y + Math.min(8, box.height / 2));
};
// 8. A sheet (<!--sheet-->) needs no markers: its prose runs edit as rich text, inputs stay one line.
const sheetSrc = ['<!--sheet-->', '# Design basis', '', 'The **load** is _applied_ at f~c~ midspan.', '', 'a: 3 m | first side', 'b: 4 m | second side', '', '| Member | Load |', '| --- | --- |', '| AB | 5 kN |', '', 'c: sqrt(a^2 + b^2) | hypotenuse'].join('\n') + '\n';
await page.evaluate(v => window.calcpadActiveEditor.setValue(v), sheetSrc);
fr = await findFrame('first side');
console.log('  sheet tagged lines:', (await fr.evaluate(() => [...document.querySelectorAll('[data-source-line]')].map(e => e.getAttribute('data-source-line') + ':' + e.tagName + ':' + e.textContent.slice(0, 14).replace(/\s+/g, ' ')))).join(' | '));
await fr.locator('[data-source-line="4"]').first().dblclick();
await page.waitForSelector('.md-edit .ProseMirror', { timeout: 15000 });
const sheetShown = await page.locator('.md-edit .ProseMirror').innerText();
check(/Design basis/.test(sheetShown) && /applied at/.test(sheetShown) && !/3 m/.test(sheetShown), `in a sheet, the paragraph and its heading open as rich text, without the input lines (${JSON.stringify(sheetShown.slice(0, 50))})`);
await page.locator('.md-edit button.primary').click();
await page.waitForTimeout(300);
check(await all() === sheetSrc, 'a sheet: applying without a change leaves the source exactly as it was');
fr = await findFrame('Design basis');
await fr.locator('[data-source-line="4"]').first().dblclick();
await page.waitForSelector('.md-edit .ProseMirror');
await page.locator('.md-edit .ProseMirror p').first().click();
await page.keyboard.press('End');
await page.keyboard.type(' More text.');
await page.locator('.md-edit button.primary').click();
await page.waitForTimeout(300);
check(await line(4) === 'The **load** is _applied_ at f~c~ midspan. More text.' && await line(2) === '# Design basis' && await line(6) === 'a: 3 m | first side' && await line(13) === 'c: sqrt(a^2 + b^2) | hypotenuse', 'a sheet: only the edited paragraph changes; the inputs and the calculation are untouched');
fr = await findFrame('More text');
check(!!fr, 'a sheet: the preview shows the edit');
// The table is a block of its own: add a row.
await page.waitForTimeout(1200); // the preview swaps its two frames after a render
fr = await findFrame('Member');
await dbl(fr, 9);
await page.waitForSelector('.md-edit .ProseMirror table');
check(await page.locator('.md-edit .ProseMirror table tr').count() === 2 && !/Design basis/.test(await page.locator('.md-edit .ProseMirror').innerText()), 'a sheet: a table opens by itself, as a table');
await page.locator('.md-edit .ProseMirror td').first().click();
await page.locator('.md-edit .md-block-bar button', { hasText: '+ Row' }).click();
await page.locator('.md-edit .ProseMirror tr').nth(2).locator('td').first().click();
await page.keyboard.type('BC');
await page.keyboard.press('Tab');
await page.keyboard.type('7 kN');
await page.locator('.md-edit button.primary').click();
await page.waitForTimeout(300);
const sheetText = (await all()).split('\n');
check(sheetText.some(l => /^\| BC +\| 7 kN +\|$/.test(l)) && sheetText.length === sheetSrc.split('\n').length + 1 && sheetText[12] === '' && sheetText[13].startsWith('c:'), `a sheet: the table gets its new row (${JSON.stringify(sheetText.slice(8, 13))})`);
// An input line still edits as one line.
fr = await findFrame('first side');
await dbl(fr, 6);
await page.waitForSelector('.line-edit-input', { timeout: 5000 });
check(await page.locator('.md-edit').count() === 0 && (await page.locator('.line-edit-input').inputValue()) === 'a: 3 m | first side', 'a sheet: an input line opens the one-line editor');
await page.keyboard.press('Escape');

check(errors.length === 0, `no page errors (${errors.join('; ')})`);
await browser.close();
server.close();
console.log(failures.length ? `${failures.length} FAILED` : 'all md-edit checks passed');
process.exit(failures.length ? 1 : 0);
