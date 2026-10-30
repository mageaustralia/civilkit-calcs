// Checks `#python` blocks in the built app served at --url (default http://127.0.0.1:4180/):
// lazy loading, the once-per-tab question, print text, an SVG figure, an exported variable used by later Calcpad lines,
// error line mapping, the timeout, the sandbox, and the exports (Report, Word, Excel).
// numpy.linalg / scipy.linalg on the engine's wasm module, with its error lines and its cache rules.
// The Plotly API (import plotly.graph_objects as go): the figure JSON, the chart in Chromium, its native menu and sliders, the
// named errors, the offline note, and the Word and Excel placeholder. The chart checks load Plotly.js from its CDN and are
// skipped, with a line saying so, when there is no network.
//   PLAYWRIGHT_BROWSERS_PATH=... node e2e/python-check.mjs [--url U] [--out DIR]
import { chromium } from 'playwright';
import { mkdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const url = arg('--url', 'http://127.0.0.1:4180/');
const out = arg('--out', join(tmpdir(), 'civilkit-calcs-python'));
mkdirSync(out, { recursive: true });
const fails = [];
const check = (ok, what) => { console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`); if (!ok) fails.push(what); };
const browser = await chromium.launch();

async function open(scheme = 'light') {
    const ctx = await browser.newContext({ viewport: { width: 1400, height: 900 }, colorScheme: scheme });
    const page = await ctx.newPage();
    const requests = [];
    const errors = [];
    page.on('request', r => requests.push(r.url()));
    page.on('pageerror', e => errors.push(String(e)));
    await page.goto(url);
    await page.waitForFunction(() => window.calcpadActiveEditor != null && window.calcpadFiles != null, null, { timeout: 60000 });
    return { ctx, page, requests, errors };
}
const setText = (page, t) => page.evaluate(v => window.calcpadActiveEditor.setValue(v), t);
/** The text of the results frame once `needle` shows in it (or '' after the timeout). */
async function frameText(page, needle, ms = 40000) {
    const t0 = Date.now();
    while (Date.now() - t0 < ms) {
        for (const f of page.frames()) {
            if (f === page.mainFrame()) continue;
            const t = await f.evaluate(() => document.body?.innerText ?? '').catch(() => '');
            if (t.includes(needle)) return t;
        }
        await page.waitForTimeout(300);
    }
    return '';
}
const frameHas = async (page, sel) => {
    for (const f of page.frames()) if (f !== page.mainFrame() && await f.locator(sel).count().catch(() => 0)) return true;
    return false;
};
const yes = page => page.locator('#cprs-python-yes');

const SHEET = [
    '"Python check',                         // 1
    'a = 5',                                  // 2
    '#python',                                // 3
    'import numpy as np',                     // 4
    'import matplotlib.pyplot as plt',        // 5
    'x = np.linspace(0, 1, 11)',              // 6
    'print("sum is", round(float(x.sum()), 2), "<b>&")',// 7
    'plt.plot(x, x * x, label="x2")',         // 8
    'plt.xlabel("x")',                        // 9
    'plt.legend()',                           // 10
    'plt.show()',                             // 11
    'export(k=float(x.sum()), v=[1, 2, 3], m=[[1, 2], [3, 4]])', // 12
    '#end python',                            // 13
    'r = k*2',                                // 14
    'z = a + 1',                              // 15
].join('\n');

// ---- 1. a worksheet without #python never loads the runtime ------------------------------
{
    const { ctx, page, requests } = await open();
    await setText(page, 'a = 3\nb = a^2');
    check((await frameText(page, 'b =')).length > 0, 'a plain worksheet calculates');
    await page.waitForTimeout(1500);
    check(!requests.some(u => /\/py\//.test(u)), 'no request to /py/ for a worksheet without #python');
    check(await page.evaluate(() => window.calcpadPython.started === false), 'the Python worker was never started');
    check(await page.locator('#cprs-python-yes').count() === 0, 'no question for a plain worksheet');
    await ctx.close();
}

// ---- 2. the question, text, figure, export, later lines, exports ---------------------------
{
    const { ctx, page, requests, errors } = await open();
    await setText(page, SHEET);
    await yes(page).waitFor({ timeout: 20000 });
    check(true, 'the confirmation prompt appears before the first run');
    check(!requests.some(u => /micropython/.test(u)), 'nothing of the runtime is fetched before the answer');
    await page.screenshot({ path: join(out, 'prompt.png') });
    await yes(page).click();
    const t = await frameText(page, 'r =');
    check(/sum is 5\.5 <b>&/.test(t), 'print() text is shown, HTML-escaped' + (/sum is 5\.5 <b>&/.test(t) ? '' : ' ' + JSON.stringify(t.slice(0, 300))));
    check(await frameHas(page, 'svg.cprs-py-figure'), 'the figure is an inline SVG');
    check(/k = 5\.5/.test(t) && /r = .*= 11/.test(t.replace(/\s+/g, ' ')), 'an exported number is used by a later Calcpad line (r = k*2 = 11)');
    check(/v⃗?\s*=\s*\[1; 2; 3\]|v = \[1; 2; 3\]/.test(t) || t.includes('[1; 2; 3]'), 'an exported list is a Calcpad vector');
    check(/\[1; 2 ?\| 3; 4\]/.test(t), 'an exported 2-D list is a Calcpad matrix');
    check(/z = .*6/.test(t.replace(/\s+/g, ' ')), 'Calcpad lines after the block still calculate');
    check(requests.some(u => /micropython\.wasm/.test(u)), 'the runtime loaded only now');
    await page.screenshot({ path: join(out, 'figure-light.png') });
    // the question is not repeated in this tab, even after an edit
    await setText(page, SHEET.replace('a = 5', 'a = 7'));
    const t2 = await frameText(page, '7 + 1');
    check(/z = .*8/.test(t2.replace(/\s+/g, ' ')) && await yes(page).count() === 0, 'no second question in the same tab' + (/z = .*8/.test(t2.replace(/\s+/g, ' ')) ? '' : ' ' + JSON.stringify(t2.slice(-200))));

    // exports through the same transport
    const exportStatus = await page.evaluate(async (content) => {
        const o = {};
        for (const ep of ['convert', 'docx', 'xlsx']) {
            const r = await fetch(`/api/calcpad/${ep}`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ content, forPrint: ep === 'convert', includeLineAnchors: false }) });
            const b = await r.arrayBuffer();
            o[ep] = { status: r.status, bytes: b.byteLength, text: ep === 'convert' ? new TextDecoder().decode(b) : '' };
        }
        return o;
    }, SHEET);
    check(exportStatus.convert.status === 200 && /<svg[^>]*cprs-py-figure/.test(exportStatus.convert.text) && /sum is/.test(exportStatus.convert.text), 'Report / Print HTML carries the figure and the text');
    check(exportStatus.docx.status === 200 && exportStatus.docx.bytes > 5000, `Word export works (${exportStatus.docx.bytes} bytes)`);
    check(exportStatus.xlsx.status === 200 && exportStatus.xlsx.bytes > 3000, `Excel export works (${exportStatus.xlsx.bytes} bytes)`);
    check(errors.length === 0, `no page errors${errors.length ? ': ' + errors[0] : ''}`);
    await ctx.close();
}

// ---- 3. dark theme figure, error line mapping, sandbox, timeout ---------------------------
{
    const { ctx, page } = await open('dark');
    await setText(page, SHEET);
    await yes(page).click({ timeout: 20000 });
    await frameText(page, 'r =');
    await page.screenshot({ path: join(out, 'figure-dark.png') });

    const BAD = ['"Error mapping', 'q = 1', '#python', 'x = 1', 'y = 1 / 0', '#end python', 'w = nothing_here + 1'].join('\n');
    await setText(page, BAD);
    const t = await frameText(page, 'Python error');
    check(/Python error on line 5: ZeroDivisionError/.test(t), 'a Python exception names the worksheet line (5) and its type');
    check(/nothing_here/.test(t), 'Calcpad lines after a failed block still calculate (and report their own errors)');
    const sourceLines = await page.evaluate(async (content) => {
        const r = await fetch('/api/calcpad/convert', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ content }) });
        return JSON.parse(decodeURIComponent(r.headers.get('X-Calcpad-Errors') ?? '[]')).map(e => [e.source, e.sourceLine]);
    }, BAD);
    check(sourceLines.some(([s, l]) => s === 'Python' && l === 5), 'the error list has the Python error on line 5');
    check(sourceLines.some(([s, l]) => s !== 'Python' && l === 7), 'an engine error after the block keeps its worksheet line (7)');
    await page.screenshot({ path: join(out, 'error-line.png') });

    await setText(page, ['"Sandbox1', '#python', 'import js', '#end python'].join('\n'));
    const s1 = await frameText(page, 'Sandbox1');
    check(/ImportError/.test(s1) && /not available/.test(s1), '`import js` is refused ' + JSON.stringify(s1.slice(0, 200)));
    await setText(page, ['"Sandbox2', '#python', 'from jsffi import create_proxy', '#end python'].join('\n'));
    check(/ImportError/.test(await frameText(page, 'Sandbox2')), '`import jsffi` is refused');
    await setText(page, ['"Sandbox3', '#python', 'import _cp_host', 'print(hasattr(_cp_host, "constructor"))', 'import builtins', 'print(builtins.__import__.__class__)', '#end python'].join('\n'));
    const s3 = await frameText(page, 'Sandbox3');
    check(!/constructor.*True/.test(s3) && /False/.test(s3), 'the host object is not reachable through a JS proxy ' + JSON.stringify(s3.slice(0, 200)));

    await setText(page, ['"Timeout', 'ok = 1', '#python', 'while True:', '    pass', '#end python', 'after = ok + 1'].join('\n'));
    const t0 = Date.now();
    // the time limit asks whether to run again with a longer one: answer Not now
    await page.locator('#cprs-python-longer-no').waitFor({ timeout: 30000 });
    check(/60 s/.test(await page.locator('#cprs-python-longer-yes').innerText()), 'the timeout offers a run with a 60 s limit');
    await page.locator('#cprs-python-longer-no').click();
    const tt = await frameText(page, 'was stopped', 30000);
    check(/ran longer than 10 s and was stopped/.test(tt), `an endless loop is killed (${Math.round((Date.now() - t0) / 1000)} s)`);
    check(/after =/.test(tt), 'the worksheet after the timed-out block still calculates');
    // the worker comes back for the next run
    await setText(page, ['"After timeout', '#python', 'print("alive")', '#end python'].join('\n'));
    check((await frameText(page, 'alive', 30000)).length > 0, 'a new run works after a timeout');

    await setText(page, ['"Open', '#python', 'x = 1'].join('\n'));
    const openText = await frameText(page, "Open", 15000);
    check(/no "#end python"/.test(openText), "an unclosed block is an error line " + JSON.stringify(openText.slice(0, 200)));
    await ctx.close();
}

// ---- 4. Not now ----------------------------------------------------------------------------
{
    const { ctx, page, requests } = await open();
    await setText(page, SHEET);
    await page.locator('#cprs-python-no').click({ timeout: 20000 });
    const t = await frameText(page, 'Python was not run');
    check(t.length > 0, 'Not now: the block is not run and says so');
    check(!requests.some(u => /micropython/.test(u)), 'Not now: the runtime is never loaded');
    check(/z = /.test(t), 'Not now: the rest of the worksheet calculates');
    await ctx.close();
}

// ---- 5. the Python samples ---------------------------------------------------------------
{
    const { ctx, page } = await open();
    const manifest = await page.evaluate(async () => (await fetch('samples/manifest.json')).json());
    const py = manifest.samples.filter(s => s.group === 'Python');
    check(py.length === 5 && manifest.groups.includes('Python'), `five samples in the Python group (${py.length})`);
    check(py[0].title === 'Beam Explorer: Plotly from Python', 'the Plotly API beam explorer sample comes first');
    check(py[1].title === 'Beam Explorer: Python Results in Raw JS Plotly' && py[1].scripts === true && py[0].scripts === false, 'the raw JS explorer is the second sample and is marked as carrying scripts; the Plotly API one is not');
    for (const s of py) {
        await page.evaluate(async (id) => { const m = (await (await fetch('samples/manifest.json')).json()).samples.find(x => x.id === id); await window.calcpadFiles.samples.openSample(m); }, s.id);
        if (await yes(page).count().catch(() => 0) || await yes(page).waitFor({ timeout: 4000 }).then(() => true, () => false)) await yes(page).click();
        const t = await frameText(page, 'Python', 40000) || await frameText(page, '=', 5000);
        const bad = /Python error|Error in "/.test(t);
        check(t.length > 0 && !bad && await frameHas(page, 'svg.cprs-py-figure, .cprs-plotly-wrap'), `sample "${s.title}" runs with a figure and no errors`);
        await page.waitForTimeout(2500);
        const markers = await page.evaluate(() => window.monaco.editor.getModelMarkers({ resource: window.calcpadActiveEditor.getModel().uri }).filter(m => m.severity >= 8).map(m => `${m.startLineNumber}: ${m.message}`));
        check(markers.length === 0, `sample "${s.title}" has no error markers in the editor${markers.length ? ' ' + JSON.stringify(markers.slice(0, 3)) : ''}`);
    }
    await ctx.close();
}

// ---- 6. the ribbon inserts a block that runs ----------------------------------------------
{
    const { ctx, page } = await open();
    await setText(page, 'a = 1');
    await page.locator('[data-ribbon-tab=math]').click();
    await page.locator('[data-ribbon-btn="p-programming"]').click();
    await page.locator('.rb-pop [data-ribbon-glyph="#python plot"]').click();
    const text = await page.evaluate(() => window.calcpadActiveEditor.getValue());
    check(/#python\n[\s\S]*plt\.show\(\)\n#end python/.test(text), 'Math > Programming > #python plot inserts a block');
    await yes(page).click({ timeout: 20000 });
    await frameText(page, 'a = 1');
    await page.waitForTimeout(3000);
    check(await frameHas(page, 'svg.cprs-py-figure'), 'the inserted block runs and draws a figure');
    await ctx.close();
}

// ---- 7. python fences in a sheet -------------------------------------------------------------
{
    const { ctx, page } = await open();
    const SHEET_SRC = [
        '<!--sheet-->', '# Sheet python', '', 'a: 5', '', '```python', 'import numpy as np', 'x = np.linspace(0, 1, 11)',
        'print("sum is", round(float(x.sum()), 2))', 'export(k=float(x.sum()))', '```', '', 'r: k * 2', 'z: a + nothing_here',
    ].join('\n');
    await setText(page, SHEET_SRC);
    await yes(page).click({ timeout: 20000 });
    const t = await frameText(page, 'Sheet python');
    await frameText(page, 'sum is');
    const t2 = await frameText(page, 'r =');
    check(/sum is 5\.5/.test(t2) && /r = .*= 11/.test(t2.replace(/\s+/g, ' ')), 'a python fence in a sheet runs, and a quantity line after it uses the exported value');
    check(/nothing_here/.test(t2), 'an engine error after the fence is reported');
    const lines = await page.evaluate(async (content) => {
        const r = await fetch('/api/calcpad/convert', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ content }) });
        return JSON.parse(decodeURIComponent(r.headers.get('X-Calcpad-Errors') ?? '[]')).map(e => e.sourceLine);
    }, SHEET_SRC);
    check(lines.includes(14), `the error maps to its sheet line (14): ${JSON.stringify(lines)}`);
    const BAD = ['<!--sheet-->', '# Bad', '```python', 'x = 1', 'y = 1 / 0', '```', 'q: 1'].join('\n');
    await setText(page, BAD);
    const eb = await frameText(page, 'Bad');
    await frameText(page, 'Python error');
    const eb2 = await frameText(page, 'ZeroDivisionError');
    check(/Python error on line 5: ZeroDivisionError/.test(eb2), 'a Python error in a sheet fence names the sheet line (5)');
    // the editor leaves the fence alone: no error markers on its lines
    const markers = await page.evaluate(() => window.monaco.editor.getModelMarkers({ resource: window.calcpadActiveEditor.getModel().uri }).filter(m => m.severity >= 8 && m.startLineNumber >= 3 && m.startLineNumber <= 6).map(m => m.message));
    check(markers.length === 0, `no Calcpad error markers inside the python fence ${JSON.stringify(markers)}`);
    // the ribbon follows the sheet mode
    await setText(page, '<!--sheet-->\nx: 1\n');
    await page.locator('[data-ribbon-tab=math]').click();
    await page.locator('[data-ribbon-btn="p-programming"]').click();
    await page.locator('.rb-pop [data-ribbon-glyph="#python plot"]').click();
    const ins = await page.evaluate(() => window.calcpadActiveEditor.getValue());
    check(/```python\n[\s\S]*plt\.show\(\)\n```/.test(ins) && !/#python/.test(ins), 'in a sheet the ribbon inserts a python fence');
    await ctx.close();
}

// ---- 8. the raw JS beam explorer: Python results drive a Plotly chart written as HTML and JavaScript ----------------------------------
{
    const { ctx, page } = await open();
    // Opened the way a user opens it (the Samples browser converts C-style operators, which must not touch the Python).
    await page.evaluate(async () => { const m = (await (await fetch('samples/manifest.json')).json()).samples.find(s => s.title === 'Beam Explorer: Python Results in Raw JS Plotly'); await window.calcpadFiles.samples.openSample(m); });
    await yes(page).click({ timeout: 20000 });
    const t = await frameText(page, 'Design check', 60000);
    check(/Equilibrium: the reactions sum to the load/.test(t) && /UDL on all spans 300\.0 of 300\.0 kN/.test(t), 'the Python solve ran and its equilibrium check holds');
    check(/η = /.test(t) && /M_Ed|MEd/.test(t) && /within the capacity/.test(t), 'the Calcpad design check uses the exported envelope');
    const frame = async () => { for (const f of page.frames()) if (f !== page.mainFrame() && await f.locator('#cx-read').count().catch(() => 0)) return f; return null; };
    let f = null;
    for (let i = 0; i < 100 && !f; i++) { f = await frame(); if (!f) await page.waitForTimeout(300); }
    check(!!f, 'the explorer is on the page');
    const plotly = f ? await f.waitForFunction(() => typeof Plotly !== 'undefined' && document.querySelectorAll('#cx-plot .main-svg').length > 0, null, { timeout: 45000 }).then(() => true, () => false) : false;
    if (!plotly) console.log('skip the chart checks: Plotly did not load (offline?)');
    else {
        const r0 = await f.evaluate(() => document.getElementById('cx-read').innerText);
        // three-moment equations for the three UDL spans (5, 6, 4 m, 20 kN/m): M_B = -63.61, M_C = -50.91 kN.m
        check(/hogging max -63\.6 kN\.m at x = 5\.00 m/.test(r0), `the chart's hogging moment over support B matches the three-moment solution (-63.6) ${JSON.stringify(r0.slice(0, 160))}`);
        const traces = await f.evaluate(() => document.getElementById('cx-plot').data.length);
        check(traces === 5, `five traces drawn (${traces})`);
        await f.evaluate(() => { const s = document.getElementById('cx-fac'); s.value = '2'; s.dispatchEvent(new Event('input')); });
        const r2 = await f.evaluate(() => document.getElementById('cx-read').innerText);
        check(/hogging max -127\.2 kN\.m/.test(r2), 'the load factor slider doubles the moments (-127.2)');
        await f.evaluate(() => { const c = document.getElementById('cx-case'); c.value = '3'; c.dispatchEvent(new Event('input')); const s = document.getElementById('cx-pos'); s.value = '2'; s.dispatchEvent(new Event('input')); });
        const r3 = await f.evaluate(() => document.getElementById('cx-read').innerText);
        check(/P = 120\.0 kN at x = 2\.5 m/.test(r3), `the point load case and its position slider respond ${JSON.stringify(r3.slice(0, 60))}`);
        await page.screenshot({ path: join(out, 'explorer.png') });
    }
    await ctx.close();
}

// ---- 9. numpy.linalg and scipy.linalg run on the Calcpad engine --------------------------------------------
{
    const { ctx, page, requests } = await open();
    const linalgLoaded = () => requests.some(u => /calcpad_linalg_wasm_bg\.wasm/.test(u));
    // 9a. the user's own section calculation needs no engine call and does not load the linear algebra module
    const SECTION = ['"Section', '#python', 'import numpy as np', 'b = np.array([300.0, 20.0])', 'h = np.array([20.0, 280.0])', 'y = np.array([290.0, 140.0])',
        'A = b * h', 'y_c = float((A * y).sum() / A.sum())', 'I_x = float((b * h**3 / 12 + A * (y - y_c) ** 2).sum())', 'print("section", round(y_c, 3), round(I_x))',
        'export(y_c=y_c, I_x=I_x)', '#end python'].join('\n');
    await setText(page, SECTION);
    await yes(page).click({ timeout: 20000 });
    const s = await frameText(page, 'section');
    check(/section 217\.586 101959080/.test(s), 'section properties from numpy arrays (b*h, (A*y).sum()/A.sum(), b*h**3/12) ' + JSON.stringify(s.slice(0, 160)));
    check(/yc = 217\.59/.test(s.replace(/\s+/g, ' ')), 'the section centroid is exported (as yc, the sheet spelling of y_c)');
    await page.waitForTimeout(800);
    check(!linalgLoaded(), 'a block that does not mention linalg does not load the linear algebra module');

    // 9b. solve, inv, det, cholesky, eigh, svd, scipy.linalg
    const LINALG = ['"Linalg', '#python', 'import numpy as np', 'import numpy.linalg as la', 'from scipy import linalg as sla',
        'A = np.array([[2.0, 1.0, -1.0], [-3.0, -1.0, 2.0], [-2.0, 1.0, 2.0]])',
        'x = np.linalg.solve(A, np.array([8.0, -11.0, -3.0]))',
        'print("solve", [round(float(v), 6) for v in x])',
        'print("det", round(float(la.det(A)), 6))',
        'print("inv", np.round(la.inv(np.array([[4.0, 7.0], [2.0, 6.0]])), 6).tolist())',
        'S = np.array([[4.0, 12.0, -16.0], [12.0, 37.0, -43.0], [-16.0, -43.0, 98.0]])',
        'print("chol", np.round(la.cholesky(S), 6).tolist())',
        'print("cho_solve", [round(float(v), 6) for v in sla.cho_solve(sla.cho_factor(S), [1.0, 2.0, 3.0])], [round(float(v), 6) for v in la.solve(S, [1.0, 2.0, 3.0])])',
        'w, v = la.eigh(np.array([[2.0, 1.0], [1.0, 2.0]]))',
        'print("eigh", [round(float(t), 6) for t in w], round(abs(float(v[0][0])), 6))',
        'u, s, vt = la.svd(np.array([[3.0, 0.0], [0.0, -2.0]]))',
        'print("svd", [round(float(t), 6) for t in s])',
        'print("at", np.round(A @ x, 6).tolist())',
        'import numpy._array as _na',
        'B = np.array([[(i * 7 + j * 3) % 11 / 11.0 for j in range(60)] for i in range(60)])',
        'P1 = B @ B',
        '_na._ENGINE_DOT_MIN = 10 ** 12',
        'P2 = B @ B',
        'print("matmul equal", P1.tolist() == P2.tolist())',
        '#end python'].join('\n');
    await setText(page, LINALG);
    const l = await frameText(page, 'matmul equal', 60000);
    check(/solve \[2\.0, 3\.0, -1\.0\]/.test(l), 'np.linalg.solve gives the hand-known 3x3 answer (2, 3, -1) ' + JSON.stringify(l.slice(0, 200)));
    check(/det -1\.0\b/.test(l), 'np.linalg.det of the same matrix is -1');
    check(/inv \[\[0\.6, -0\.7\], \[-0\.2, 0\.4\]\]/.test(l), 'np.linalg.inv of [[4,7],[2,6]]');
    check(/chol \[\[2\.0, 0\.0, 0\.0\], \[6\.0, 1\.0, 0\.0\], \[-8\.0, 5\.0, 3\.0\]\]/.test(l), 'np.linalg.cholesky gives the lower factor [[2,0,0],[6,1,0],[-8,5,3]]');
    check(/cho_solve \[28\.583333, -7\.666667, 1\.333333\] \[28\.583333, -7\.666667, 1\.333333\]/.test(l), 'scipy.linalg.cho_factor + cho_solve agree with solve');
    check(/eigh \[1\.0, 3\.0\] 0\.707107/.test(l), 'np.linalg.eigh: ascending eigenvalues 1 and 3, unit eigenvectors');
    check(/svd \[3\.0, 2\.0\]/.test(l), 'np.linalg.svd: singular values 3 and 2');
    check(/at \[8\.0, -11\.0, -3\.0\]/.test(l), 'A @ x reproduces the right-hand side');
    check(/matmul equal True/.test(l), 'a 60x60 product from the engine equals the Python product value for value');
    check(linalgLoaded(), 'the linear algebra module loaded for a block that uses linalg');

    // 9c. errors name the Python line
    await setText(page, ['"Singular', 'q = 1', '#python', 'import numpy as np', 'M = np.array([[1.0, 2.0], [2.0, 4.0]])', 'x = np.linalg.solve(M, [1.0, 2.0])', '#end python'].join('\n'));
    const e1 = await frameText(page, 'Python error', 30000);
    check(/Python error on line 6: LinAlgError/.test(e1) && /singular/i.test(e1), 'a singular matrix is an error on its Python line (6) ' + JSON.stringify(e1.slice(0, 200)));
    await setText(page, ['"NotPD', '#python', 'import numpy as np', 'np.linalg.cholesky(np.array([[1.0, 2.0], [2.0, 1.0]]))', '#end python'].join('\n'));
    check(/Python error on line 4: LinAlgError/.test(await frameText(page, 'on line 4: LinAlgError', 30000)), 'a matrix that is not positive definite is a LinAlgError on its line (4)');
    await setText(page, ['"Unsupported', '#python', 'import numpy as np', 'np.linalg.eig(np.array([[1.0, 2.0], [3.0, 4.0]]))', '#end python'].join('\n'));
    const e3 = await frameText(page, 'NotImplementedError', 30000);
    check(/NotImplementedError/.test(e3) && /non-symmetric/.test(e3), 'eig of a non-symmetric matrix says it is not available and why');
    await setText(page, ['"Unknown', '#python', 'import numpy as np', 'np.linalg.lstsq([[1.0]], [1.0])', '#end python'].join('\n'));
    const e5 = await frameText(page, 'lstsq', 30000);
    check(/NotImplementedError/.test(e5) && /numpy\.linalg\.lstsq is not available/.test(e5), 'an unsupported linalg name raises NotImplementedError');

    // 9d. a stale or wrong module cannot answer: a wrong wasm behind the glue is refused with a clear error
    const ctx2 = await browser.newContext({ viewport: { width: 1400, height: 900 } });
    const page2 = await ctx2.newPage();
    await page2.route(/calcpad_linalg_wasm_bg\.wasm/, async route => {
        const r = await page2.request.get(new URL('py/micropython/micropython.wasm', url).href);
        await route.fulfill({ status: 200, contentType: 'application/wasm', body: await r.body() });
    });
    await page2.goto(url);
    await page2.waitForFunction(() => window.calcpadActiveEditor != null && window.calcpadFiles != null, null, { timeout: 60000 });
    await setText(page2, ['"Mismatch', '#python', 'import numpy as np', 'np.linalg.det(np.array([[1.0, 2.0], [3.0, 4.0]]))', '#end python'].join('\n'));
    await yes(page2).click({ timeout: 20000 });
    const e4 = await frameText(page2, 'Python error', 30000);
    check(/did not load/.test(e4), 'a wasm that does not match the glue is refused ("did not load"), never used ' + JSON.stringify(e4.slice(0, 200)));
    await ctx2.close();
    await ctx.close();
}

// ---- 10. caching rules: the wasm the Python worker loads is revalidated -------------------------------------------
{
    const { readFileSync } = await import('node:fs');
    const headers = readFileSync(new URL('../calcpad-web/cloudflare/_headers', import.meta.url), 'utf8').split('\n');
    const rule = (path) => { const i = headers.findIndex(l => l.trim() === path); return i < 0 ? '' : headers.slice(i + 1, i + 4).join(' '); };
    check(/no-cache/.test(rule('/calcs/wasm/*.wasm')), '_headers: /calcs/wasm/*.wasm is revalidated (covers the linear algebra module)');
    check(/no-cache/.test(rule('/calcs/py/micropython/*.wasm')), '_headers: /calcs/py/micropython/*.wasm is revalidated');
    const worker = readFileSync(new URL('../calcpad-web/src/wasm/python-worker.ts', import.meta.url), 'utf8');
    check(/calcpad_linalg_wasm_bg\.wasm`, \{ cache: 'no-cache' \}/.test(worker) && /LINALG_ABI/.test(worker), 'the worker fetches the linear algebra wasm with cache: no-cache and checks its abi()');
    const rust = readFileSync(new URL('../../calcpad-linalg-wasm/src/lib.rs', import.meta.url), 'utf8');
    const abiRust = /pub const ABI: &str = "([^"]+)"/.exec(rust)?.[1];
    const abiTs = /const LINALG_ABI = '([^']+)'/.exec(worker)?.[1];
    check(!!abiRust && abiRust === abiTs, `the Rust module and the worker agree on the interface version (${abiRust})`);
}

// ---- 11. the plane truss sample against independent hand values -----------------------------------------------------
// The sample (Python group, "Plane Truss by the Stiffness Method") is opened as a user opens it; its own block is run with one extra
// print line, so the numbers checked are the ones the sample computes, at full precision.
//
// Hand derivation (kN, m; joints 1 (0,0) pin, 2 (4,0), 3 (8,0) roller, 4 (2,3), 5 (6,3); diagonals 1-4, 4-2, 2-5, 5-3 have
// length sqrt(13), sin = s = 3/sqrt(13), cos = c = 2/sqrt(13); loads: joint 4 (+20, -40), joint 5 (0, -60), joint 2 (0, -80)):
//   Reactions: sum Fx = 0 -> Rx1 = -20. Moments about joint 1 (x Fy - y Fx summed over the loads, plus 8 Ry3): 8 Ry3 + (2*(-40) - 3*20)
//   + 6*(-60) + 4*(-80) = 0 -> 8 Ry3 = 820 -> Ry3 = 102.5; sum Fy = 0 -> Ry1 = 180 - 102.5 = 77.5.
//   Method of joints (tension positive, each member force times its unit vector away from the joint):
//     joint 1: Fy: 77.5 + s N14 = 0 -> N14 = -77.5/s = -93.143408; Fx: -20 + N12 + c N14 = 0 -> N12 = 20 + 77.5 c/s = 71.666667
//     joint 3: Fy: 102.5 + s N53 = 0 -> N53 = -102.5/s = -123.189669; Fx: -N23 - c N53 = 0 -> N23 = 102.5 c/s = 68.333333
//     joint 4: Fy: -40 - s N14 - s N42 = 0 -> N42 = (-40 + 77.5)/s = 37.5/s = 45.069391; Fx: 20 - c N14 + N45 + c N42 = 0 -> N45 = -96.666667
//     joint 5: Fy: -60 - s N25 - s N53 = 0 -> N25 = (-60 + 102.5)/s = 42.5/s = 51.078643 (joint 2 and the Fx of joint 5 then close).
//   Virtual work (unit downward load at joint 2): reactions 1/2 and 1/2, forces f12 = f23 = 1/3, f45 = -2/3, f14 = f53 = -0.5/s,
//   f42 = f25 = 0.5/s; deflection = sum f N L / (E A) with E A = 200 * 1200 kN (chords) or 200 * 800 kN (diagonals) = 6.083367 mm.
//   Unit load at joint 4: reactions 3/4 and 1/4, f12 = 1/2, f23 = 1/6, f45 = -1/3, f14 = -0.75/s, f42 = -0.25/s, f25 = 0.25/s, f53 = -0.25/s
//   -> 4.090834 mm.
{
    const { ctx, page } = await open();
    const m = (await page.evaluate(async () => (await fetch('samples/manifest.json')).json())).samples.find(x => x.title === 'Plane Truss by the Stiffness Method');
    check(!!m && m.python === true && m.group === 'Python', 'the truss sample is in the Python group');
    const src = await page.evaluate(async (f) => (await fetch(f)).text(), m.file);
    check(!src.includes(String.fromCharCode(0x2014)), 'the truss sample has no em-dash');
    const probe = src.replace('#end python', 'print("FULL", repr(N), repr([R_x1, R_y1, R_y3]), repr([-u[3] * 1000.0, -u[7] * 1000.0]))\n#end python');
    check(probe !== src, 'the probe line was added to the sample block');
    await setText(page, probe);
    await yes(page).click({ timeout: 20000 });
    const t = await frameText(page, 'FULL', 60000);
    const mm = /FULL \[([^\]]*)\] \[([^\]]*)\] \[([^\]]*)\]/.exec(t);
    check(!!mm, 'the truss block ran and printed its full-precision numbers ' + (mm ? '' : JSON.stringify(t.slice(0, 300))));
    if (mm) {
        const N = mm[1].split(',').map(Number), R = mm[2].split(',').map(Number), D = mm[3].split(',').map(Number);
        const rel = (a, b) => Math.abs(a - b) / Math.max(Math.abs(b), 1e-12);
        const s = 3 / Math.sqrt(13), c = 2 / Math.sqrt(13);
        // member order in the sample: 1-2, 2-3, 4-5, 1-4, 4-2, 2-5, 5-3
        const hand = [20 + 77.5 * c / s, 102.5 * c / s, -96.666666666666667, -77.5 / s, 37.5 / s, 42.5 / s, -102.5 / s];
        const literal = [71.666667, 68.333333, -96.666667, -93.143408, 45.069391, 51.078643, -123.189669];
        check(hand.every((h, i) => rel(h, literal[i]) < 1e-6), 'the hand values written in the comment agree with their closed forms');
        check(N.length === 7 && N.every((v, i) => rel(v, hand[i]) < 1e-6), `member forces agree with the method of joints to 1e-6 (${N.map(v => v.toFixed(4)).join(', ')})`);
        check(rel(R[0], -20) < 1e-6 && rel(R[1], 77.5) < 1e-6 && rel(R[2], 102.5) < 1e-6, `reactions: Rx1 = -20, Ry1 = 77.5, Ry3 = 102.5 (${R.map(v => v.toFixed(4)).join(', ')})`);
        check(Math.abs(-20 + 20) < 1e-9 && Math.abs(R[1] + R[2] - 180) < 1e-9 && Math.abs(R[0] + 20) < 1e-9, 'the reactions satisfy equilibrium: sum Fx = 0 and sum Fy = 0');
        // moment equilibrium about joint 1: 8 Ry3 - (2*40 + 6*60 + 4*80) - 3*20 ... loads: joint 4 at (2,3): Fx 20 Fy -40; joint 5 (6,3): Fy -60; joint 2 (4,0): Fy -80
        const M1 = 8 * R[2] + (2 * -40 - 3 * 20) + 6 * -60 + 4 * -80;
        check(Math.abs(M1) < 1e-9, `the reactions satisfy moment equilibrium about joint 1 (${M1.toExponential(2)})`);
        const E = 200, L = [4, 4, 4, Math.sqrt(13), Math.sqrt(13), Math.sqrt(13), Math.sqrt(13)], A = [1200, 1200, 1200, 800, 800, 800, 800];
        const f2 = [1 / 3, 1 / 3, -2 / 3, -0.5 / s, 0.5 / s, 0.5 / s, -0.5 / s];
        const f4 = [0.5, 1 / 6, -1 / 3, -0.75 / s, -0.25 / s, 0.25 / s, -0.25 / s];
        const vw = f => f.reduce((acc, fi, i) => acc + fi * hand[i] * L[i] / (E * A[i]), 0) * 1000;
        check(rel(vw(f2), 6.083367) < 1e-6 && rel(vw(f4), 4.090834) < 1e-6, `the virtual-work values are 6.083367 and 4.090834 mm (${vw(f2).toFixed(6)}, ${vw(f4).toFixed(6)})`);
        check(rel(D[0], vw(f2)) < 1e-6 && rel(D[1], vw(f4)) < 1e-6, `deflections at joints 2 and 4 match virtual work to 1e-6 (${D.map(v => v.toFixed(6)).join(', ')})`);
    }
    // the sample as shipped: no error lines, a figure, and the Calcpad lines after the block use the exports
    await setText(page, src);
    const t2 = (await frameText(page, 'Reactions and deflection', 60000)).replace(/\s+/g, ' ');
    check(!/Python error|Error in "/.test(t2), 'the truss sample shows no error line');
    check(await frameHas(page, 'svg.cprs-py-figure'), 'the truss sample draws its figure');
    check(/Largest vertical deflection 6\.083 mm at joint 2/.test(t2), 'the printed maximum deflection is 6.083 mm at joint 2');
    check(/Nt_max_kN = 71\.67/.test(t2) && /Nc_max_kN = 123\.19/.test(t2) && /deltamax_mm = 6\.08/.test(t2), 'the maximum tension, compression and deflection are exported');
    check(/ηt = .*= 0\.185/.test(t2) && /ηc = .*= 0\.477/.test(t2) && /within their capacity/.test(t2), 'the Calcpad member check uses the exports (eta 0.185 and 0.477)');
    check(/Ry1 \+ Ry3 = .*= 180 kN/.test(t2), 'the Calcpad line adds the vertical reactions to 180 kN');
    await ctx.close();
}

// ---- 12. a sheet holding a bare #python block, and exported names with an underscore or a capital -------------------
{
    const { ctx, page } = await open();
    const BARE = ['<!--sheet-->', '# Bare block', '', 'a: 5', '', '#python', 'import numpy as np', 'export(k=float(np.linspace(0, 1, 5).sum()))', '#end python', '', 'r: k * 2'].join('\n');
    await setText(page, BARE);
    await yes(page).click({ timeout: 20000 });
    await frameText(page, 'Bare block');
    const tb = (await frameText(page, 'r = k', 40000)).replace(/\s+/g, ' ');
    check(/k = 2\.5/.test(tb) && /r = .*= 5\b/.test(tb), 'a bare #python block in a sheet runs, and a quantity line after it uses the export (r = 5) ' + JSON.stringify(tb.slice(0, 200)));
    check(!/Python error|#end python|#python/.test(tb), 'the bare block is not shown as sheet text and gives no error');
    const errsBare = await page.evaluate(async (content) => {
        const r = await fetch('/api/calcpad/convert', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ content }) });
        return JSON.parse(decodeURIComponent(r.headers.get('X-Calcpad-Errors') ?? '[]'));
    }, BARE);
    check(errsBare.length === 0, `no error lines for the bare block ${JSON.stringify(errsBare).slice(0, 200)}`);

    const NAMES = ['<!--sheet-->', '# Exported names', '', '```python', 'export(sigma_c=2.0, phi=3.0, Mstar=4.0)', '```', '', 'a: sigma_c*2', 'b: phi + 1', 'c: Mstar + 1'].join('\n');
    await setText(page, NAMES);
    const tn = (await frameText(page, 'c = Mstar', 40000)).replace(/\s+/g, ' ');
    check(/a = σc · 2 = 2 · 2 = 4/.test(tn) && /b = φ \+ 1 = 3 \+ 1 = 4/.test(tn) && /c = Mstar \+ 1 = 4 \+ 1 = 5/.test(tn), 'export(sigma_c, phi, Mstar) in a sheet: a = 4, b = 4, c = 5 ' + JSON.stringify(tn.slice(0, 300)));
    const errsNames = await page.evaluate(async (content) => {
        const r = await fetch('/api/calcpad/convert', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ content }) });
        return JSON.parse(decodeURIComponent(r.headers.get('X-Calcpad-Errors') ?? '[]'));
    }, NAMES);
    check(errsNames.length === 0 && !/Python error|Undefined|error/i.test(tn), `no error lines for the exported names ${JSON.stringify(errsNames).slice(0, 200)}`);
    await ctx.close();
}

// ---- 13. the section properties of the native sample, computed again in Python ---------------------------------------
{
    const { ctx, page } = await open();
    // independent values: parts b x h with centroids y above the base
    const b = [200, 12, 150], h = [16, 300, 20], y = [8, 166, 324];
    const Ap = b.map((v, i) => v * h[i]);
    const At = Ap.reduce((p, q) => p + q, 0);
    const yc = Ap.reduce((p, q, i) => p + q * y[i], 0) / At;
    const Ix = b.reduce((p, v, i) => p + v * h[i] ** 3 / 12 + Ap[i] * (y[i] - yc) ** 2, 0);
    const native = (await page.evaluate(async () => (await fetch('samples/manifest.json')).json())).samples.find(x => x.title === 'Built-up Section Properties');
    check(!!native && native.group === 'Sections' && native.python === false, 'the native section sample is in the Sections group and has no Python');
    const nsrc = await page.evaluate(async (f) => (await fetch(f)).text(), native.file);
    await setText(page, nsrc);
    const tn = (await frameText(page, 'Bending stress', 40000)).replace(/\s+/g, ' ');
    const grab = re => { const mm = re.exec(tn); return mm ? Number(mm[1]) : NaN; };
    const nat = { yc: grab(/9800 mm2 = ([0-9.]+) mm/), Ix: grab(/= ([0-9]{6,}) mm4/), At: grab(/Atot = sum \( A \) = ([0-9]+) mm2/) };
    const near = (a, bb) => Math.abs(a - bb) / Math.abs(bb) < 1e-4;
    check(near(nat.yc, yc) && near(nat.At, At) && near(nat.Ix, Ix), `the native sample gives A, y_c and I_x equal to the hand values (${JSON.stringify(nat)} against ${At}, ${yc.toFixed(3)}, ${Ix.toExponential(6)})`);
    const PY = ['"Same section in Python', '#python', 'import numpy as np', 'b = np.array([200.0, 12.0, 150.0])', 'h = np.array([16.0, 300.0, 20.0])', 'y = np.array([8.0, 166.0, 324.0])',
        'A = b * h', 'y_c = float((A * y).sum() / A.sum())', 'I_x = float((b * h**3 / 12 + A * (y - y_c) ** 2).sum())', 'print("section", repr(float(A.sum())), repr(y_c), repr(I_x))', '#end python'].join('\n');
    await setText(page, PY);
    await yes(page).click({ timeout: 20000 }).catch(() => {});
    const tp = await frameText(page, 'section 9800', 40000);
    const pm = /section ([0-9.eE+-]+) ([0-9.eE+-]+) ([0-9.eE+-]+)/.exec(tp);
    check(!!pm && Math.abs(Number(pm[1]) - At) < 1e-9 && Math.abs(Number(pm[2]) - yc) < 1e-9 && Math.abs(Number(pm[3]) - Ix) / Ix < 1e-12, `the same calculation in Python gives the same A, y_c and I_x (${pm ? pm.slice(1).join(' ') : tp.slice(0, 100)})`);
    check(pm && near(Number(pm[2]), nat.yc) && near(Number(pm[3]), nat.Ix), 'the Python and the native sample agree');
    await ctx.close();
}

// ---- 14. the sample list shows the Sections group ---------------------------------------------------------------
{
    const { ctx, page } = await open();
    await page.locator('[data-ribbon-btn=file]').click();
    await page.locator('.cprs-menu button', { hasText: 'Samples...' }).click();
    await page.locator('.cprs-samples').waitFor({ timeout: 10000 });
    const groupBtn = page.locator('.cprs-s-group', { hasText: /^Sections / });
    check(await groupBtn.count() === 1, 'the Samples browser lists a Sections group');
    await groupBtn.click();
    const titles = await page.locator('.cprs-samples .cprs-s-item .cprs-s-title').allInnerTexts();
    check(titles.length === 1 && titles[0] === 'Built-up Section Properties', `the Sections group holds the built-up section sample ${JSON.stringify(titles)}`);
    await page.locator('.cprs-s-group', { hasText: /^Python / }).click();
    const pt = await page.locator('.cprs-samples .cprs-s-item .cprs-s-title').allInnerTexts();
    check(pt.includes('Plane Truss by the Stiffness Method') && pt.includes('Beam Explorer: Plotly from Python') && pt.length === 5, `the Python group lists the truss and the Plotly explorer samples (${pt.length} samples)`);
    await ctx.close();
}

// ---- 15. the Plotly API: figure JSON, lazy load, named errors, offline note, Word and Excel ------------------------------
const CDN = 'https://cdn.plot.ly/plotly-2.26.0.min.js';
const online = await fetch(CDN, { method: 'HEAD' }).then((r) => r.ok, () => false);
if (!online) console.log(`skip the Plotly chart checks (sections 15b and 16): ${CDN} is not reachable (offline?); the figure JSON, error, offline-note and export checks still run`);
const FIG = ['"Plotly check', '#python', 'import plotly.graph_objects as go', 'from plotly.subplots import make_subplots', 'import numpy as np',
    'x = np.linspace(0, 4, 5)',
    'fig = make_subplots(rows=2, cols=1, shared_xaxes=True, subplot_titles=["Top", "Bottom"])',
    'fig.add_trace(go.Scatter(x=x, y=x * 2, name="up", line_color="#d62728"), row=1, col=1)',
    'fig.add_trace(go.Bar(x=[1, 2, 3], y=[3, float("nan"), 1], name="bars"), row=2, col=1)',
    'fig.add_vline(x=2, line_dash="dot")',
    'fig.update_layout(title="Check chart", height=420, xaxis2_title="x (m)", updatemenus=[dict(type="dropdown", buttons=[dict(label="all", method="update", args=[{"visible": [True, True]}]), dict(label="up only", method="update", args=[{"visible": [True, False]}])])], sliders=[dict(active=0, steps=[dict(label=str(k), method="restyle", args=[{"y": [[k * v for v in [0, 1, 2, 3, 4]]]}, [0]]) for k in (1, 2, 3)])])',
    'fig.show()', 'print("after the chart")', '#end python', 'z = 1+1'].join('\n');
const plotlyFrame = async (page, ms = 30000) => {
    const t0 = Date.now();
    while (Date.now() - t0 < ms) {
        for (const f of page.frames()) if (f !== page.mainFrame() && await f.locator('.cprs-plotly').count().catch(() => 0)) return f;
        await page.waitForTimeout(300);
    }
    return null;
};
/** Waits (polling, the frame's CSP does not allow waitForFunction) until the chart is drawn or the note says why not. */
const chartState = async (page, f, ms = 60000) => {
    const t0 = Date.now();
    while (Date.now() - t0 < ms) {
        const st = await f.evaluate(() => ({ svg: !!document.querySelector('.cprs-plotly .main-svg'), note: document.querySelector('.cprs-plotly-note')?.textContent ?? '', shown: document.querySelector('.cprs-plotly-note')?.style.display !== 'none' })).catch(() => null);
        if (st && (st.svg || (st.shown && /connection|could not/.test(st.note)))) return st;
        await page.waitForTimeout(300);
    }
    return null;
};
{
    // 15a. lazy: a block without plotly never loads the shim; a block that imports it does; the figure JSON reaches the page
    const { ctx, page, requests, errors } = await open();
    await setText(page, ['"No plotly', '#python', 'print("plain")', '#end python'].join('\n'));
    await yes(page).click({ timeout: 20000 });
    await frameText(page, 'plain');
    check(!requests.some((u) => /py\/lib\/plotly\//.test(u)), 'a Python block that does not mention plotly never fetches the plotly shim');
    await page.route(CDN, (r) => r.abort());     // 15a looks at the markup only
    await setText(page, FIG);
    const t = await frameText(page, 'after the chart');
    check(/after the chart/.test(t) && /z = .*2/.test(t.replace(/\s+/g, ' ')), 'a block with fig.show() runs, prints after it, and the Calcpad line after the block calculates');
    check(requests.some((u) => /py\/lib\/plotly\/graph_objects\.py/.test(u)) && requests.some((u) => /py\/lib\/plotly\/subplots\.py/.test(u)), 'the plotly shim is fetched when a block mentions plotly');
    const res = await page.evaluate(async (content) => {
        const r = await fetch('/api/calcpad/convert', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ content, forPrint: true, includeLineAnchors: false }) });
        return await r.text();
    }, FIG);
    const m = /<div class="cprs-plotly" data-fig="([A-Za-z0-9+\/=]+)"><\/div>/.exec(res);
    check(!!m, 'the report HTML carries the figure as a cprs-plotly container');
    const fig = m ? JSON.parse(Buffer.from(m[1], 'base64').toString('utf8')) : {};
    const d = fig.data ?? [], L = fig.layout ?? {};
    check(d.length === 2 && d[0].type === 'scatter' && d[1].type === 'bar', `two traces: scatter and bar (${d.map((x) => x.type)})`);
    check(JSON.stringify(d[0]?.y) === '[0,2,4,6,8]' && d[0]?.line?.color === '#d62728' && d[0]?.name === 'up', 'the scatter keeps its numbers and its line_color is expanded to line.color');
    check(d[1]?.y?.[1] === null && d[1]?.y?.[0] === 3, 'NaN becomes null, other numbers are kept');
    check(d[0]?.xaxis === 'x' && d[0]?.yaxis === 'y' && d[1]?.xaxis === 'x2' && d[1]?.yaxis === 'y2', 'row and col put the traces on the subplot axes');
    check(L.title === 'Check chart' && L.height === 420 && L.xaxis2?.title === 'x (m)', 'update_layout keywords are written as given (title, height, xaxis2_title)');
    check(L.xaxis?.matches === 'x2' && L.xaxis?.showticklabels === false && L.xaxis2?.domain?.[0] === 0 && L.xaxis2?.domain?.[1] === 1, 'shared_xaxes: the top axis follows the bottom one and hides its labels');
    check(Array.isArray(L.shapes) && L.shapes.length === 2 && L.shapes[0].yref === 'y domain' && L.shapes[1].yref === 'y2 domain' && L.shapes[0].line?.dash === 'dot', 'add_vline draws one line per subplot, with line_dash expanded');
    check(L.annotations?.length === 2 && L.annotations.map((a) => a.text).sort().join() === 'Bottom,Top', 'subplot_titles become annotations');
    check(L.updatemenus?.[0]?.buttons?.length === 2 && L.updatemenus[0].buttons[1].args[0].visible[1] === false && L.sliders?.[0]?.steps?.length === 3 && L.sliders[0].steps[2].args[0].y[0][4] === 12, 'updatemenus and sliders pass through unchanged');
    check(/Interactive chart: Check chart\./.test(res), 'the page holds a note naming the chart, for print and export');
    check(errors.length === 0, `no page errors${errors.length ? ': ' + errors[0] : ''}`);

    // 15c. Word and Excel: a placeholder paragraph naming the chart, not the figure
    const files = await page.evaluate(async (content) => {
        const o = {};
        for (const ep of ['docx', 'xlsx']) {
            const r = await fetch(`/api/calcpad/${ep}`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ content, forPrint: true, includeLineAnchors: false }) });
            o[ep] = { status: r.status, bytes: [...new Uint8Array(await r.arrayBuffer())] };
        }
        return o;
    }, FIG);
    const { writeFileSync } = await import('node:fs');
    const { execFileSync } = await import('node:child_process');
    const unzipAll = (name, bytes) => { const f = join(out, name); writeFileSync(f, Buffer.from(bytes)); try { return execFileSync('unzip', ['-p', f], { encoding: 'utf8', maxBuffer: 64 << 20 }); } catch { return ''; } };
    const docx = unzipAll('plotly-check.docx', files.docx.bytes), xlsx = unzipAll('plotly-check.xlsx', files.xlsx.bytes);
    check(files.docx.status === 200 && /Interactive chart: Check chart/.test(docx) && /not carried by print or export/.test(docx), 'Word: a paragraph names the chart ("Interactive chart: Check chart") where it stood');
    check(!/data-fig|cprs-plotly/.test(docx), 'Word: the figure data is not in the document');
    check(files.xlsx.status === 200 && /Interactive chart: Check chart/.test(xlsx) && !/data-fig|cprs-plotly/.test(xlsx), 'Excel: the report sheet holds the placeholder paragraph naming the chart, not the figure');
    await ctx.close();
}

{
    // the named errors: anything the shim does not provide fails by name, and so does a spelling Plotly.js would ignore
    const { ctx, page } = await open();
    const ERR = ['"Plotly errors', '#python', 'import plotly.graph_objects as go', 'import plotly.graph_objs as gobjs',
        'def tell(f):', '    try:', '        f()', '        print("no error")', '    except Exception as e:', '        print("E> " + type(e).__name__ + ": " + str(e)[:150])',
        'fig = go.Figure()',
        'tell(lambda: go.Sunburst)',
        'tell(lambda: go.Scatter(x=[1], line={"line_color": 1}))',
        'tell(lambda: fig.write_html("a.html"))',
        'tell(lambda: fig.update_layout(template="plotly_white"))',
        'tell(lambda: fig.show(renderer="browser"))',
        'tell(lambda: gobjs.Foo)',
        'tell(lambda: go.Scatter(x=[object()]))',
        'tell(lambda: setattr(fig, "data", 1))',
        'tell(lambda: __import__("plotly.express"))',
        'print("magic", go.Scatter(line_color="red", marker_size=4).to_dict() == {"type": "scatter", "line": {"color": "red"}, "marker": {"size": 4}}, gobjs.Scatter is go.Scatter)',
        'go.Surface3d', '#end python'].join('\n');
    await setText(page, ERR);
    await yes(page).click({ timeout: 20000 });
    const t = (await frameText(page, 'Python error', 40000)).replace(/[ \t]+/g, ' ');
    check(/E> NotImplementedError: plotly\.graph_objects\.Sunburst is not available/.test(t), 'an unsupported trace type fails by name (go.Sunburst) ' + JSON.stringify(t.slice(0, 200)));
    check(/E> ValueError: go\.Scatter line: the key 'line_color' is written with underscores inside a dict/.test(t), "a magic-underscore key inside a dict fails naming the key ('line_color')");
    check(/E> NotImplementedError: go\.Figure\.write_html is not available/.test(t), 'an unsupported Figure method fails by name (write_html)');
    check(/E> ValueError: Figure\.update_layout: template='plotly_white': named templates are not available/.test(t), 'a named template fails by name instead of being ignored');
    check(/E> NotImplementedError: Figure\.show\(renderer=\.\.\.\) is not available/.test(t), 'an unsupported fig.show option fails by name (renderer)');
    check(/E> NotImplementedError: plotly\.graph_objs\.Foo is not available/.test(t), 'plotly.graph_objs fails by name for an unknown class');
    check(/E> TypeError: go\.Scatter x: a object cannot be sent to Plotly/.test(t), 'a value that is not JSON-able fails naming the property');
    check(/E> ImportError: no module named 'plotly\.express'/.test(t), 'plotly.express is an ImportError naming the module');
    check(/E> AttributeError: go\.Figure: cannot set 'data'/.test(t), 'assigning to a figure attribute fails instead of being ignored');
    check(/magic True True/.test(t), 'line_color and marker_size keywords are expanded to nested dicts; graph_objs is graph_objects');
    check(/Python error on line 22: NotImplementedError: plotly\.graph_objects\.Surface3d is not available/.test(t), 'an uncaught unsupported name is a Python error on its own line (22)');
    await ctx.close();
}

// ---- 15b. the chart in Chromium; the offline note ------------------------------------------------------------------------
{
    const { ctx, page } = await open();
    await page.route(CDN, (r) => r.abort());
    await setText(page, FIG);
    await yes(page).click({ timeout: 20000 });
    const f = await plotlyFrame(page);
    check(!!f, 'the chart container is on the page (offline run)');
    const st = f ? await chartState(page, f, 30000) : null;
    check(!!st && !st.svg && st.shown && /Interactive chart: Check chart\. Interactive charts need a connection to load Plotly/.test(st.note), 'with the CDN blocked the page says interactive charts need a connection, naming the chart (no silent blank) ' + JSON.stringify(st));
    await page.screenshot({ path: join(out, 'plotly-offline.png') });
    await ctx.close();
}
if (online) {
    const LIBS = JSON.parse((await import('node:fs')).readFileSync(new URL('../calcpad-web/src/wasm/library-defaults.json', import.meta.url), 'utf8'));
    const plotlyLib = LIBS.find((l) => l.id === 'plotly');
    const { ctx, page, requests, errors } = await open();
    await setText(page, FIG);
    await yes(page).click({ timeout: 20000 });
    const f = await plotlyFrame(page);
    const st = f ? await chartState(page, f) : null;
    check(!!st && st.svg, 'the chart is drawn by Plotly.js in the report page ' + JSON.stringify(st));
    if (st?.svg) {
        const info = await f.evaluate((cdn) => {
            const gd = document.querySelector('.cprs-plotly');
            const sc = [...document.scripts].find((s) => s.src === cdn);
            return { traces: gd.data.length, types: gd.data.map((t) => t.type), subplots: Object.keys(gd._fullLayout._plots ?? {}).length, svgs: document.querySelectorAll('.cprs-plotly .main-svg').length,
                integrity: sc?.integrity, cors: sc?.crossOrigin, note: document.querySelector('.cprs-plotly-note').style.display, font: gd._fullLayout.font.color, bg: getComputedStyle(document.querySelector('.cprs-plotly svg.main-svg')).backgroundColor,
                menu: document.querySelectorAll('.updatemenu-header').length, sliders: document.querySelectorAll('.slider-rail-rect').length, url: window.Plotly?.version };
        }, CDN);
        check(info.traces === 2 && info.types.join() === 'scatter,bar' && info.subplots === 2, `Chromium draws two traces on two subplots (${JSON.stringify([info.traces, info.types, info.subplots])})`);
        check(info.integrity === plotlyLib.files[0].integrity && info.cors === 'anonymous', 'the Plotly script tag carries the library list integrity hash and crossorigin=anonymous');
        check(requests.filter((u) => /plot\.ly/.test(u)).every((u) => u === CDN) && requests.some((u) => u === CDN), 'the only Plotly request is the pinned CDN file');
        check(info.note === 'none' && info.menu === 1 && info.sliders === 1, `the note is hidden once the chart is live, and the native menu and slider are drawn (${info.menu}, ${info.sliders})`);
        check(info.font === '#1c2129' && info.bg === 'rgba(0, 0, 0, 0)', `light theme: dark text, transparent paper (${info.font}, ${info.bg})`);
        await page.screenshot({ path: join(out, 'plotly-light.png') });
    }
    check(errors.length === 0, `no page errors while drawing${errors.length ? ': ' + errors[0] : ''}`);
    await ctx.close();
    // dark theme: light text, and no white card behind the chart (the app's dark rule paints every svg white; the chart opts out)
    const dk = await open('dark');
    await setText(dk.page, FIG);
    await yes(dk.page).click({ timeout: 20000 });
    const fd = await plotlyFrame(dk.page);
    const sd = fd ? await chartState(dk.page, fd) : null;
    if (sd?.svg) {
        const info = await fd.evaluate(() => {
            const sv = document.querySelector('.cprs-plotly svg.main-svg'), cs = getComputedStyle(sv);
            return { font: document.querySelector('.cprs-plotly')._fullLayout.font.color, bg: cs.backgroundColor, pad: cs.paddingLeft, radius: cs.borderTopLeftRadius, body: getComputedStyle(document.body).backgroundColor };
        });
        check(info.font === '#e7ebf2' && info.bg === 'rgba(0, 0, 0, 0)' && info.pad === '0px' && info.radius === '0px', `dark theme: light text and no white card behind the chart (${JSON.stringify(info)})`);
        await dk.page.screenshot({ path: join(out, 'plotly-dark.png') });
    } else check(false, 'the chart is drawn in the dark theme ' + JSON.stringify(sd));
    await dk.ctx.close();
}

// ---- 16. the beam explorer from Python: native menu and sliders change the plotted data ------------------------------------
if (online) {
    const { ctx, page, errors } = await open();
    await page.evaluate(async () => { const m = (await (await fetch('samples/manifest.json')).json()).samples.find(s => s.title === 'Beam Explorer: Plotly from Python'); await window.calcpadFiles.samples.openSample(m); });
    await yes(page).click({ timeout: 20000 });
    const t = await frameText(page, 'Design check', 60000);
    check(/Equilibrium: the reactions sum to the load/.test(t) && /UDL on all spans 300\.0 of 300\.0 kN/.test(t), 'the Python solve ran and its equilibrium check holds');
    check(/η = /.test(t) && /M_Ed|MEd/.test(t) && /within the capacity/.test(t), 'the Calcpad design check uses the exported envelope');
    check(/UDL on all spans\s+sagging\s+34\.65\s+hogging\s+-63\.61/.test(t), 'the printed table gives the hogging moment -63.61 kN.m for the UDL on all spans');
    check(/The chart holds \d+ kB of precomputed results/.test(t), 'the sample prints the size of its chart data: ' + (/The chart holds \d+ kB[^\n]*/.exec(t)?.[0] ?? ''));
    const f = await plotlyFrame(page, 60000);
    const st = f ? await chartState(page, f) : null;
    check(!!st && st.svg, 'the beam explorer chart is drawn');
    const src = await page.evaluate(async () => { const m = (await (await fetch('samples/manifest.json')).json()).samples.find(x => x.title === 'Beam Explorer: Plotly from Python'); return (await fetch(m.file)).text(); });
    check(!/<script|<style|<div|<select|<input|<label/i.test(src) && !src.includes(String.fromCharCode(0x2014)), 'the sample source has no script, no HTML controls and no em-dash')
    const read = () => f.evaluate(() => {
        const gd = document.querySelector('.cprs-plotly');
        return { n: gd.data.length, vis: gd.data.map((t) => t.visible !== false && t.visible !== 'legendonly'), y: gd.data.map((t) => t.y), x: gd.data[2].x, names: gd.data.map((t) => t.name),
            active: gd.layout.sliders.map((s) => s.active), menu: gd.layout.updatemenus[0].active, shape4: gd.layout.shapes[4].x0, shape4b: gd.layout.shapes[4].x1 };
    });
    const mn = (a) => Math.min(...a), mx = (a) => Math.max(...a);
    const D0 = await read();
    check(D0.n === 14, `fourteen traces: the band, three uniform cases (M, V, deflection) and the point load (${D0.n})`);
    // three-moment equations, spans 5, 6, 4 m, w = 20 kN/m, simple supports at A and D:
    //   2 M_B (L1 + L2) + M_C L2 = -w (L1^3 + L2^3) / 4;   M_B L2 + 2 M_C (L2 + L3) = -w (L2^3 + L3^3) / 4
    const [L1, L2, L3, w] = [5, 6, 4, 20];
    const a11 = 2 * (L1 + L2), a12 = L2, a21 = L2, a22 = 2 * (L2 + L3), b1 = -w * (L1 ** 3 + L2 ** 3) / 4, b2 = -w * (L2 ** 3 + L3 ** 3) / 4;
    const det = a11 * a22 - a12 * a21, MB = (b1 * a22 - a12 * b2) / det, MC = (a11 * b2 - a21 * b1) / det;
    const at = (D, tr, xv) => D.y[tr][D.x.findIndex((v) => Math.abs(v - xv) < 1e-6)];
    check(Math.abs(MB + 63.61) < 0.01 && Math.abs(MC + 50.91) < 0.01, `the three-moment solution is M_B = ${MB.toFixed(2)} and M_C = ${MC.toFixed(2)} kN.m`);
    check(Math.abs(at(D0, 2, 5) - MB) < 0.01 && Math.abs(at(D0, 2, 11) - MC) < 0.01 && Math.abs(mn(D0.y[2]) - MB) < 0.01, `the plotted UDL-on-all-spans moment is ${at(D0, 2, 5)} over B and ${at(D0, 2, 11)} over C (hogging, as the three-moment solution)`);
    check(D0.vis[0] && D0.vis[1] && D0.vis[2] && !D0.vis[5] && !D0.vis[8] && D0.vis[11], 'initially the UDL-on-all-spans case, the band and the point load are shown');
    // the real controls
    const base8 = mn(D0.y[8]), baseHi = mx(D0.y[0]);
    await f.locator('.updatemenu-header').first().click();
    await f.locator('.updatemenu-item-text', { hasText: 'UDL on span 2 only' }).first().click();
    await page.waitForTimeout(500);
    const D1 = await read();
    check(D1.vis[8] && D1.vis[9] && D1.vis[10] && !D1.vis[2] && !D1.vis[5] && D1.vis[11] && D1.menu === 2, `the menu entry "UDL on span 2 only" shows that case and hides the other (visible: ${JSON.stringify(D1.vis)}, active ${D1.menu})`);
    const rail = (i) => f.locator('.slider-rail-rect').nth(i);
    const clickRail = async (i, frac) => { await rail(i).scrollIntoViewIfNeeded(); await page.waitForTimeout(200); const bb = await rail(i).boundingBox(); await page.mouse.click(bb.x + Math.min(bb.width - 1, Math.max(1, bb.width * frac)), bb.y + bb.height / 2); await page.waitForTimeout(600); };
    await clickRail(1, 1);
    const D2 = await read();
    const fac = 0.5 + 0.05 * D2.active[1];
    check(D2.active[1] === 30 && Math.abs(fac - 2) < 1e-9, `the load factor slider, clicked at its end, is at step ${D2.active[1]} (factor ${fac.toFixed(2)})`);
    check(Math.abs(mn(D2.y[8]) - 2 * base8) < 0.03 && Math.abs(mx(D2.y[0]) - 2 * baseHi) < 0.03 && Math.abs(mn(D2.y[2]) - 2 * mn(D0.y[2])) < 0.03,
        `factor 2 doubles the plotted moments: span-2 case ${mn(D2.y[8])} against 2 x ${base8}, band ${mx(D2.y[0])} against 2 x ${baseHi}`);
    check(JSON.stringify(D2.y[11]) === JSON.stringify(D1.y[11]) && JSON.stringify(D2.x) === JSON.stringify(D0.x), 'the load factor slider leaves the point load and the x values alone');
    await clickRail(0, 0.5);
    const D3 = await read();
    const p = 0.5 + D3.active[0];
    check(D3.active[0] !== D0.active[0] && D3.shape4 === p && D3.shape4b === p && D3.names[11] === `P = 60 kN at x = ${p.toFixed(1)} m`, `the point load slider moved the load to x = ${p} m (marker line ${D3.shape4}, trace name "${D3.names[11]}")`);
    check(JSON.stringify(D3.y[11]) !== JSON.stringify(D2.y[11]) && JSON.stringify(D3.y[8]) === JSON.stringify(D2.y[8]) && JSON.stringify(D3.vis) === JSON.stringify(D2.vis), 'the point load slider changes only the point load traces');
    // independent check of the moved point load: the reactions of a three-span beam sum to P, so the shear just after the last support is the sum of the end shears
    const Mp = D3.y[11];
    check(Math.abs(Mp[0]) < 0.01 && Math.abs(Mp[Mp.length - 1]) < 0.01, `the point load moment is zero at both free ends (${Mp[0]}, ${Mp[Mp.length - 1]})`);
    await page.setViewportSize({ width: 1400, height: 1800 });
    await page.waitForTimeout(800);
    await f.locator('.cprs-plotly-wrap').first().screenshot({ path: join(out, 'beam-explorer-plotly.png') });
    check(errors.length === 0, `no page errors${errors.length ? ': ' + errors[0] : ''}`);
    await ctx.close();
    // the same sample in the dark theme
    const dk = await open('dark');
    await dk.page.evaluate(async () => { const m = (await (await fetch('samples/manifest.json')).json()).samples.find(s => s.title === 'Beam Explorer: Plotly from Python'); await window.calcpadFiles.samples.openSample(m); });
    await yes(dk.page).click({ timeout: 20000 });
    const fd = await plotlyFrame(dk.page, 60000);
    const sd = fd ? await chartState(dk.page, fd) : null;
    check(!!sd?.svg, 'the beam explorer is drawn in the dark theme');
    await dk.page.setViewportSize({ width: 1400, height: 1800 });
    await dk.page.waitForTimeout(800);
    if (fd) await fd.locator('.cprs-plotly-wrap').first().screenshot({ path: join(out, 'beam-explorer-plotly-dark.png') });
    await dk.ctx.close();
}

// ---- 9z. the math and numpy functions a plotting script reaches for next, and long list literals --------------------------
{
    const { ctx, page } = await open();
    const FUNCS = ['"Stand-in functions', '#python', 'import math', 'from math import hypot', 'import numpy as np',
        'print("m1", hypot(3, 4), math.hypot(3, 4, 12), math.prod([1, 2, 3, 4]), math.fsum([0.1] * 10), math.dist((0, 0), (3, 4)))',
        'print("m2", math.isqrt(17), math.comb(5, 2), math.gcd(12, 18), round(math.atan2(1, 1), 6), math.copysign(2, -1), math.isclose(1.0, 1.0 + 1e-10))',
        'print("n1", np.hypot(np.array([3.0, 5.0]), np.array([4.0, 12.0])).tolist(), np.round(np.arctan2([1.0, -1.0], [-1.0, 0.0]), 6).tolist())',
        'X, Y = np.meshgrid(np.array([1, 2, 3]), np.array([10, 20]))',
        'print("n2", X.shape, X.tolist(), Y.tolist())',
        'I, J = np.meshgrid([1, 2, 3], [10, 20], indexing="ij")',
        'print("n3", I.shape, I.tolist(), J.tolist())',
        'print("n4", np.round(np.sqrt(X ** 2 + Y ** 2), 3)[0].tolist(), np.outer([1, 2], [3, 4]).tolist())',
        'print("n5", np.interp([0, 1.5, 4], [1, 2, 3], [3, 2, 0]).tolist(), np.round(np.polyfit([-1, 0, 1, 2], [2, 1, 2, 5], 2), 10).tolist(), np.polyval([1, 0, -1], 3))',
        'big = [' + Array.from({ length: 4000 }, (_, i) => i).join(', ') + ']',
        'print("n6", len(big), big[-1])',
        '#end python'].join('\n');
    await setText(page, FUNCS);
    await yes(page).click({ timeout: 20000 });
    const f = (await frameText(page, 'n6', 60000)).replace(/\s+/g, ' ');
    check(/m1 5\.0 13\.0 24 1\.0 5\.0/.test(f), 'math.hypot (2 and 3 arguments), math.prod, math.fsum, math.dist ' + JSON.stringify(f.slice(0, 300)));
    check(/m2 4 10 6 0\.785398 -2\.0 True/.test(f), 'math.isqrt, comb, gcd, atan2, copysign, isclose');
    check(/n1 \[5\.0, 13\.0\] \[2\.356194, -1\.570796\]/.test(f), 'numpy.hypot and numpy.arctan2 on arrays');
    check(/n2 \(2, 3\) \[\[1, 2, 3\], \[1, 2, 3\]\] \[\[10, 10, 10\], \[20, 20, 20\]\]/.test(f), 'numpy.meshgrid with indexing="xy" (rows follow the second vector)');
    check(/n3 \(3, 2\) \[\[1, 1\], \[2, 2\], \[3, 3\]\] \[\[10, 20\], \[10, 20\], \[10, 20\]\]/.test(f), 'numpy.meshgrid with indexing="ij"');
    check(/n4 \[10\.05, 10\.198, 10\.44\] \[\[3, 4\], \[6, 8\]\]/.test(f), 'numpy.sqrt of a meshgrid expression, numpy.outer');
    check(/n5 \[3\.0, 2\.5, 0\.0\] \[1\.0, 0\.0, 1\.0\] 8/.test(f), 'numpy.interp, numpy.polyfit (x^2 + 1), numpy.polyval');
    check(/n6 4000 3999/.test(f), 'a list literal of 4,000 numbers runs');
    // Past the limit (16,000 elements) the block is refused before it runs, and the error says what to do instead.
    await setText(page, ['"Too long', '#python', 'big = [' + '0,'.repeat(20000) + '0]', '#end python'].join('\n'));
    const e = (await frameText(page, 'Python error', 60000)).replace(/\s+/g, ' ');
    check(/Python error on line 3/.test(e) && /a literal of 20001 elements is too long \(at most 16000\)/.test(e)
        && /long list literals are limited: build the list in a loop or parse a text string/.test(e), 'a list literal past the limit names the way round it ' + JSON.stringify(e.slice(0, 300)));
    // A literal just under it runs.
    await setText(page, ['"Long', '#python', 'big = [' + '1,'.repeat(15999) + '1]', 'print("n7", len(big), sum(big))', '#end python'].join('\n'));
    check(/n7 16000 16000/.test(await frameText(page, 'n7', 60000)), 'a list literal of 16,000 numbers runs');
    await ctx.close();
}

// ---- the longer time limit: a block that needs 13 s is stopped at 10 s, then runs when the user takes the 60 s limit --------
{
    const lg = await open();
    const sheet = ['"Slow block', '#python', 'import time', 't = time.ticks_ms()', 'while time.ticks_ms() - t < 13000:', '    pass', 'print("slow done")', '#end python'].join('\n');
    await setText(lg.page, sheet);
    await lg.page.locator('#cprs-python-yes').click({ timeout: 30000 });
    await lg.page.locator('#cprs-python-longer-yes').waitFor({ timeout: 30000 });
    await lg.page.locator('#cprs-python-longer-yes').click();
    check((await frameText(lg.page, 'slow done', 60000)).length > 0, 'a 13 s block finishes after "Run again with 60 s"');
    // the longer limit stays for the tab: no second question for the next slow run
    await setText(lg.page, sheet.replace('slow done', 'slow again'));
    const again = await frameText(lg.page, 'slow again', 60000);
    check(again.length > 0 && (await lg.page.locator('#cprs-python-longer').count()) === 0, 'the longer limit is kept for the tab');
    await lg.ctx.close();
}

await browser.close();
console.log(fails.length ? `\n${fails.length} FAILED` : '\nall passed');
process.exit(fails.length ? 1 : 0);
