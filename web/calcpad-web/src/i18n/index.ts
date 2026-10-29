// The app's translator: a small in-house t() (docs/i18n-plan.md, section 2).
//
//   t('key', { name: 'x' })     plain text; {name} placeholders
//   tn('key', 3, { name: 'x' }) plural: picks key_one / key_few / key_many / key_other with Intl.PluralRules
//   tHtml('key', params)        catalogue markup; the parameters are escaped, the catalogue is trusted
//   td('key', params, 'English') a key built at run time (ribbon, setting definitions): the English is the fallback
//
// English is bundled (en.json) so the default language has no flash and no network dependency. Every other
// language is a JSON file in public/i18n/, fetched on demand and left out of the install precache (pwa-plugin.ts).
// A key missing from a language shows its English, so a language ships at any coverage. The catalogue is held in a
// shallowRef, so a template that calls t() re-renders when the language changes.
import { ref, shallowRef } from 'vue';
import type { Ref } from 'vue';
import en from './en.json';

export type Key = keyof typeof en;
type PluralKey<K extends string> = K extends `${infer B}_other` ? B : never;
/** The base of every `_other` key: what tn() takes. */
export type PluralBase = PluralKey<Key>;
export type Params = Record<string, string | number>;

export interface Language { code: string; native: string; english: string }
/** The languages a person can pick, in picker order. `xx` (a pseudo-language for QA) is reachable only through ?lang=xx. */
export const LANGUAGES: readonly Language[] = [
    { code: 'en', native: 'English', english: 'English' },
    { code: 'es-ES', native: 'Español (España)', english: 'Spanish (Spain)' },
    { code: 'pt-BR', native: 'Português (Brasil)', english: 'Portuguese (Brazil)' },
    { code: 'fr', native: 'Français', english: 'French' },
    { code: 'de', native: 'Deutsch', english: 'German' },
    { code: 'zh-Hans', native: '简体中文', english: 'Chinese (Simplified)' },
];
export const PSEUDO = 'xx';
const STORAGE_KEY = 'calcpad-language';

const source = en as Record<string, string>;
const catalogue = shallowRef<Record<string, string>>({});
const current = ref('en');
const loaded = new Map<string, Record<string, string>>();
const listeners = new Set<(lang: string) => void>();

/** The active language code (reactive). */
export const language: Readonly<Ref<string>> = current;
export const getLanguage = (): string => current.value;
export function onLanguageChanged(cb: (lang: string) => void): () => void {
    listeners.add(cb);
    return () => { listeners.delete(cb); };
}

/** A code or browser tag to one of our codes: 'es-MX' to 'es-ES', 'pt' to 'pt-BR', 'zh-CN' to 'zh-Hans'. Traditional Chinese is not supported (null). */
export function matchLanguage(tag: string | null | undefined, allowPseudo = false): string | null {
    if (!tag) return null;
    const t = tag.trim().toLowerCase().replace(/_/g, '-');
    if (allowPseudo && t === PSEUDO) return PSEUDO;
    const primary = t.split('-')[0];
    switch (primary) {
        case 'en': return 'en';
        case 'es': return 'es-ES';
        case 'pt': return 'pt-BR';
        case 'fr': return 'fr';
        case 'de': return 'de';
        case 'zh': return /(^|-)(hant|tw|hk|mo)(-|$)/.test(t) ? null : 'zh-Hans';
        default: return null;
    }
}

/** The first of the browser's languages we can serve; null when the first language it lists is English or none matches. */
export function suggestedLanguage(tags: readonly string[] = navigator.languages?.length ? navigator.languages : [navigator.language]): string | null {
    for (const tag of tags) {
        const primary = (tag ?? '').toLowerCase().split('-')[0];
        if (primary === 'en') return null;
        const m = matchLanguage(tag);
        if (m) return m;
    }
    return null;
}

/** The language named by `?lang=` on this page view, if any. It is never stored. */
export function urlLanguage(search: string = location.search): string | null {
    return matchLanguage(new URLSearchParams(search).get('lang'), true);
}
export function storedLanguage(): string | null {
    try {
        const v = localStorage.getItem(STORAGE_KEY);
        return v && LANGUAGES.some(l => l.code === v) ? v : null;
    } catch { return null; }
}
function store(lang: string): void {
    try { localStorage.setItem(STORAGE_KEY, lang); } catch { /* storage blocked */ }
}

function catalogueUrl(lang: string): string {
    const v = import.meta.env?.VITE_APP_VERSION ?? '';
    return new URL(`i18n/${lang}.json${v ? `?v=${encodeURIComponent(v)}` : ''}`, document.baseURI).href;
}

/** Fetches a language's catalogue (once). Resolves null when it cannot be had: the caller keeps English. */
export async function loadCatalogue(lang: string): Promise<Record<string, string> | null> {
    if (lang === 'en' || lang === PSEUDO) return {};
    const hit = loaded.get(lang);
    if (hit) return hit;
    try {
        const res = await fetch(catalogueUrl(lang));
        if (!res.ok) return null;
        const body = (await res.json()) as Record<string, unknown>;
        const out: Record<string, string> = {};
        for (const [k, v] of Object.entries(body)) if (k !== '_meta' && typeof v === 'string') out[k] = v;
        loaded.set(lang, out);
        return out;
    } catch {
        return null;
    }
}

/** Switches the language. `persist` stores the choice (the picker and the first-visit prompt do; ?lang= does not). */
export async function setLanguage(lang: string, opts: { persist?: boolean } = {}): Promise<boolean> {
    const next = lang === PSEUDO || LANGUAGES.some(l => l.code === lang) ? lang : 'en';
    const cat = await loadCatalogue(next);
    const ok = cat !== null;
    const code = ok ? next : 'en';
    catalogue.value = cat ?? {};
    current.value = code;
    document.documentElement.lang = code === PSEUDO ? 'en' : code;
    if (opts.persist && ok && code !== PSEUDO) store(code);
    for (const cb of listeners) cb(code);
    return ok;
}

/** First run: ?lang= wins (not stored), then the stored choice, else English. The prompt (languagePrompt) handles the browser's language. */
export async function initLanguage(): Promise<void> {
    const wanted = urlLanguage() ?? storedLanguage() ?? 'en';
    if (wanted !== 'en') await setLanguage(wanted);
    else document.documentElement.lang = 'en';
}

/** Whether to ask the person to switch: no ?lang=, no stored choice, and a browser language we can serve. */
export function languagePrompt(): string | null {
    if (urlLanguage() || storedLanguage()) return null;
    return suggestedLanguage();
}
/** Remembers that the person chose (English included), so the prompt does not return. */
export function rememberLanguage(lang: string): void { store(lang); }

const ACCENTED: Record<string, string> = { a: 'à', e: 'é', i: 'ï', o: 'ö', u: 'ü', c: 'ç', n: 'ñ', y: 'ý', A: 'Å', E: 'É', I: 'Ï', O: 'Ö', U: 'Ü' };
/** The QA pseudo-language: accented letters, about a third longer, in brackets. Placeholders and tags are left alone. */
function pseudo(s: string): string {
    const body = s.replace(/(\{\w+\}|<[^>]+>|&\w+;)|([A-Za-z])/g, (_m, keep: string, ch: string) => keep ?? ACCENTED[ch] ?? ch);
    const pad = ' ' + '·'.repeat(Math.ceil(s.length * 0.35));
    return `[!! ${body}${pad} !!]`;
}

function fill(s: string, params?: Params, escape = false): string {
    if (!params) return s;
    return s.replace(/\{(\w+)\}/g, (m, k: string) => {
        if (!(k in params)) return m;
        const v = String(params[k]);
        return escape ? escapeHtml(v) : v;
    });
}
export function escapeHtml(s: string): string {
    return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

function pick(key: string, fallback?: string): string {
    const s = catalogue.value[key] ?? source[key] ?? fallback ?? key.slice(key.lastIndexOf('.') + 1);
    return current.value === PSEUDO ? pseudo(s) : s;
}

/** Plain text for a key. */
export function t(key: Key, params?: Params): string {
    return fill(pick(key), params);
}
/** A key built at run time, or one used by the shared frontend package: the English is the fallback. */
export function td(key: string, params?: Params, fallback?: string): string {
    return fill(pick(key, fallback), params);
}
/** Catalogue markup (a few fixed tags); the parameters are escaped. */
export function tHtml(key: Key, params?: Params): string {
    return fill(pick(key), params, true);
}
/** A plural: key_one, key_few, key_many or key_other, chosen by the language's own rules; `{count}` is filled in. */
export function tn(key: PluralBase, n: number, params?: Params): string {
    const all: Params = { count: n, ...params };
    const lang = current.value === PSEUDO ? 'en' : current.value;
    const own = catalogue.value;
    const has = (k: string) => k in own;
    const cat = new Intl.PluralRules(lang).select(n);
    for (const c of [cat, 'other']) if (has(`${key}_${c}`)) return fill(pick(`${key}_${c}`), all);
    // Not in this language: the English text, with English plural rules.
    const ecat = new Intl.PluralRules('en').select(n);
    const k = `${key}_${ecat}` in source ? `${key}_${ecat}` : `${key}_other`;
    const s = source[k] ?? key;
    return fill(current.value === PSEUDO ? pseudo(s) : s, all);
}

/** The active language's own text for a key, or undefined (English and the pseudo-language have none). */
export function translated(key: string): string | undefined {
    return catalogue.value[key];
}
/** The English text of a key. */
export function english(key: string): string | undefined {
    return source[key];
}

/** A string of another language, for text shown before the switch (the first-visit prompt). Falls back to English. */
export async function tIn(lang: string, key: Key, params?: Params): Promise<string> {
    const cat = await loadCatalogue(lang);
    return fill(cat?.[key] ?? source[key] ?? key, params);
}
