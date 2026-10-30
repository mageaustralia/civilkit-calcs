// The input form of a worksheet: one field per input (a box, with a slider when the input has bounds; a list; radio
// buttons; a checkbox). Shared by the embed (embed.ts) and the docked inputs of the app's Input mode (input-dock.ts),
// which give it the values and take the changes. Styles: form.css.
import { checkValue, type Checked, type EmbedInput } from './inputs';

/** The words of the form, so the app can pass translated ones (the embed keeps English). */
export interface FormText {
    /** The accessible name of a slider. */
    slider: (label: string) => string;
    /** A number outside the bounds. */
    range: (min: number, max: number) => string;
    /** Something that is not a number. */
    notNumber: () => string;
}

export const ENGLISH_TEXT: FormText = {
    slider: (label) => `${label} slider`,
    range: (min, max) => `must be from ${min} to ${max}`,
    notNumber: () => 'not a finite number',
};

export interface FieldOptions {
    /** The identity of an input: what data-name holds and what the values are keyed by (default: its name). */
    keyOf?: (input: EmbedInput) => string;
    /** The value an input shows, or undefined for its default. */
    value: (key: string) => string | undefined;
    /**
     * A value the person set. `live` is a slider being moved (more follow); otherwise the value is settled (a
     * released slider, a committed box, a list, a radio button, a checkbox).
     */
    changed: (input: EmbedInput, value: string, live: boolean) => void;
    /** Makes element ids unique between forms on one page. */
    idPrefix: string;
    text?: FormText;
    /** Checks a typed number (default checkValue). */
    check?: (input: EmbedInput, raw: unknown) => Checked;
}

type Props<K extends keyof HTMLElementTagNameMap> = Partial<HTMLElementTagNameMap[K]> & Record<string, unknown>;
export const el = <K extends keyof HTMLElementTagNameMap>(tag: K, props: Props<K> = {}, ...kids: (Node | string)[]) => {
    const e = document.createElement(tag);
    for (const [k, v] of Object.entries(props)) {
        if (k.startsWith('data-') || k.startsWith('aria-') || k === 'role' || k === 'for') e.setAttribute(k, String(v));
        else (e as unknown as Record<string, unknown>)[k] = v;
    }
    e.append(...kids);
    return e;
};

const uids = new Map<string, number>();

const decimals = (x: number): number => {
    const t = String(x);
    if (t.includes('e-')) return parseInt(t.split('e-')[1], 10) + (t.split('e-')[0].split('.')[1] ?? '').length;
    return (t.split('.')[1] ?? '').length;
};
/** A slider position as the box shows it: the decimals of the step and the minimum, no float noise (0.30000000000000004). */
function tidy(v: string, input: EmbedInput): string {
    const n = Number(v);
    if (!Number.isFinite(n) || input.step === undefined) return v;
    return String(parseFloat(n.toFixed(Math.max(decimals(input.step), decimals(input.min ?? 0)))));
}

/** The message of a refused number, in the form's words. */
function message(input: EmbedInput, c: Checked, text: FormText): string {
    if (c.ok) return '';
    if (input.min !== undefined && input.max !== undefined && /must be from/.test(c.error)) return text.range(input.min, input.max);
    if (/finite/.test(c.error)) return text.notNumber();
    return c.error.replace(`${input.name}: `, '');
}

/** One field of the form. */
export function buildField(input: EmbedInput, opts: FieldOptions): HTMLElement {
    const text = opts.text ?? ENGLISH_TEXT;
    const check = opts.check ?? checkValue;
    const key = opts.keyOf ? opts.keyOf(input) : input.name;
    const n = (uids.get(opts.idPrefix) ?? 0) + 1;
    uids.set(opts.idPrefix, n);
    const id = `${opts.idPrefix}-${n}`;
    const current = opts.value(key) ?? input.value;
    const name = el('span', { className: 'ce-name' }, input.name);
    const label = (forId?: string) => el(forId ? 'label' : 'span', forId ? { for: forId, className: 'ce-label' } : { className: 'ce-label' }, input.label || input.name, ' ', ...(input.label ? [name] : []));
    const wrap = el('div', { className: `ce-field ce-k-${input.kind}`, 'data-name': key });
    const changed = (v: string, live = false) => opts.changed(input, v, live);
    if (input.kind === 'number') {
        const box = el('input', { id, type: 'text', inputMode: 'decimal', autocomplete: 'off', spellcheck: false, value: current, className: 'ce-box', 'data-role': 'box' });
        const err = el('span', { className: 'ce-err', id: `${id}-err`, hidden: true });
        box.setAttribute('aria-describedby', `${id}-err`);
        const row = el('div', { className: 'ce-row' });
        let slider: HTMLInputElement | null = null;
        if (input.min !== undefined && input.max !== undefined) {
            slider = el('input', { type: 'range', min: String(input.min), max: String(input.max), step: String(input.step ?? 'any'), value: box.value, className: 'ce-slider', 'aria-label': text.slider(input.label || input.name), 'data-role': 'slider' });
            slider.setAttribute('aria-valuetext', `${box.value} ${input.unit}`.trim());
            const moved = (live: boolean) => {
                box.value = tidy(slider!.value, input);
                slider!.setAttribute('aria-valuetext', `${box.value} ${input.unit}`.trim());
                err.hidden = true; box.removeAttribute('aria-invalid');
                changed(box.value, live);
            };
            slider.addEventListener('input', () => moved(true));
            slider.addEventListener('change', () => moved(false));
            row.append(slider);
        }
        row.append(box);
        if (input.unit) row.append(el('span', { className: 'ce-unit' }, input.unit));
        const commit = (settled: boolean) => {
            const c = check(input, box.value);
            if (!c.ok) {
                err.textContent = message(input, c, text);
                err.hidden = false;
                box.setAttribute('aria-invalid', 'true');
                return;
            }
            err.hidden = true; box.removeAttribute('aria-invalid');
            if (slider) slider.value = c.value;
            if ((opts.value(key) ?? input.value) !== c.value) changed(c.value);
            else if (settled && box.value.trim() !== c.value) box.value = c.value;
        };
        box.addEventListener('input', () => { if (box.value.trim() !== '' && Number.isFinite(Number(box.value))) commit(false); });
        box.addEventListener('change', () => commit(true));
        box.addEventListener('keydown', (e) => { if (e.key === 'Enter') commit(true); });
        wrap.append(label(id), row, err);
    } else if (input.kind === 'select') {
        const sel = el('select', { id, className: 'ce-select', 'data-role': 'select' }, ...input.options!.map((o) => el('option', { value: o.value }, o.label)));
        sel.value = current;
        sel.addEventListener('change', () => changed(sel.value));
        wrap.append(label(id), sel);
    } else if (input.kind === 'radio') {
        const set = el('fieldset', { className: 'ce-radios' }, el('legend', {}, label()));
        for (const o of input.options!) {
            const r = el('input', { type: 'radio', name: id, value: o.value, checked: current === o.value });
            r.addEventListener('change', () => { if (r.checked) changed(o.value); });
            set.append(el('label', { className: 'ce-radio' }, r, ' ', o.label));
        }
        wrap.append(set);
    } else {
        const box = el('input', { id, type: 'checkbox', checked: current === '1', 'data-role': 'check' });
        box.addEventListener('change', () => changed(box.checked ? '1' : '0'));
        wrap.append(el('label', { for: id, className: 'ce-check' }, box, ' ', label()));
    }
    return wrap;
}

/** Shows a value set from outside (the embedding page, another control) in the field of `key`. */
export function showValue(root: ParentNode, key: string, input: EmbedInput, v: string): void {
    const wrap = root.querySelector<HTMLElement>(`.ce-field[data-name="${CSS.escape(key)}"]`);
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
