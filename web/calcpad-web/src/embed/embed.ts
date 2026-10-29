// The embed view (docs/embedding.md): `/calcs/?embed=1&sample=<id>` shows one sample as an input form and its live
// results, for an iframe on another site or on our own landing pages. It reuses the app's calculation path (the wasm
// transport answers /api/calcpad/convert in the page) but none of the app's chrome or state: no editor, no ribbon,
// no tabs, nothing written to localStorage or IndexedDB, no service worker registration, no network request beyond
// the app's own static files.
//
// Parameters: sample=<manifest id>, theme=light|dark|auto, mode=input|results|both, hide=title,
// embed=edit (a compact text editor below), allowpython=1 (Python blocks may run, after the usual question).
//
// postMessage API (only with the embedding page, event.source === window.parent):
//   to the host    {type:'civilkit-embed-height', height}   on every layout change
//                  {type:'civilkit-ready', sample, inputs}   once the form is built
//                  {type:'civilkit-results', sample, values:{name:{value, unit, text}}}   after each calculation
//                  {type:'civilkit-error', errors:[...]}     a civilkit-set that was refused (in full or in part)
//   from the host  {type:'civilkit-set', values:{L: 8}}     set inputs by name (checked: a known name, a finite number
//                                                            within the slider bounds, one of the options)
//                  {type:'civilkit-get'}                    post the last results again
// Inlined (?inline) rather than imported as style sheets: the entry's one dynamic import would otherwise preload the
// app's style sheets instead of these.
import tokensCss from '../styles/civilkit-tokens.css?inline';
import embedCss from './embed.css?inline';
import { installWasmTransport } from '../wasm/transport';
import { compileSheet, isSheetText } from '../wasm/sheet';
import { loadManifest, sampleSource, type Sample } from '../wasm/samples';
import { buildApiSettings, DEFAULT_CALCPAD_SETTINGS } from 'calcpad-frontend/types/settings';
import { checkValue, parseInputs, promoteNumberLines, pythonBlocks, withoutPython, type EmbedInput } from './inputs';

const DEFAULT_SAMPLE = 'template-simply-supported-beam';
const params = new URLSearchParams(window.location.search);
const editMode = params.get('embed') === 'edit';
const sampleId = params.get('sample') || DEFAULT_SAMPLE;
const themeParam = ((v) => (v === 'light' || v === 'dark' ? v : 'auto'))(params.get('theme'));
const viewMode = ((v) => (v === 'input' || v === 'results' ? v : 'both'))(params.get('mode'));
const hidden = new Set((params.get('hide') ?? '').split(',').map((s) => s.trim()));
const allowPython = params.get('allowpython') === '1';

const base = new URL(import.meta.env.BASE_URL || './', document.baseURI).href;
const framed = window.parent !== window;
/** Where messages to the host go: its origin when the browser tells us (Chromium), else any (the host is whoever framed us). */
const hostOrigin = (() => {
    const o = (window.location as Location & { ancestorOrigins?: DOMStringList }).ancestorOrigins?.[0];
    return o && o !== 'null' ? o : '*';
})();
const toHost = (msg: Record<string, unknown>) => { if (framed) window.parent.postMessage(msg, hostOrigin); };

const transport = installWasmTransport();
transport.pythonScope = () => 'embed';

const esc = (s: string) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
const el = <K extends keyof HTMLElementTagNameMap>(tag: K, props: Partial<HTMLElementTagNameMap[K]> & Record<string, unknown> = {}, ...kids: (Node | string)[]) => {
    const e = document.createElement(tag);
    for (const [k, v] of Object.entries(props)) {
        if (k.startsWith('data-') || k.startsWith('aria-') || k === 'role' || k === 'for') e.setAttribute(k, String(v));
        else (e as unknown as Record<string, unknown>)[k] = v;
    }
    e.append(...kids);
    return e;
};

document.head.append(Object.assign(document.createElement('style'), { textContent: tokensCss + embedCss }));

// ---- theme --------------------------------------------------------------------------------------------------------
const dark = window.matchMedia('(prefers-color-scheme: dark)');
const theme = (): 'light' | 'dark' => (themeParam === 'auto' ? (dark.matches ? 'dark' : 'light') : themeParam);
const paintTheme = () => { document.documentElement.dataset.theme = theme(); };
paintTheme();

// ---- the page -----------------------------------------------------------------------------------------------------
const openUrl = new URL(`./?sample=${encodeURIComponent(sampleId)}`, base).href;
const root = el('div', { className: 'ce', 'data-mode': viewMode });
const title = el('h1', { className: 'ce-title' }, 'Loading...');
if (hidden.has('title')) title.classList.add('ce-sr');
const open = el('a', { className: 'ce-open', href: openUrl, target: '_blank', rel: 'noopener' }, 'Open in CivilKit Calcs', el('span', { className: 'ce-sr' }, ' (opens in a new tab)'));
const head = el('header', { className: 'ce-head' }, title, open);
const note = el('div', { className: 'ce-note', role: 'note', hidden: true });
const form = el('form', { className: 'ce-form', 'aria-label': 'Inputs', noValidate: true });
form.addEventListener('submit', (e) => e.preventDefault());
const frame = el('iframe', { className: 'ce-frame', title: 'Results', sandbox: 'allow-scripts allow-popups allow-popups-to-escape-sandbox' } as never) as HTMLIFrameElement;
const status = el('p', { className: 'ce-status', role: 'status', 'aria-live': 'polite' });
const results = el('section', { className: 'ce-results', 'aria-label': 'Results' }, frame, status);
const body = el('div', { className: 'ce-body' });
if (viewMode !== 'results') body.append(form);
if (viewMode !== 'input') body.append(results);
root.append(head, note, body);
document.getElementById('app')?.replaceWith(root);
if (!root.isConnected) document.body.append(root);
document.querySelector('noscript')?.remove();

// ---- state --------------------------------------------------------------------------------------------------------
let sample: Sample | null = null;
let source = '';          // the sheet text as written (the edit box holds it)
let content = '';         // what the engine is given: Python left out unless allowed, number lines made inputs
let extraFiles: Record<string, string> = {};
let images = (html: string) => html;
let inputs: EmbedInput[] = [];
const values = new Map<string, string>();
let lastResults: Record<string, unknown> | null = null;
const settings = buildApiSettings(DEFAULT_CALCPAD_SETTINGS);

const api = (endpoint: string) => `${window.location.origin}/api/calcpad/${endpoint}`;
const overrides = () => Object.fromEntries(values);
const request = () => ({
    content, settings, outputFormat: 'html', forPrint: false, theme: theme(), enableUi: false, uiOverrides: overrides(),
    includeLineAnchors: false, hideErrorLines: true, write: false, extraFiles, sourceFilePath: `/${sampleId}.${isSheetText(content) ? 'sheet' : 'cpd'}`,
});

// ---- the form -----------------------------------------------------------------------------------------------------
let uid = 0;
function field(input: EmbedInput): HTMLElement {
    const id = `ce-in-${++uid}`;
    const name = el('span', { className: 'ce-name' }, input.name);
    const label = (forId?: string) => el(forId ? 'label' : 'span', forId ? { for: forId, className: 'ce-label' } : { className: 'ce-label' }, input.label || input.name, ' ', ...(input.label ? [name] : []));
    const wrap = el('div', { className: `ce-field ce-k-${input.kind}`, 'data-name': input.name });
    const changed = (v: string) => { values.set(input.name, v); schedule(); };
    if (input.kind === 'number') {
        const box = el('input', { id, type: 'text', inputMode: 'decimal', autocomplete: 'off', spellcheck: false, value: values.get(input.name) ?? input.value, className: 'ce-box', 'data-role': 'box' });
        const err = el('span', { className: 'ce-err', id: `${id}-err`, hidden: true });
        box.setAttribute('aria-describedby', `${id}-err`);
        const row = el('div', { className: 'ce-row' });
        let slider: HTMLInputElement | null = null;
        if (input.min !== undefined && input.max !== undefined) {
            slider = el('input', { type: 'range', min: String(input.min), max: String(input.max), step: String(input.step ?? 'any'), value: box.value, className: 'ce-slider', 'aria-label': `${input.label || input.name} slider`, 'data-role': 'slider' });
            slider.setAttribute('aria-valuetext', `${box.value} ${input.unit}`.trim());
            slider.addEventListener('input', () => {
                box.value = slider!.value;
                slider!.setAttribute('aria-valuetext', `${box.value} ${input.unit}`.trim());
                err.hidden = true; box.removeAttribute('aria-invalid');
                changed(slider!.value);
            });
            row.append(slider);
        }
        row.append(box);
        if (input.unit) row.append(el('span', { className: 'ce-unit' }, input.unit));
        const commit = () => {
            const c = checkValue(input, box.value);
            if (!c.ok) {
                err.textContent = c.error.replace(`${input.name}: `, '');
                err.hidden = false;
                box.setAttribute('aria-invalid', 'true');
                return;
            }
            err.hidden = true; box.removeAttribute('aria-invalid');
            if (slider) slider.value = c.value;
            if (values.get(input.name) !== c.value) changed(c.value);
        };
        box.addEventListener('input', () => { if (box.value.trim() !== '' && Number.isFinite(Number(box.value))) commit(); });
        box.addEventListener('change', commit);
        wrap.append(label(id), row, err);
    } else if (input.kind === 'select') {
        const sel = el('select', { id, className: 'ce-select', 'data-role': 'select' }, ...input.options!.map((o) => el('option', { value: o.value }, o.label)));
        sel.value = values.get(input.name) ?? input.value;
        sel.addEventListener('change', () => changed(sel.value));
        wrap.append(label(id), sel);
    } else if (input.kind === 'radio') {
        const set = el('fieldset', { className: 'ce-radios' }, el('legend', {}, label()));
        for (const o of input.options!) {
            const r = el('input', { type: 'radio', name: id, value: o.value, checked: (values.get(input.name) ?? input.value) === o.value });
            r.addEventListener('change', () => { if (r.checked) changed(o.value); });
            set.append(el('label', { className: 'ce-radio' }, r, ' ', o.label));
        }
        wrap.append(set);
    } else {
        const box = el('input', { id, type: 'checkbox', checked: (values.get(input.name) ?? input.value) === '1', 'data-role': 'check' });
        box.addEventListener('change', () => changed(box.checked ? '1' : '0'));
        wrap.append(el('label', { for: id, className: 'ce-check' }, box, ' ', label()));
    }
    return wrap;
}

/** Shows a value set from outside (the host) in the form. */
function showValue(input: EmbedInput, v: string): void {
    const wrap = form.querySelector<HTMLElement>(`.ce-field[data-name="${CSS.escape(input.name)}"]`);
    if (!wrap) return;
    const box = wrap.querySelector<HTMLInputElement>('[data-role=box]');
    const slider = wrap.querySelector<HTMLInputElement>('[data-role=slider]');
    if (box) { box.value = v; box.removeAttribute('aria-invalid'); wrap.querySelector<HTMLElement>('.ce-err')!.hidden = true; }
    if (slider) { slider.value = v; slider.setAttribute('aria-valuetext', `${v} ${input.unit}`.trim()); }
    const sel = wrap.querySelector<HTMLSelectElement>('select');
    if (sel) sel.value = v;
    for (const r of wrap.querySelectorAll<HTMLInputElement>('input[type=radio]')) r.checked = r.value === v;
    const check = wrap.querySelector<HTMLInputElement>('[data-role=check]');
    if (check) check.checked = v === '1';
}

function buildForm(): void {
    form.replaceChildren();
    if (!inputs.length) form.append(el('p', { className: 'ce-empty' }, 'This sheet has no inputs to change.'));
    for (const i of inputs) form.append(field(i));
}

// ---- calculation --------------------------------------------------------------------------------------------------
let timer = 0;
let running = false;
let again = false;
function schedule(delay = 90): void {
    window.clearTimeout(timer);
    timer = window.setTimeout(() => { void render(); }, delay);
}

/** The results page: links open in a new tab, and the page reports its height so the frame can fit it. */
function prepare(html: string): string {
    const extra = '<base target="_blank"><style>html,body{overflow:hidden!important}</style><script>(function(){'
        + 'var last=0;function h(){var b=document.body;if(!b)return;var r=Math.ceil(b.getBoundingClientRect().bottom+(parseFloat(getComputedStyle(b).marginBottom)||0)+window.scrollY);'
        + 'if(r!==last){last=r;parent.postMessage({type:"civilkit-results-height",height:r},"*")}}'
        + 'document.addEventListener("DOMContentLoaded",function(){h();new ResizeObserver(h).observe(document.body)});addEventListener("load",h)})()</script>';
    const i = html.search(/<head[^>]*>/i);
    if (i < 0) return extra + html;
    const end = html.indexOf('>', i) + 1;
    return html.slice(0, end) + extra + html.slice(end);
}

window.addEventListener('message', (e) => {
    if (e.source !== frame.contentWindow) return;
    const d = e.data as { type?: unknown; height?: unknown };
    if (d?.type === 'civilkit-results-height' && typeof d.height === 'number' && Number.isFinite(d.height)) {
        frame.style.height = `${Math.max(80, Math.min(d.height, 20000))}px`;
    }
});

async function render(): Promise<void> {
    if (running) { again = true; return; }
    running = true;
    results.setAttribute('aria-busy', 'true');
    status.textContent = 'Calculating...';
    try {
        const res = await fetch(api('convert'), { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(request()) });
        if (!res.ok) throw new Error(await res.text());
        const errors = JSON.parse(decodeURIComponent(res.headers.get('X-Calcpad-Errors') ?? '%5B%5D')) as unknown[];
        frame.srcdoc = prepare(images(await res.text()));
        status.textContent = errors.length ? `Calculated, with ${errors.length} error${errors.length === 1 ? '' : 's'} in the sheet.` : 'Calculated.';
        if (framed && !again) await postResults();
    } catch (err) {
        status.textContent = 'The calculation failed.';
        frame.srcdoc = `<p style="font:14px system-ui,sans-serif;color:#b00">The calculation failed: ${esc(String(err instanceof Error ? err.message : err)).slice(0, 400)}</p>`;
    } finally {
        running = false;
        results.removeAttribute('aria-busy');
        if (again) { again = false; schedule(0); }
    }
}

/** What the variables came to (the transport's values endpoint), to the host. */
async function postResults(): Promise<void> {
    const res = await fetch(api('values'), { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(request()) });
    if (!res.ok) return;
    const { variables } = (await res.json()) as { variables: { name: string; value: string; unit: string }[] };
    const out: Record<string, { value: number | string; unit: string; text: string }> = {};
    for (const v of variables) {
        const n = Number(v.value);
        const value = v.value.trim() !== '' && Number.isFinite(n) ? n : v.value;
        out[v.name] = { value, unit: v.unit ?? '', text: `${v.value}${v.unit ? ' ' + v.unit : ''}` };
    }
    lastResults = { type: 'civilkit-results', sample: sampleId, values: out };
    toHost(lastResults);
}

// ---- the host -----------------------------------------------------------------------------------------------------
window.addEventListener('message', (e) => {
    if (!framed || e.source !== window.parent) return;
    const d = e.data as { type?: unknown; values?: unknown };
    if (!d || typeof d !== 'object' || typeof d.type !== 'string') return;
    if (d.type === 'civilkit-get') { if (lastResults) toHost(lastResults); return; }
    if (d.type !== 'civilkit-set') return;
    const errors: string[] = [];
    if (!d.values || typeof d.values !== 'object' || Array.isArray(d.values)) {
        toHost({ type: 'civilkit-error', errors: ['values must be an object of input names'] });
        return;
    }
    let changed = false;
    for (const [name, raw] of Object.entries(d.values as Record<string, unknown>).slice(0, 200)) {
        const input = inputs.find((i) => i.name === name);
        if (!input) { errors.push(`${String(name).slice(0, 60)}: not an input of this sheet`); continue; }
        const c = checkValue(input, raw);
        if (!c.ok) { errors.push(c.error); continue; }
        if (values.get(name) !== c.value) { values.set(name, c.value); changed = true; }
        showValue(input, c.value);
    }
    if (errors.length) toHost({ type: 'civilkit-error', errors });
    if (changed) schedule(0);
    else if (!errors.length && lastResults) toHost(lastResults);
});

// The embed's own height, for a host that resizes the frame (embed.js).
let lastHeight = 0;
new ResizeObserver(() => {
    const h = Math.ceil(root.getBoundingClientRect().height);
    if (h !== lastHeight) { lastHeight = h; toHost({ type: 'civilkit-embed-height', height: h }); }
}).observe(root);

// ---- the sheet ----------------------------------------------------------------------------------------------------
async function useSource(text: string): Promise<void> {
    source = text;
    const py = pythonBlocks(text);
    let run = allowPython || !py.length ? text : withoutPython(text);
    const sheet = isSheetText(run);
    if (sheet) run = promoteNumberLines(run);
    content = run;
    note.replaceChildren();
    note.hidden = !(py.length && !allowPython);
    if (!note.hidden) {
        note.append(
            el('p', {}, `This sheet has ${py.length === 1 ? 'a Python block' : `${py.length} Python blocks`}. An embed does not run Python, so results that need it are missing here. `,
                el('a', { href: openUrl, target: '_blank', rel: 'noopener' }, 'Open it in CivilKit Calcs'), ' to run it.'),
            el('details', {}, el('summary', {}, 'Show the Python'), el('pre', {}, py.join('\n\n'))),
        );
    }
    const calcpad = sheet ? (await compileSheet(run)).text : run;
    const next = parseInputs(calcpad);
    for (const k of [...values.keys()]) if (!next.some((i) => i.name === k)) values.delete(k);
    inputs = next;
    buildForm();
}

function addEditor(): void {
    const area = el('textarea', { className: 'ce-editor', spellcheck: false, value: source, rows: 12, 'aria-label': 'Sheet text' });
    let t = 0;
    area.addEventListener('input', () => {
        window.clearTimeout(t);
        t = window.setTimeout(async () => { await useSource(area.value); schedule(0); }, 400);
    });
    root.append(el('details', { className: 'ce-edit' }, el('summary', {}, 'Edit the sheet'), area,
        el('p', { className: 'ce-hint' }, 'Changes stay in this page. Open in CivilKit Calcs for the full editor.')));
}

function fail(message: string): void {
    title.textContent = 'CivilKit Calcs';
    title.classList.remove('ce-sr');
    body.replaceChildren(el('p', { className: 'ce-missing' }, message, ' ', el('a', { href: new URL('./embed/', base).href, target: '_blank', rel: 'noopener' }, 'See the samples that can be embedded.')));
}

async function start(): Promise<void> {
    try {
        const m = await loadManifest();
        sample = m.samples.find((s) => s.id === sampleId) ?? null;
    } catch { /* fall through */ }
    if (!sample) { fail(`There is no sample "${sampleId}".`); return; }
    title.textContent = sample.title;
    document.title = `${sample.title}: CivilKit Calcs`;
    frame.title = `Results: ${sample.title}`;
    const src = await sampleSource(sample);
    extraFiles = src.extraFiles;
    images = src.images;
    await useSource(src.text);
    if (editMode) addEditor();
    toHost({ type: 'civilkit-ready', sample: sampleId, inputs: inputs.map(({ name, kind, label, unit, value, min, max, step, options }) => ({ name, kind, label, unit, value, min, max, step, options })) });
    await render();
}

if (themeParam === 'auto') dark.addEventListener('change', () => { paintTheme(); schedule(0); });
start().catch((err) => fail(`The sample could not be loaded (${String(err instanceof Error ? err.message : err).slice(0, 200)}).`));
