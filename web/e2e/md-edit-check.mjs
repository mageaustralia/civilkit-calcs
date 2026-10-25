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

const src = ["'before", '#md on', "'## Design basis", "'The **load** is _applied_ at midspan.", '#md off', 'a = 2', 'b = a + 3', 'b'].join('\n') + '\n';
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

// 1. Double-click the paragraph: the rich-text editor opens with the block's text.
await fr.locator('[data-source-line="4"]').first().dblclick();
await page.waitForSelector('.md-edit .ProseMirror', { timeout: 15000 });
check(true, 'double-click on a Markdown paragraph opens the rich-text editor');
const shown = await page.locator('.md-edit .ProseMirror').innerText();
check(/Design basis/.test(shown) && /applied at midspan/.test(shown), `the editor shows the whole block (${JSON.stringify(shown.slice(0, 60))})`);
check(await page.locator('.md-edit .ProseMirror h2').count() === 1 && await page.locator('.md-edit .ProseMirror strong').count() === 1, 'heading and bold are shown as formatting, not as symbols');
await page.screenshot({ path: join(process.env.TMPDIR ?? '/tmp', 'md-edit-open.png') });

// 2. Apply without a change: the source is untouched.
await page.locator('.md-edit button.primary').click();
await page.waitForTimeout(300);
check(await all() === src, 'applying without a change leaves the source exactly as it was');

// 3. Edit: type at the end of the paragraph, make a word bold with the toolbar, apply.
fr = await findFrame('Design basis');
await fr.locator('[data-source-line="4"]').first().dblclick();
await page.waitForSelector('.md-edit .ProseMirror');
const para = page.locator('.md-edit .ProseMirror p').first();
await para.click();
await page.keyboard.press('End');
await page.keyboard.type(' Check the deflection.');
await page.locator('.md-edit button.primary').click();
await page.waitForTimeout(300);
check(await line(3) === "'## Design basis", 'the untouched heading keeps its source line');
check(/^'The \*\*load\*\* is _applied_ at midspan\. Check the deflection\.$/.test(await line(4)), `the edited paragraph is written as one Markdown line (${JSON.stringify(await line(4))})`);
check(await line(2) === '#md on' && await line(5) === '#md off' && await line(6) === 'a = 2', 'the block markers and the rest of the sheet are unchanged');

// 4. The preview re-renders the new text.
fr = await findFrame('Check the deflection');
check(!!fr, 'the preview shows the edited text');

// 5. Undo restores the block in one step.
await page.evaluate(() => { window.calcpadActiveEditor.focus(); window.calcpadActiveEditor.trigger('t', 'undo', null); });
check(await all() === src, 'one undo restores the block');

// 6. Esc cancels; an equation still edits as a source line.
fr = await findFrame('Design basis');
await fr.locator('[data-source-line="4"]').first().dblclick();
await page.waitForSelector('.md-edit .ProseMirror');
await page.keyboard.press('Escape');
await page.waitForTimeout(200);
check(await page.locator('.md-edit').count() === 0 && await all() === src, 'Esc cancels without a change');
await fr.locator('[data-source-line="7"]').first().dblclick();
await page.waitForSelector('.line-edit-input', { timeout: 5000 });
check(await page.locator('.md-edit').count() === 0 && (await page.locator('.line-edit-input').inputValue()) === 'b = a + 3', 'an equation line still opens the source-line editor');

// 7. Inline HTML (sub / sup) and escaped characters survive an edit of the paragraph.
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
console.log('  rich line:', rich);
check(/<sub>c<\/sub>/.test(rich) && /<sup>2<\/sup>/.test(rich) && /a\\?_b/.test(rich) && /ok$/.test(rich), 'sub, sup and an in-word underscore survive an edit (an underscore inside a word is not emphasis)');

check(errors.length === 0, `no page errors (${errors.join('; ')})`);
await browser.close();
server.close();
console.log(failures.length ? `${failures.length} FAILED` : 'all md-edit checks passed');
process.exit(failures.length ? 1 : 0);
