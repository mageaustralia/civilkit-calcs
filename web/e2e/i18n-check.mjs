// The interface language, end to end in a browser: the Language setting, the translated Settings dialog, ribbon, About, Shortcuts,
// block toasts and engine messages, the first-visit offer (asked, never switched by itself), ?lang=, and the install precache.
//   node e2e/i18n-check.mjs --dist DIR [--shots DIR]      serves the build itself
//   node e2e/i18n-check.mjs --url http://127.0.0.1:4180/ [--dist DIR] [--shots DIR]
import { chromium } from 'playwright';
import { readFileSync, existsSync, readdirSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { serve } from './i18n-english-check.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const shots = arg('--shots', '');
const dist = arg('--dist') ? resolve(arg('--dist')) : null;
let url = arg('--url');
let srv = null;
if (!url) { srv = await serve(dist ?? resolve(here, '..', 'calcpad-web', 'dist')); url = srv.url; }
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };

const browser = await chromium.launch();
const errors = [];
async function open(opts = {}, query = '') {
    const ctx = await browser.newContext({ viewport: { width: 1400, height: 950 }, ...opts });
    await ctx.addInitScript(() => Object.defineProperty(Navigator.prototype, 'platform', { get: () => 'Win32' }));
    const page = await ctx.newPage();
    page.on('pageerror', (e) => { if (!/^Canceled/.test(String(e))) errors.push(String(e)); });
    await page.goto(url + query);
    await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 90000 });
    await page.waitForSelector('.rb-tab');
    return { ctx, page };
}
const stored = (page) => page.evaluate(() => localStorage.getItem('calcpad-language'));
const htmlLang = (page) => page.evaluate(() => document.documentElement.lang);
const tabText = (page, id) => page.locator(`[data-ribbon-tab="${id}"]`).innerText();
async function tabOfButton(page, btn) {
    const ids = await page.locator('[data-ribbon-tab]').evaluateAll((els) => els.map((e) => e.getAttribute('data-ribbon-tab')));
    for (const id of ids) { await page.locator(`[data-ribbon-tab="${id}"]`).click(); if (await page.locator(`[data-ribbon-btn="${btn}"]`).first().isVisible()) return id; }
    return null;
}
async function openSettings(page) {
    if (!await page.locator('.tab-content.as-dialog .settings-tab').count()) {
        await tabOfButton(page, 'k-settings');
        await page.locator('[data-ribbon-btn="k-settings"]').first().click();
        await page.waitForSelector('.tab-content.as-dialog .settings-tab');
    }
    // Calculation settings opens the Calculation tab; Language is on General.
    await page.locator('.tab-content.as-dialog [role=tab][data-settings-tab=general]').click();
    return page.locator('.tab-content.as-dialog');
}
const closeDialog = async (page) => { await page.keyboard.press('Escape'); await page.waitForTimeout(200); };
const shot = async (page, name) => { if (shots) await page.screenshot({ path: join(shots, `${name}.png`) }); };

// ---- the build: catalogues are published, and are not in the install precache
if (dist) {
    const files = existsSync(join(dist, 'i18n')) ? readdirSync(join(dist, 'i18n')).sort() : [];
    check(files.join(',') === 'de.json,es-ES.json,fr.json,pt-BR.json,zh-Hans.json', `the build publishes the five catalogues (${files.join(', ')})`);
    const sw = readFileSync(join(dist, 'sw.js'), 'utf8');
    check(!/i18n\//.test(sw.split('self.addEventListener')[0]), 'the language catalogues are not in the install precache list');
    for (const f of files) {
        const j = JSON.parse(readFileSync(join(dist, 'i18n', f), 'utf8'));
        check(/machine draft/.test(j._meta?.status ?? '') && !j._meta?.reviewed_by, `${f} says it is a machine draft with no reviewer`);
    }
}

// ---- English by default, and the picker
{
    const { ctx, page } = await open({ locale: 'en-US' });
    check(await page.locator('[data-language-prompt]').count() === 0, 'an English browser gets no language prompt');
    check(await htmlLang(page) === 'en', 'the page language is en');
    const dlg = await openSettings(page);
    const options = await dlg.locator('#cprs-language-select option').allInnerTexts();
    check(options.join('|') === 'English|Español (España)|Português (Brasil)|Français|Deutsch|简体中文', `the picker lists each language in its own name (${options.join(', ')})`);
    check(await dlg.locator('#cprs-language-select').inputValue() === 'en', 'the picker shows English');
    check(!(await dlg.innerText()).includes('[!!'), 'no pseudo-language text in English');
    check((await dlg.locator('[data-settings-section=language]').count()) === 1 && (await dlg.evaluate((d) => d.querySelector('.settings-container [role=tabpanel]').firstElementChild.getAttribute('data-settings-section'))) === 'language', 'Language is the first section of the Settings dialog (on its first tab)');
    await shot(page, 'en-settings');

    // ---- German
    await dlg.locator('#cprs-language-select').selectOption('de');
    await page.waitForFunction(() => document.documentElement.lang === 'de');
    await page.waitForTimeout(300);
    const text = await dlg.textContent(); // every tab, the hidden ones too
    check(text.includes('Mathematik-Einstellungen') && text.includes('Dezimalstellen') && text.includes('Papierformat'), 'the Settings dialog is in German at once, without a reload');
    check(await dlg.locator('.search-input').getAttribute('placeholder') === 'Einstellungen durchsuchen...', 'the search placeholder is in German');
    check((await dlg.locator('#cprs-language-select').inputValue()) === 'de' && text.includes('Sprache der Oberfläche:'), 'the picker itself is in German and shows Deutsch');
    check(await stored(page) === 'de' && await htmlLang(page) === 'de', 'the choice is stored and the page language follows');
    const tabs = await page.locator('[data-ribbon-tab]').allInnerTexts();
    check(tabs[0] === 'Mathematik' && tabs.includes('Berechnung') && tabs.includes('Textformatierung'), `the ribbon tabs are in German (${tabs.join(', ')})`);
    // a search by the German section title finds it; the English keywords still work
    await dlg.locator('.search-input').fill('Diagnose');
    await page.waitForTimeout(150);
    check(await dlg.locator('section:visible', { hasText: 'Diagnose' }).count() === 1, 'searching the German section title finds the section');
    await dlg.locator('.search-input').fill('');
    await shot(page, 'de-settings');
    await closeDialog(page);

    // ribbon buttons: label, title and group name
    await page.locator('[data-ribbon-tab="calc"]').click();
    const calcBtn = page.locator('[data-ribbon-btn="k-calc"]').first();
    check((await calcBtn.innerText()).trim() === 'Berechnen' && (await calcBtn.getAttribute('title')) === 'Das Arbeitsblatt jetzt berechnen (Strg+Alt+X im Editor)', 'a ribbon button has its German label and tooltip');
    check(await page.locator('[data-ribbon-panel="calc"] .rb-gname').first().innerText() === 'Ausführen', 'a ribbon group name is in German');
    await shot(page, 'de-ribbon');

    // About and Shortcuts
    await tabOfButton(page, 'x-about');
    await page.locator('[data-ribbon-btn="x-about"]').first().click();
    const about = page.locator('#cprs-dialog');
    await about.waitFor();
    const at = await about.innerText();
    check(at.includes('Basiert auf CalcpadCE') && at.includes('Schließen') && at.includes('[') === false, 'About is in German, links rendered');
    const links = await about.locator('a').allInnerTexts();
    check(links.length === 6 && links.includes('das Änderungsprotokoll') && links.includes('fertige Rechner'), `About keeps its six links and translates the link text (${links.join(' | ')})`);
    await shot(page, 'de-about');
    await page.keyboard.press('Escape');
    await page.locator('[data-ribbon-btn="x-keys"]').first().click();
    await page.locator('#cprs-dialog').waitFor();
    const kt = await page.locator('#cprs-dialog').innerText();
    check(kt.includes('Tastenkürzel') && kt.includes('Strg') === false && kt.includes('Ctrl+S speichern'), 'Shortcuts is in German with the key names kept');
    await page.keyboard.press('Escape');

    // engine messages
    await page.evaluate(() => window.calcpadActiveEditor.setValue('a = nosuchvariable + 1\nb = 2 m + 3 s'));
    const previewText = async () => {
        for (let i = 0; i < 80; i++) {
            const h = await page.locator('iframe.preview-frame:not(.preview-frame-back)').first().elementHandle().catch(() => null);
            const f = h ? await h.contentFrame() : null;
            const t = f ? await f.locator('body').innerText().catch(() => '') : '';
            if (/Fehler in|Error in/.test(t)) return t;
            await page.waitForTimeout(400);
        }
        return '';
    };
    const pt = await previewText();
    check(pt.includes('Fehler in "a = nosuchvariable + 1"') && pt.includes('Nicht definierte Variable oder Einheiten: "nosuchvariable".'), `an engine message and its frame are in German in the preview (${pt.replace(/\s+/g, ' ').slice(0, 120)})`);
    check(/Inconsistent units|Uneinheitliche Einheiten/.test(pt), 'a message with no German text yet stays English or German, never blank');

    // back to English without a reload
    await openSettings(page);
    await page.locator('#cprs-language-select').selectOption('en');
    await page.waitForFunction(() => document.documentElement.lang === 'en');
    check((await page.locator('.tab-content.as-dialog').textContent()).includes('Math Settings') && (await tabText(page, 'calc')) === 'Calculation', 'switching back to English restores the English text at once');
    check(await stored(page) === 'en', 'English is stored as the choice');
    await closeDialog(page);
    await ctx.close();
}

// ---- the choice is kept across a reload; ?lang= is for one page view
{
    const { ctx, page } = await open({ locale: 'en-US' });
    await page.evaluate(() => localStorage.setItem('calcpad-language', 'de'));
    await page.reload();
    await page.waitForSelector('.rb-tab');
    check(await tabText(page, 'calc') === 'Berechnung', 'a stored language applies after a reload');
    await page.goto(url + '?lang=fr');
    await page.waitForSelector('.rb-tab');
    check(await tabText(page, 'calc') === 'Calcul' && await htmlLang(page) === 'fr', '?lang=fr overrides the stored language for that view');
    check(await stored(page) === 'de', '?lang= stores nothing');
    await ctx.close();
}

// ---- the first visit: asked, not switched
{
    const { ctx, page } = await open({ locale: 'de-DE' });
    const prompt = page.locator('[data-language-prompt]');
    check(await prompt.count() === 1, 'a German browser with no stored choice is offered German');
    check(await tabText(page, 'calc') === 'Calculation', 'the app stays in English until the person says yes');
    await page.waitForFunction(() => document.querySelector('[data-language-prompt] .lp-sub') != null, null, { timeout: 15000 });
    const pt = await prompt.innerText();
    check(pt.includes('CivilKit Calcs ist auf Deutsch verfügbar') && pt.includes('CivilKit Calcs is available in Deutsch') && pt.includes('Zu Deutsch wechseln') && pt.includes('Englisch beibehalten'), `the offer is written in German and in English (${pt.replace(/\s+/g, ' ')})`);
    await shot(page, 'de-prompt');
    await prompt.locator('.lp-keep').click();
    check(await page.locator('[data-language-prompt]').count() === 0 && await stored(page) === 'en', 'Keep English closes the offer and remembers the answer');
    check(await tabText(page, 'calc') === 'Calculation', 'and the app stays English');
    await page.reload();
    await page.waitForSelector('.rb-tab');
    check(await page.locator('[data-language-prompt]').count() === 0, 'the offer does not come back');
    await ctx.close();
}
{
    const { ctx, page } = await open({ locale: 'de-DE' });
    await page.locator('[data-language-prompt] .lp-go').click();
    await page.waitForFunction(() => document.documentElement.lang === 'de');
    check(await tabText(page, 'calc') === 'Berechnung' && await stored(page) === 'de' && await page.locator('[data-language-prompt]').count() === 0, 'Switching changes the language, stores it and closes the offer');
    await page.reload();
    await page.waitForSelector('.rb-tab');
    check(await tabText(page, 'calc') === 'Berechnung' && await page.locator('[data-language-prompt]').count() === 0, 'and it holds after a reload');
    await ctx.close();
}
{
    const { ctx, page } = await open({ locale: 'de-DE' });
    await page.locator('[data-language-prompt] .lp-x').click();
    check(await stored(page) === 'en' && await page.locator('[data-language-prompt]').count() === 0, 'Closing the offer counts as Keep English');
    await ctx.close();
}
for (const [locale, want] of [['ja-JP', false], ['zh-TW', false], ['en-GB', false], ['fr-CA', true], ['zh-CN', true], ['pt-PT', true], ['es-MX', true]]) {
    const { ctx, page } = await open({ locale });
    check((await page.locator('[data-language-prompt]').count() === 1) === want, `browser language ${locale}: ${want ? 'the offer shows' : 'no offer'}`);
    if (want) check(await tabText(page, 'calc') === 'Calculation', `${locale}: and the app has not switched by itself`);
    await ctx.close();
}
{
    const { ctx, page } = await open({ locale: 'de-DE' }, '?lang=es-ES');
    check(await page.locator('[data-language-prompt]').count() === 0 && await tabText(page, 'calc') === 'Cálculo', '?lang= on the address means no offer');
    await ctx.close();
}

// ---- block toasts in the active language
{
    const { ctx, page } = await open({ locale: 'en-US' }, '?lang=de');
    const sheet = ['<!--sheet-->', '# Beam', '', 'a: 1 m | first', '', 'b: 2 m | second', '', 'c: 3 m | third'].join('\n');
    await page.evaluate((t) => window.calcpadActiveEditor.setValue(t), sheet);
    const front = async () => {
        for (let i = 0; i < 60; i++) {
            const h = await page.locator('iframe.preview-frame:not(.preview-frame-back)').first().elementHandle().catch(() => null);
            const f = h ? await h.contentFrame() : null;
            if (f && await f.locator('.ck-grip').count().catch(() => 0) >= 4 && /Beam/.test(await f.locator('body').innerText().catch(() => ''))) return f;
            await page.waitForTimeout(500);
        }
        return null;
    };
    let frame = await front();
    check(!!frame, 'the preview shows the sheet');
    if (frame) {
        await page.waitForTimeout(1500);
        frame = await front();
        const target = frame.locator('[data-source-line]').nth(2);
        await target.scrollIntoViewIfNeeded();
        const bb = await target.boundingBox();
        await page.mouse.move(bb.x + bb.width / 2 - 30, bb.y + bb.height / 2 + 1);
        await page.mouse.move(bb.x + bb.width / 2, bb.y + bb.height / 2, { steps: 4 });
        await page.waitForTimeout(250);
        await frame.locator('.ck-grip.on').click();
        await page.waitForSelector('.block-menu');
        const items = await page.locator('.block-menu button').allInnerTexts();
        check(items.map((s) => s.trim()).join('|').includes('Nach oben') && items.map((s) => s.trim()).join('|').includes('Löschen'), `the block menu is in German (${items.map((s) => s.trim()).join(', ')})`);
        check(await page.locator('.block-menu').getAttribute('aria-label') === 'Blockaktionen', 'the block menu label is in German');
        await page.locator('.block-menu button', { hasText: 'Löschen' }).click();
        await page.waitForSelector('.block-toast');
        const toast = (await page.locator('.block-toast').innerText()).replace(/\s+/g, ' ').trim();
        check(toast === 'Block gelöscht Rückgängig', `the toast is in German (${toast})`);
    }
    await ctx.close();
}

// ---- Chinese: the text and the fonts
{
    const { ctx, page } = await open({ locale: 'en-US' }, '?lang=zh-Hans');
    check(await htmlLang(page) === 'zh-Hans' && await tabText(page, 'calc') === '计算', 'Chinese: the ribbon is in Chinese and the page language is zh-Hans');
    const fam = await page.evaluate(() => getComputedStyle(document.body).fontFamily);
    check(/PingFang SC/.test(fam) && /Microsoft YaHei/.test(fam), `Chinese: the interface font stack names system faces with Han glyphs (${fam.slice(0, 90)}...)`);
    const mono = await page.evaluate(() => getComputedStyle(document.documentElement).getPropertyValue('--vscode-editor-font-family'));
    check(/PingFang SC/.test(mono), 'Chinese: the editor font stack ends with faces that have Han glyphs');
    // Han text really paints: a line of Chinese measures wider than the same count of Latin letters would, and is not the missing-glyph box width
    const ok = await page.evaluate(() => {
        const s = document.createElement('span');
        s.style.cssText = 'position:absolute;visibility:hidden;font:16px var(--vscode-font-family)';
        s.textContent = '计算设置语言';
        document.body.append(s);
        const w = s.getBoundingClientRect().width;
        s.textContent = '￿￿￿￿￿￿';
        const tofu = s.getBoundingClientRect().width;
        s.remove();
        return [w, tofu];
    });
    check(ok[0] >= 6 * 12, `Chinese: six Han characters measure ${Math.round(ok[0])}px (a real face, not the empty box)`);
    // Chinese text in the editor
    await page.evaluate(() => window.calcpadActiveEditor.setValue("'这是一个中文注释，检查光标和字形\na = 2 m + 3 m\n\"标题 Heading\nb = a * 2"));
    await page.waitForTimeout(1500);
    // Monaco measures a Chinese character from a fallback face: the caret at the end of the line must sit at the end of the painted text.
    const drift = await page.evaluate(() => {
        const ed = window.calcpadActiveEditor;
        const model = ed.getModel();
        const pos = ed.getScrolledVisiblePosition({ lineNumber: 1, column: model.getLineMaxColumn(1) });
        const dom = ed.getDomNode();
        const lines = [...dom.querySelectorAll('.view-lines .view-line')].sort((a, b) => a.getBoundingClientRect().top - b.getBoundingClientRect().top);
        const text = lines[0].querySelector('span').getBoundingClientRect();
        return { caret: dom.getBoundingClientRect().left + pos.left, textRight: text.right };
    });
    check(Math.abs(drift.caret - drift.textRight) <= 3, `Chinese: the caret at the end of a line of Chinese is ${Math.abs(drift.caret - drift.textRight).toFixed(1)}px from the end of the text`);
    await shot(page, 'zh-editor');
    const dlg = await openSettings(page);
    check((await dlg.textContent()).includes('数学设置') && (await dlg.textContent()).includes('界面语言：'), 'Chinese: the Settings dialog is in Chinese');
    await shot(page, 'zh-settings');
    await closeDialog(page);
    await page.locator('[data-ribbon-tab="functions"]').click();
    await shot(page, 'zh-ribbon');
    await ctx.close();
}

// ---- the pseudo-language reaches every converted piece, and breaks nothing
{
    const { ctx, page } = await open({ locale: 'en-US' }, '?lang=xx');
    const dlg = await openSettings(page);
    const t = await dlg.innerText();
    check(t.includes('[!!') && (await tabText(page, 'calc')).startsWith('[!!'), 'the pseudo-language marks the Settings dialog and the ribbon');
    const bad = await page.evaluate(() => [...document.querySelectorAll('.rb button, .tab-content.as-dialog label, .tab-content.as-dialog h3')].filter((e) => e.scrollWidth > e.clientWidth + 2 && getComputedStyle(e).overflow === 'hidden').length);
    check(bad === 0, `the longer pseudo-text clips no ribbon button or Settings label (${bad} clipped)`);
    await shot(page, 'xx-ribbon');
    await ctx.close();
}

await browser.close();
srv?.close();
check(errors.length === 0, `no page errors${errors[0] ? ': ' + errors[0] : ''}`);
console.log(fails.length ? `${fails.length} FAIL` : 'all ok');
process.exit(fails.length ? 1 : 0);
