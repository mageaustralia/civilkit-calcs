// The shared frontend package serves other hosts, so it cannot import the web app's translator. A host installs one
// with setTranslator(); until it does, t() returns the English given as its third argument, so a host that never
// installs one sees exactly the text this package always showed.
export type Translator = (key: string, params?: Record<string, string | number>, fallback?: string) => string;

let translator: Translator | null = null;

export function setTranslator(fn: Translator | null): void {
    translator = fn;
}

function fill(s: string, params?: Record<string, string | number>): string {
    return params ? s.replace(/\{(\w+)\}/g, (m, k: string) => (k in params ? String(params[k]) : m)) : s;
}

/** Text for `key`; `fallback` is the English the package ships. */
export function t(key: string, params?: Record<string, string | number>, fallback?: string): string {
    if (translator) return translator(key, params, fallback);
    return fill(fallback ?? key.slice(key.lastIndexOf('.') + 1), params);
}
