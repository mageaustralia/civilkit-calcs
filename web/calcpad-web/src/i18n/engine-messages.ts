// The engine's messages in the reader's language, swapped at the transport edge (docs/i18n-plan.md, section 2.6).
//
// The engine writes English. Asked with `"messageCodes": true`, it also records each message from its catalogue with a code and
// the arguments, and the wasm layer wraps that text in the page as `<ck-msg code="..." args="...">English</ck-msg>` and adds
// `code` and `args` to its error list. This module is the other half, the same way number-format.ts is for numbers:
//   - English (or a message the active language has no text for): the wrapper is removed, so the page is exactly the one the
//     engine wrote before codes existed;
//   - another language: the text inside the wrapper is replaced by the catalogue entry `msg.<code>` with the arguments filled in.
// Nothing else in the page is touched. An argument that is itself a coded message (the "Error in ..." frame holds the message it
// frames) is rendered the same way, in English when its own entry is missing.
import { english, getLanguage, translated } from './index';

type Arg = string | { code: string; args: Arg[] };

/** The request body with the engine asked to record its coded messages. */
export function withMessageCodes(body: string): string {
    try {
        const req = JSON.parse(body) as Record<string, unknown>;
        if (req && typeof req === 'object') { req.messageCodes = true; return JSON.stringify(req); }
    } catch { /* leave a body we cannot read as it is */ }
    return body;
}

const decode = (s: string): string => s.replace(/&quot;/g, '"').replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/&amp;/g, '&');

function fill(template: string, args: Arg[], lang: boolean): string {
    return template.replace(/\{(\d+)\}/g, (m, i: string) => {
        const a = args[Number(i)];
        if (a === undefined) return m;
        return typeof a === 'string' ? a : (render(a.code, a.args, lang) ?? render(a.code, a.args, false) ?? '');
    });
}
/** The message `code` with its arguments: in the active language when `lang`, else in English; null when there is no such entry. */
function render(code: string, args: Arg[], lang: boolean): string | null {
    const template = lang ? translated(`msg.${code}`) : english(`msg.${code}`);
    return template === undefined ? null : fill(template, args, lang);
}

/** The page with each coded message in the active language; English removes the wrappers and nothing else. */
export function showMessages(html: string): string {
    if (!html.includes('<ck-msg')) return html;
    const active = getLanguage() !== 'en';
    return html.replace(/<ck-msg code="([^"]*)" args="([^"]*)">([\s\S]*?)<\/ck-msg>/g, (_m, code: string, args: string, text: string) => {
        if (!active) return text;
        try {
            return render(code, JSON.parse(decode(args)) as Arg[], true) ?? text;
        } catch {
            return text;
        }
    });
}

interface CodedError { message?: string; code?: string; args?: Arg[] }
/** The engine's error list in the active language (the entries keep their `code` and `args`). */
export function showErrors(errors: CodedError[]): CodedError[] {
    if (getLanguage() === 'en') return errors;
    return errors.map((e) => {
        if (!e.code) return e;
        const m = render(e.code, e.args ?? [], true);
        return m === null ? e : { ...e, message: m };
    });
}

/** The engine's answer (the JSON string of calcpad-wasm's convert()) with its page and errors in the active language. */
export function showEngineMessages(answer: string): string {
    if (!answer.includes('<ck-msg') && !answer.includes('"code"')) return answer;
    try {
        const out = JSON.parse(answer) as { html?: string; errors?: CodedError[] };
        if (typeof out.html === 'string') out.html = showMessages(out.html);
        if (Array.isArray(out.errors)) out.errors = showErrors(out.errors);
        return JSON.stringify(out);
    } catch {
        return answer;
    }
}
