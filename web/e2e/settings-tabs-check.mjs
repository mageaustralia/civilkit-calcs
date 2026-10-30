// The tabs of the Settings dialog, on a desktop window (1300 px) and on a phone (iPhone 14):
//   the tabs and what each holds; arrow keys, Home and End; the search looks through every tab and clearing it brings
//   the tabs back; the last tab is remembered for the session; the ribbon's Calculation settings and Header and footer
//   open their tab; nothing is wider than a phone; Number format and Language still work.
//   node e2e/settings-tabs-check.mjs [--url http://127.0.0.1:5171/] [--shots DIR]
//   (after npm run build, with a static server on calcpad-web/dist)
import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import { chromium, devices } from 'playwright';

const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const url = arg('--url', 'http://127.0.0.1:5171/');
const shots = arg('--shots', '');
if (shots) mkdirSync(shots, { recursive: true });
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };

// Each tab, its name, and controls that must be shown on it (and only on it).
const TABS = [
    ['general', 'General', ['#cprs-language-select', '#cprs-numfmt-choice', '#colorTheme', '#previewTheme']],
    ['calculation', 'Calculation', ['#decimals', '#degrees', '#plotWidth', '#nonMetricUnits']],
    ['editor', 'Editor', ['#commentFormat', '#linterMinSeverity']],
    ['export', 'Export', ['#pdf-format', '#pdf-orientation']],
    ['advanced', 'Advanced', ['#serverUrl', '#logLevel', '.cprs-libraries']],
];
const DLG = '.tab-content.as-dialog';

const browser = await chromium.launch();
for (const [name, opts, phone] of [['desktop', { viewport: { width: 1300, height: 800 } }, false], ['phone', { ...devices['iPhone 14'] }, true]]) {
    for (const scheme of ['light', 'dark']) {
        const ctx = await browser.newContext({ ...opts, colorScheme: scheme, locale: 'en-US' });
        const page = await ctx.newPage();
        const errors = [];
        page.on('pageerror', (e) => { if (!/^Canceled/.test(String(e))) errors.push(String(e)); });
        await page.goto(url);
        await page.waitForFunction(() => window.calcpadActiveEditor != null, null, { timeout: 90000 });
        const press = (loc) => (phone ? loc.tap() : loc.click());
        const tabSel = (id) => `${DLG} [role=tab][data-settings-tab=${id}]`;
        const openGear = async () => {
            await press(page.locator('[data-ribbon-btn=q-settings]'));
            await page.locator(`${DLG} [role=tablist]`).waitFor({ timeout: 15000 });
        };
        const close = async () => {
            await page.keyboard.press('Escape');
            await page.locator(DLG).waitFor({ state: 'detached', timeout: 5000 }).catch(() => {});
        };
        const selected = () => page.locator(`${DLG} [role=tab][aria-selected=true]`).getAttribute('data-settings-tab');
        const shown = (sel) => page.locator(`${DLG} ${sel}`).first().isVisible();
        const tag = `${name} ${scheme}`;

        await openGear();
        if (scheme === 'dark') {
            // Dark: the pictures of each tab (the behaviour is the same as in light, checked below).
            for (const [id] of TABS) {
                await press(page.locator(tabSel(id)));
                await page.waitForTimeout(150);
                if (shots) await page.screenshot({ path: join(shots, `${name}-dark-${id}.png`) });
            }
            const bg = await page.locator(DLG).evaluate((e) => getComputedStyle(e).backgroundColor);
            check(/rgb\((\d+), (\d+), (\d+)\)/.test(bg) && bg.match(/\d+/g).slice(0, 3).every((v) => Number(v) < 90), `${tag}: the dialog follows the dark theme (${bg})`);
            check(errors.length === 0, `${tag}: no page errors${errors[0] ? ': ' + errors[0] : ''}`);
            await ctx.close();
            continue;
        }

        // ---- the tabs and their panels
        const tabs = await page.locator(`${DLG} [role=tablist] [role=tab]`).evaluateAll((es) => es.map((e) => ({
            id: e.dataset.settingsTab, text: e.textContent.trim(), controls: e.getAttribute('aria-controls'), tabindex: e.tabIndex, h: e.getBoundingClientRect().height,
        })));
        check(tabs.map((t) => t.text).join(',') === TABS.map((t) => t[1]).join(','), `${tag}: the dialog has the tabs ${tabs.map((t) => t.text).join(', ')}`);
        check(await selected() === 'general', `${tag}: it opens on General`);
        check(tabs.filter((t) => t.tabindex === 0).length === 1, `${tag}: one tab is in the Tab order (roving tabindex)`);
        check(tabs.every((t) => t.h >= 40), `${tag}: the tabs are at least 40 px high (${tabs.map((t) => Math.round(t.h)).join(', ')})`);
        const wired = await page.evaluate((dlg) => [...document.querySelectorAll(`${dlg} [role=tab]`)].every((t) => {
            const p = document.getElementById(t.getAttribute('aria-controls'));
            return p && p.getAttribute('role') === 'tabpanel' && p.getAttribute('aria-labelledby') === t.id;
        }), DLG);
        check(wired, `${tag}: each tab controls a tabpanel labelled by it`);
        check(!!(await page.locator(`${DLG} [role=tablist]`).getAttribute('aria-label')), `${tag}: the tab list has a name`);
        check(await page.locator(`${DLG} .settings-toolbar button`).count() === 0, `${tag}: no Collapse All button is left`);

        for (const [id, label, probes] of TABS) {
            await press(page.locator(tabSel(id)));
            await page.waitForTimeout(100);
            const visiblePanels = await page.locator(`${DLG} [role=tabpanel]:visible`).evaluateAll((es) => es.map((e) => e.dataset.settingsPanel));
            check(visiblePanels.join(',') === id, `${tag}: ${label} shows its panel only (${visiblePanels.join(',')})`);
            const here = [];
            for (const p of probes) here.push(await shown(p));
            const others = [];
            for (const [oid, , ops] of TABS) if (oid !== id) for (const p of ops) if (await shown(p)) others.push(p);
            check(here.every(Boolean) && others.length === 0, `${tag}: ${label} shows ${probes.join(' ')}${others.length ? ' but also ' + others.join(' ') : ' and nothing of the other tabs'}`);
            if (shots) await page.screenshot({ path: join(shots, `${name}-light-${id}.png`) });
            // nothing is wider than the dialog, and the dialog is not wider than the window
            const ov = await page.evaluate((dlg) => {
                const d = document.querySelector(dlg);
                return { page: document.documentElement.scrollWidth - window.innerWidth, dlg: d.scrollWidth - d.clientWidth, right: d.getBoundingClientRect().right - window.innerWidth };
            }, DLG);
            check(ov.page <= 0 && ov.dlg <= 1 && ov.right <= 1, `${tag}: ${label} has no horizontal overflow (page ${ov.page}, dialog ${ov.dlg})`);
        }
        const tabsInView = await page.locator(`${DLG} [role=tab]`).evaluateAll((es) => es.every((e) => { const r = e.getBoundingClientRect(); return r.left >= 0 && r.right <= window.innerWidth; }));
        check(tabsInView, `${tag}: every tab is whole on screen`);

        // ---- keyboard
        await press(page.locator(tabSel('general')));
        await page.locator(tabSel('general')).focus();
        const focused = () => page.evaluate(() => document.activeElement?.dataset?.settingsTab ?? null);
        await page.keyboard.press('ArrowRight');
        check(await selected() === 'calculation' && await focused() === 'calculation', `${tag}: ArrowRight moves to Calculation and focuses it`);
        check(await shown('#decimals') && !(await shown('#cprs-numfmt-choice')), `${tag}: the Calculation panel shows as its tab takes the focus`);
        await page.keyboard.press('End');
        check(await selected() === 'advanced' && await focused() === 'advanced', `${tag}: End moves to the last tab`);
        await page.keyboard.press('ArrowRight');
        check(await selected() === 'general', `${tag}: ArrowRight on the last tab wraps to the first`);
        await page.keyboard.press('ArrowLeft');
        check(await selected() === 'advanced', `${tag}: ArrowLeft on the first tab wraps to the last`);
        await page.keyboard.press('Home');
        check(await selected() === 'general' && await focused() === 'general', `${tag}: Home moves to the first tab`);
        const ring = await page.locator(tabSel('general')).evaluate((e) => getComputedStyle(e).outlineStyle);
        check(ring !== 'none', `${tag}: the focused tab shows a focus ring (${ring})`);

        // ---- search through every tab
        const search = page.locator(`${DLG} .search-input`);
        await search.fill('Decimals');
        await page.waitForTimeout(150);
        check(await shown('#decimals'), `${tag}: searching "Decimals" on General finds the Calculation setting`);
        check(!(await page.locator(`${DLG} [role=tablist]`).isVisible()), `${tag}: the tabs step aside while searching`);
        check(!(await shown('#cprs-numfmt-choice')) && !(await shown('#pdf-format')), `${tag}: settings that do not match are hidden`);
        const heads = await page.locator(`${DLG} .panel-heading:visible`).allInnerTexts();
        check(heads.map((h) => h.trim().toLowerCase()).join(',') === 'calculation', `${tag}: the match is under its tab's name (${heads.join(', ')})`);
        await search.fill('theme');
        await page.waitForTimeout(150);
        if (shots) await page.screenshot({ path: join(shots, `${name}-light-search.png`) });
        await search.fill('Decimals');
        await page.waitForTimeout(150);
        await search.fill('');
        await page.waitForTimeout(150);
        check(await page.locator(`${DLG} [role=tablist]`).isVisible() && await selected() === 'general' && await shown('#cprs-numfmt-choice') && !(await shown('#decimals')), `${tag}: clearing the search restores the tabs on General`);
        await press(page.locator(tabSel('calculation')));
        await search.fill('Number format');
        await page.waitForTimeout(150);
        check(await shown('#cprs-numfmt-choice') && !(await shown('#decimals')), `${tag}: searching "Number format" on Calculation finds the General setting`);
        await search.fill('margin');
        await page.waitForTimeout(150);
        check(await shown('#pdf-marginTop') && !(await shown('#pdf-format')), `${tag}: a search shows the matching rows only ("margin")`);
        await search.fill('zzzzqq');
        await page.waitForTimeout(150);
        check(await page.locator(`${DLG} .no-results`).isVisible() && await page.locator(`${DLG} .settings-panel:visible`).count() === 0, `${tag}: a search with no match says so`);
        await search.fill('');
        await page.waitForTimeout(150);
        check(await selected() === 'calculation' && await shown('#decimals'), `${tag}: clearing the search goes back to the tab that was open`);

        // ---- the last tab is remembered; the ribbon asks for a tab
        await press(page.locator(tabSel('editor')));
        await close();
        await openGear();
        check(await selected() === 'editor' && await shown('#commentFormat'), `${tag}: the last tab (Editor) is open again after closing and reopening`);
        await close();
        const ribbonBtn = async (btn) => {
            const b = page.locator(`[data-ribbon-btn=${btn}]`).first();
            if (!await b.isVisible()) {
                for (const t of await page.locator('[data-ribbon-tab]').evaluateAll((es) => es.map((e) => e.getAttribute('data-ribbon-tab')))) {
                    await press(page.locator(`[data-ribbon-tab=${t}]`));
                    if (await b.isVisible()) break;
                }
            }
            await press(b);
            await page.locator(`${DLG} [role=tablist]`).waitFor({ timeout: 15000 });
        };
        await ribbonBtn('k-settings');
        check(await selected() === 'calculation', `${tag}: Calculation settings in the ribbon opens the Calculation tab`);
        await close();
        await ribbonBtn('d-pdfsettings');
        check(await selected() === 'export' && await shown('#pdf-showPageNumbers'), `${tag}: Header and footer in the ribbon opens the Export tab`);
        await close();
        await openGear();
        check(await selected() === 'export', `${tag}: the gear opens the tab last shown (Export)`);
        await press(page.locator(tabSel('general')));

        // ---- Number format and Language still work (the steps of locale-check and i18n-check)
        await page.selectOption(`${DLG} #cprs-numfmt-choice`, 'comma');
        const box = page.locator(`${DLG} #cprs-numfmt-group`);
        if (!(await box.isChecked())) await press(box);
        const sample = (await page.locator(`${DLG} .cprs-sample`).innerText()).trim();
        check(/^\d{1,3}(\.\d{3})+,\d+$/.test(sample), `${tag}: Number format applies at once (example ${sample})`);
        check(await page.evaluate(() => Object.keys(localStorage).some((k) => /numfmt|number-?format/i.test(k) && /comma/.test(localStorage.getItem(k)))), `${tag}: the Number format choice is stored as before`);
        await page.selectOption(`${DLG} #cprs-numfmt-choice`, 'auto');
        if (await box.isChecked()) await press(box);
        await page.selectOption(`${DLG} #cprs-language-select`, 'de');
        await page.waitForFunction(() => document.documentElement.lang === 'de', null, { timeout: 15000 });
        await page.waitForTimeout(300);
        check(await page.locator(`${DLG} label[for=cprs-language-select]`).innerText().then((t) => /Sprache/.test(t)), `${tag}: the Language picker switches the dialog to German`);
        check(await selected() === 'general', `${tag}: the dialog stays on its tab across a language change`);
        await page.selectOption(`${DLG} #cprs-language-select`, 'en');
        await page.waitForFunction(() => document.documentElement.lang === 'en', null, { timeout: 15000 });
        check(errors.length === 0, `${tag}: no page errors${errors[0] ? ': ' + errors[0] : ''}`);
        await ctx.close();
    }
}
await browser.close();
console.log(fails.length ? `${fails.length} FAIL` : 'all ok');
process.exit(fails.length ? 1 : 0);
