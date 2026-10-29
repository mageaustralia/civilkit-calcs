// calcpad-rs: the Samples browser. Worksheets copied from CalcpadCE's Examples (MIT; see
// public/samples/NOTICE.md) with the files they use, described by public/samples/manifest.json
// (made by tools/gen_samples.mjs). File > Samples, or the link on an empty editor, opens it:
// groups, search over title, description and codes, a design code filter, a rendered preview,
// and Open, which makes a new untitled copy and puts the sample's files into the library.

import { replaceCStyleOperators } from './c-style';
import { isSheetText } from './sheet';

/** A sheet keeps its own operators; a Calcpad worksheet gets the C-style ones converted. */
const opened = (t: string) => (isSheetText(t) ? t : replaceCStyleOperators(t));
import type { FileLibrary } from './library';
import type { TabsLike } from './web-files';

export interface SampleAsset { key: string; url: string }
export interface Sample {
    id: string;
    title: string;
    group: string;
    folder: string;
    file: string;
    upstream: string;
    description: string;
    codes: string[];
    ui: boolean;
    scripts: boolean;
    read: boolean;
    /** The worksheet has `#python` blocks: its preview shows the source and does not run them. */
    python?: boolean;
    assets: SampleAsset[];
}
interface Manifest { groups: string[]; samples: Sample[] }

const base = new URL(import.meta.env.BASE_URL || './', document.baseURI).href;
let manifest: Promise<Manifest> | null = null;

export function loadManifest(): Promise<Manifest> {
    manifest ??= (async () => {
        const r = await fetch(`${base}samples/manifest.json`);
        if (!r.ok) throw new Error(`samples/manifest.json: ${r.status}`);
        const m = (await r.json()) as Manifest;
        // Sheet syntax samples (our addition): their own folder and manifest, listed first.
        try {
            const sr = await fetch(`${base}sheet-samples/manifest.json`);
            if (sr.ok) {
                const sm = (await sr.json()) as Manifest;
                return { groups: [...sm.groups, ...m.groups], samples: [...sm.samples, ...m.samples] };
            }
        } catch { /* the sheet samples are optional */ }
        return m;
    })();
    return manifest;
}

/** Samples matching every word of `query` (title, description, codes, folder) and `code`. */
export function filterSamples(all: Sample[], group: string, query: string, code: string): Sample[] {
    const words = query.toLowerCase().split(/\s+/).filter(Boolean);
    return all.filter((s) => {
        if (group && s.group !== group) return false;
        if (code && !s.codes.includes(code)) return false;
        const text = `${s.title} ${s.description} ${s.codes.join(' ')} ${s.folder}`.toLowerCase();
        return words.every((w) => text.includes(w));
    });
}

/** The library path a reference from an untitled sheet resolves to (as the engine does). */
export function libraryKey(ref: string): string {
    const parts: string[] = [];
    for (const p of ref.replace(/\\/g, '/').split('/')) {
        if (p === '' || p === '.') continue;
        if (p === '..') parts.pop(); else parts.push(p);
    }
    return '/' + parts.join('/');
}

async function bytes(url: string): Promise<Uint8Array> {
    const r = await fetch(new URL(url, base).href);
    if (!r.ok) throw new Error(`${url}: ${r.status}`);
    return new Uint8Array(await r.arrayBuffer());
}

function toBase64(b: Uint8Array): string {
    let s = '';
    for (let i = 0; i < b.length; i += 0x8000) s += String.fromCharCode(...b.subarray(i, i + 0x8000));
    return btoa(s);
}

const IMG_SRC = /(<img\b[^>]*?\bsrc\s*=\s*)(["'])([^"']+)\2/gi;

/** Points relative <img src> at the sample's static copies, for the preview. */
function staticImages(html: string, sample: Sample): string {
    const byKey = new Map(sample.assets.map((a) => [a.key, new URL(a.url, base).href]));
    return html.replace(IMG_SRC, (m, pre: string, q: string, src: string) => {
        if (/^[a-z]+:|^\/\//i.test(src)) return m;
        const url = byKey.get(libraryKey(src));
        return url ? `${pre}${q}${url}${q}` : m;
    });
}

/**
 * A sample's text as it opens and the data files it reads (#include, #read), for a render outside the editor (the embed,
 * src/embed/embed.ts), with the images it shows pointed at their static copies by `staticImages`.
 */
export async function sampleSource(s: Sample): Promise<{ text: string; extraFiles: Record<string, string>; images: (html: string) => string }> {
    const text = opened(new TextDecoder().decode(await bytes(s.file)).replace(/^\uFEFF/, ''));
    const extraFiles: Record<string, string> = {};
    for (const a of s.assets) {
        if (/\.(cpd|txt|csv|xlsx|xlsm)$/i.test(a.key)) extraFiles[a.key] = toBase64(await bytes(a.url));
    }
    return { text, extraFiles, images: (html) => staticImages(html, s) };
}

export interface SamplesHost {
    library: FileLibrary;
    tabs: () => TabsLike;
    log: (level: 'info' | 'warn' | 'error', message: string) => void;
}

export class SamplesBrowser {
    private root: HTMLDivElement | null = null;
    private selected: Sample | null = null;
    private previewSeq = 0;
    private readonly onKey = (e: KeyboardEvent) => { if (e.key === 'Escape') this.close(); };

    constructor(private readonly host: SamplesHost) {}

    async open(): Promise<void> {
        if (this.root) return;
        let m: Manifest;
        try {
            m = await loadManifest();
        } catch (e) {
            this.host.log('error', `The samples could not be loaded: ${e instanceof Error ? e.message : String(e)}`);
            return;
        }
        this.render(m);
    }

    close(): void {
        document.removeEventListener('keydown', this.onKey, true);
        this.root?.remove();
        this.root = null;
        this.selected = null;
    }

    /**
     * Opens the sample named by `key` (the `?sample=` address parameter): its id, with or without the leading
     * `template-`, its title, or its file name, ignoring case. Returns false when nothing matches.
     */
    async openByKey(key: string): Promise<boolean> {
        const k = key.trim().toLowerCase();
        if (!k) return false;
        const m = await loadManifest();
        const same = (v: string) => v.toLowerCase() === k;
        const s = m.samples.find((x) => same(x.id) || same(x.id.replace(/^template-/, '')) || same(x.title)
            || same(x.file.replace(/^.*\//, '')) || same(x.file.replace(/^.*\//, '').replace(/\.[^.]+$/, '')));
        if (!s) return false;
        await this.openSample(s);
        return true;
    }

    /** Opens `sample` as a new untitled worksheet and puts its files into the library. */
    async openSample(sample: Sample): Promise<void> {
        const text = opened(new TextDecoder().decode(await bytes(sample.file)).replace(/^﻿/, ''));
        for (const a of sample.assets) await this.host.library.put(a.key, await bytes(a.url));
        this.host.tabs().newUntitled(text, sample.title);
        this.host.log('info', `Opened the sample "${sample.title}" as a new worksheet${sample.assets.length ? `; its ${sample.assets.length} file(s) are in the library` : ''}.`);
    }

    private render(m: Manifest): void {
        injectStyle();
        const root = document.createElement('div');
        root.className = 'cprs-samples-backdrop';
        root.innerHTML = `
<div class="cprs-samples" role="dialog" aria-label="Samples">
  <div class="cprs-s-head"><b>Samples</b><span class="cprs-s-sub">Sheet samples and templates, and the CalcpadCE examples (MIT). Open makes a copy; the originals are never edited.</span>
    <button class="cprs-s-close" title="Close (Esc)" aria-label="Close">×</button></div>
  <div class="cprs-s-body">
    <div class="cprs-s-groups"></div>
    <div class="cprs-s-listcol">
      <div class="cprs-s-filters"><input class="cprs-s-search" type="search" placeholder="Search title, description, codes">
        <select class="cprs-s-code"><option value="">All design codes</option></select></div>
      <div class="cprs-s-count"></div>
      <div class="cprs-s-list" role="listbox"></div>
    </div>
    <div class="cprs-s-previewcol">
      <div class="cprs-s-meta"></div>
      <iframe class="cprs-s-preview" sandbox="allow-scripts" title="Sample preview"></iframe>
      <div class="cprs-s-actions"><button class="cprs-s-open" disabled>Open</button></div>
    </div>
  </div>
</div>`;
        document.body.appendChild(root);
        this.root = root;
        const $ = <T extends Element>(sel: string) => root.querySelector(sel) as T;
        const search = $<HTMLInputElement>('.cprs-s-search');
        const codeSel = $<HTMLSelectElement>('.cprs-s-code');
        const list = $<HTMLDivElement>('.cprs-s-list');
        const groups = $<HTMLDivElement>('.cprs-s-groups');
        const count = $<HTMLDivElement>('.cprs-s-count');
        let group = '';
        for (const c of [...new Set(m.samples.flatMap((s) => s.codes))].sort()) {
            const o = document.createElement('option');
            o.value = c;
            o.textContent = c;
            codeSel.appendChild(o);
        }
        const groupButtons: HTMLButtonElement[] = [];
        for (const g of ['', ...m.groups]) {
            const b = document.createElement('button');
            b.className = 'cprs-s-group';
            b.dataset.group = g;
            b.textContent = `${g || 'All'} (${g ? m.samples.filter((s) => s.group === g).length : m.samples.length})`;
            b.addEventListener('click', () => { group = g; refresh(); });
            groups.appendChild(b);
            groupButtons.push(b);
        }
        const refresh = () => {
            for (const b of groupButtons) b.classList.toggle('active', b.dataset.group === group);
            const shown = filterSamples(m.samples, group, search.value, codeSel.value);
            count.textContent = `${shown.length} of ${m.samples.length}`;
            list.replaceChildren(...shown.map((s) => {
                const item = document.createElement('div');
                item.className = 'cprs-s-item';
                item.setAttribute('role', 'option');
                item.dataset.sampleId = s.id;
                const t = document.createElement('div');
                t.className = 'cprs-s-title';
                t.textContent = s.title;
                const d = document.createElement('div');
                d.className = 'cprs-s-desc';
                d.textContent = s.description;
                const tags = document.createElement('div');
                tags.className = 'cprs-s-tags';
                for (const tag of [...s.codes.filter((c) => /-|^[A-Z]+ \d+$/.test(c) && !/^EN \d{4}$/.test(c)), ...(s.ui ? ['input form'] : []), ...(s.scripts ? ['scripts'] : []), ...(s.read ? ['#read'] : []), ...(s.python ? ['python'] : [])]) {
                    const span = document.createElement('span');
                    span.textContent = tag;
                    tags.appendChild(span);
                }
                item.append(t, d, tags);
                item.classList.toggle('active', this.selected?.id === s.id);
                item.addEventListener('click', () => this.select(s));
                item.addEventListener('dblclick', () => void this.openAndClose(s));
                return item;
            }));
        };
        search.addEventListener('input', refresh);
        codeSel.addEventListener('change', refresh);
        $<HTMLButtonElement>('.cprs-s-close').addEventListener('click', () => this.close());
        root.addEventListener('mousedown', (e) => { if (e.target === root) this.close(); });
        document.addEventListener('keydown', this.onKey, true);
        $<HTMLButtonElement>('.cprs-s-open').addEventListener('click', () => { if (this.selected) void this.openAndClose(this.selected); });
        refresh();
        search.focus();
    }

    private async openAndClose(s: Sample): Promise<void> {
        this.close();
        try {
            await this.openSample(s);
        } catch (e) {
            this.host.log('error', `The sample could not be opened: ${e instanceof Error ? e.message : String(e)}`);
        }
    }

    private async select(s: Sample): Promise<void> {
        const root = this.root;
        if (!root) return;
        this.selected = s;
        for (const el of Array.from(root.querySelectorAll('.cprs-s-item'))) {
            el.classList.toggle('active', (el as HTMLElement).dataset.sampleId === s.id);
        }
        (root.querySelector('.cprs-s-open') as HTMLButtonElement).disabled = false;
        const meta = root.querySelector('.cprs-s-meta') as HTMLDivElement;
        meta.textContent = `${s.group} / ${s.folder}${s.codes.length ? ' | ' + s.codes.filter((c) => /-/.test(c) || !/^EN/.test(c)).join(', ') : ''}`;
        const frame = root.querySelector('.cprs-s-preview') as HTMLIFrameElement;
        const seq = ++this.previewSeq;
        frame.srcdoc = '<p style="font:13px sans-serif;color:#666">Rendering...</p>';
        try {
            const text = opened(new TextDecoder().decode(await bytes(s.file)).replace(/^﻿/, ''));
            if (s.python) {
                // Browsing must not run code: show the source; opening the sample asks before it runs.
                const esc = text.replace(/&/g, '&amp;').replace(/</g, '&lt;');
                if (seq === this.previewSeq && this.root) frame.srcdoc = '<p style="font:13px sans-serif;color:#555">This worksheet runs Python when it is opened and calculated. Its source:</p>'
                    + `<pre style="font:12px/1.4 monospace;white-space:pre-wrap">${esc}</pre>`;
                return;
            }
            const extraFiles: Record<string, string> = {};
            for (const a of s.assets) {
                if (/\.(cpd|txt|csv|xlsx|xlsm)$/i.test(a.key)) extraFiles[a.key] = toBase64(await bytes(a.url));
            }
            const res = await fetch(`${window.location.origin}/api/calcpad/convert`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ content: text, extraFiles, sourceFilePath: `/${s.id}.cpd`, includeLineAnchors: false }),
            });
            const html = await res.text();
            if (seq === this.previewSeq && this.root) frame.srcdoc = staticImages(html, s);
        } catch (e) {
            if (seq === this.previewSeq) frame.srcdoc = `<p style="font:13px sans-serif;color:#900">The preview failed: ${String(e).replace(/</g, '&lt;')}</p>`;
        }
    }
}

/** A "Browse samples" link over an empty editor. */
export function installEmptyEditorLink(tabs: () => TabsLike | undefined, openSamples: () => void): void {
    injectStyle();
    const link = document.createElement('button');
    link.className = 'cprs-samples-link';
    link.textContent = 'Start from a sample...';
    link.addEventListener('click', openSamples);
    link.hidden = true;
    document.body.appendChild(link);
    const place = () => {
        const empty = (tabs()?.activeModel?.getValue() ?? 'x') === '';
        const pane = document.querySelector('.editor-pane') as HTMLElement | null;
        if (!empty || !pane || pane.offsetParent === null) {
            link.hidden = true;
            return;
        }
        const r = pane.getBoundingClientRect();
        link.style.left = `${Math.round(r.left + r.width / 2)}px`;
        link.style.top = `${Math.round(r.top + Math.min(160, r.height / 3))}px`;
        link.hidden = false;
    };
    setInterval(place, 500);
}

let styled = false;
function injectStyle(): void {
    if (styled) return;
    styled = true;
    const style = document.createElement('style');
    style.textContent = `
.cprs-samples-backdrop{position:fixed;inset:0;z-index:2000;background:rgba(0,0,0,.35);display:flex;align-items:center;justify-content:center}
.cprs-samples{width:min(1200px,94vw);height:min(760px,90vh);display:flex;flex-direction:column;background:var(--vscode-editor-background,#fff);
  color:var(--vscode-editor-foreground,#222);border:1px solid var(--vscode-panel-border,#ccc);box-shadow:0 8px 32px rgba(0,0,0,.3);font:13px system-ui,sans-serif}
.cprs-s-head{display:flex;align-items:center;gap:12px;padding:8px 12px;border-bottom:1px solid var(--vscode-panel-border,#ddd)}
.cprs-s-sub{opacity:.65;font-size:12px;flex:1}
.cprs-s-close{border:0;background:none;font-size:20px;cursor:pointer;color:inherit}
.cprs-s-body{flex:1;display:flex;min-height:0}
.cprs-s-groups{width:170px;overflow:auto;border-right:1px solid var(--vscode-panel-border,#ddd);padding:6px 0}
.cprs-s-group{display:block;width:100%;text-align:left;border:0;background:none;color:inherit;padding:5px 12px;cursor:pointer;font:inherit}
.cprs-s-group.active,.cprs-s-item.active{background:var(--vscode-list-activeSelectionBackground,#0060c0);color:var(--vscode-list-activeSelectionForeground,#fff)}
.cprs-s-listcol{width:360px;display:flex;flex-direction:column;border-right:1px solid var(--vscode-panel-border,#ddd);min-height:0}
.cprs-s-filters{display:flex;flex-direction:column;gap:6px;padding:8px}
.cprs-s-count{padding:0 10px 4px;opacity:.65;font-size:12px}
.cprs-s-list{flex:1;overflow:auto}
.cprs-s-item{padding:7px 10px;border-bottom:1px solid var(--vscode-panel-border,#eee);cursor:pointer}
.cprs-s-title{font-weight:600}
.cprs-s-desc{font-size:12px;opacity:.8;margin-top:2px;display:-webkit-box;-webkit-line-clamp:2;-webkit-box-orient:vertical;overflow:hidden}
.cprs-s-tags span{display:inline-block;font-size:10.5px;margin:3px 4px 0 0;padding:0 5px;border-radius:3px;background:rgba(0,96,192,.12)}
.cprs-s-previewcol{flex:1;display:flex;flex-direction:column;min-width:0}
.cprs-s-meta{padding:8px 10px;font-size:12px;opacity:.75}
.cprs-s-preview{flex:1;border:0;border-top:1px solid var(--vscode-panel-border,#eee);background:#fff}
.cprs-s-actions{padding:8px 10px;text-align:right;border-top:1px solid var(--vscode-panel-border,#ddd)}
.cprs-s-open{padding:5px 18px;cursor:pointer}
.cprs-samples-link{position:fixed;z-index:900;transform:translateX(-50%);padding:8px 16px;border:1px solid var(--vscode-button-background,#0e639c);
  background:var(--vscode-editor-background,#fff);color:var(--vscode-textLink-foreground,#0e639c);cursor:pointer;border-radius:4px;font:13px system-ui,sans-serif}`;
    document.head.appendChild(style);
}
