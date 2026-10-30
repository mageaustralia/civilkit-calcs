// Docked inputs (book: Templates, "Docked inputs"): in Input mode every input of the sheet in one panel of the host page,
// beside or above the results, so a slider can be dragged with the plot it moves in view. The panel lives outside the
// results frame, so a re-render that swaps the frame never interrupts a drag. Its fields are the embed's form
// (embed/form.ts); the list of inputs is read from the rendered form itself (the engine's data-ui-* attributes are
// exact through includes, macros and loops), with the labels and units of the sheet's own input lines.
import { buildField, el, showValue, type FormText } from './embed/form';
import { checkValue, parseInputs, plain, type EmbedInput } from './embed/inputs';
import { td } from './i18n';

export interface DockInput extends EmbedInput {
    /** The control's identity in the rendered form (`data-ui-var`, e.g. `L:1`): what the entered values are keyed by. */
    key: string;
}

/**
 * The inputs the dock can show from a rendered input form: the boxes that hold a number (not a value with its unit, not an
 * expression), lists, radio buttons and checkboxes. Data grids and the hand-made HTML controls stay in the document.
 * `source` is the Calcpad text (a sheet compiled), read for the labels and units.
 */
export function dockInputsFromHtml(html: string, source: string): DockInput[] {
    const doc = new DOMParser().parseFromString(html, 'text/html');
    const declared = new Map(parseInputs(source).map((i) => [i.name, i]));
    const out: DockInput[] = [];
    const seen = new Set<string>();
    const num = (e: Element, a: string) => { const v = e.getAttribute(a); const n = v == null || v === '' ? NaN : Number(v); return Number.isFinite(n) ? n : undefined; };
    for (const c of doc.querySelectorAll<HTMLElement>('.calcpad-ui-input, select.calcpad-ui-dropdown, .calcpad-ui-radio, input.calcpad-ui-checkbox')) {
        const key = c.getAttribute('data-ui-var');
        if (!key || seen.has(key)) continue;
        const name = key.split(':')[0];
        const d = declared.get(name);
        // The paragraph's own text when the line is not one of the sheet's (an included file): the name is the label.
        const label = d?.label ?? '';
        let input: DockInput | null = null;
        if (c.classList.contains('calcpad-ui-input')) {
            if (c.getAttribute('data-ui-allow-expression') === '1' || c.getAttribute('data-ui-force-units') === '0') continue;
            const value = (c as HTMLInputElement).getAttribute('value') ?? '';
            if (!/^[-+−]?(\d+\.?\d*|\.\d+)$/.test(value.trim())) continue;
            input = { key, name, kind: 'number', label, value: value.trim(), unit: d?.kind === 'number' ? d.unit : '' };
            const lo = num(c, 'data-ui-min'), hi = num(c, 'data-ui-max'), step = num(c, 'data-ui-step');
            if (lo !== undefined && hi !== undefined && lo <= hi) { input.min = lo; input.max = hi; }
            // Without a step the slider takes the one the document's own slider takes (ui-preview-script.html).
            if (step !== undefined && step > 0) input.step = step;
            else if (input.min !== undefined && input.max !== undefined) input.step = niceStep(input.min, input.max);
        } else if (c.tagName === 'SELECT') {
            const opts = [...(c as HTMLSelectElement).options].map((o) => ({ label: plain(o.textContent ?? ''), value: o.getAttribute('value') ?? '' }));
            if (!opts.length) continue;
            const chosen = [...(c as HTMLSelectElement).options].find((o) => o.hasAttribute('selected'))?.getAttribute('value') ?? opts[0].value;
            input = { key, name, kind: 'select', label, value: chosen, unit: '', options: opts };
        } else if (c.classList.contains('calcpad-ui-radio')) {
            const radios = [...c.querySelectorAll('input[type=radio]')];
            if (!radios.length) continue;
            const opts = radios.map((r) => ({ label: plain(r.parentElement?.textContent ?? ''), value: r.getAttribute('value') ?? '' }));
            const chosen = radios.find((r) => r.hasAttribute('checked'))?.getAttribute('value') ?? opts[0].value;
            input = { key, name, kind: 'radio', label, value: chosen, unit: '', options: opts };
        } else {
            input = { key, name, kind: 'check', label, value: c.hasAttribute('checked') ? '1' : '0', unit: '' };
        }
        seen.add(key);
        out.push(input);
    }
    return out;
}

/** A step of 1, 2 or 5 times a power of ten that gives about a hundred positions (as the document's slider). */
function niceStep(lo: number, hi: number): number {
    const raw = (hi - lo) / 100;
    if (!(raw > 0)) return 1;
    const pow = Math.pow(10, Math.floor(Math.log10(raw)));
    const m = raw / pow;
    return (m < 1.5 ? 1 : m < 3.5 ? 2 : m < 7.5 ? 5 : 10) * pow;
}

/**
 * A style for the input form while the dock holds its inputs: their lines are hidden (the dock is the control), and so is a
 * sheet's form block left with nothing to show. Screen only; nothing printed or exported is rendered with it.
 */
export function dockHideStyle(inputs: DockInput[]): string {
    if (!inputs.length) return '';
    const rows = inputs.map((i) => `[data-ui-type][data-ui-var="${CSS.escape(i.key)}"]`).join(',');
    return `<style id="calcpad-dock-hide">@media screen{${rows}{display:none!important}`
        // A sheet's form block holds only input lines and blank ones, so with none of its inputs left it goes too.
        + `.sheet-form:not(:has([data-ui-type]:not(${rows}))){display:none!important}}</style>`;
}

const text = (): FormText => ({
    slider: (label) => td('inputDock.sliderLabel', { label }, '{label} slider'),
    range: (min, max) => td('inputDock.range', { min, max }, 'must be from {min} to {max}'),
    notNumber: () => td('inputDock.notNumber', {}, 'not a number'),
});

/** How a typed number is checked: as in the document's box (no exponent form), then the bounds. */
function check(input: EmbedInput, raw: unknown) {
    const t = String(raw ?? '').trim().replace('−', '-').replace(/\.$/, '');
    if (input.kind === 'number' && !/^[-+]?(\d+\.?\d*|\.\d+)$/.test(t)) return { ok: false as const, error: `${input.name}: not a finite number` };
    return checkValue(input, input.kind === 'number' ? t : raw);
}

export interface DockHost {
    /** The entered value of a control, or undefined for its default. */
    value: (key: string) => string | undefined;
    /** A value set in the dock; `live` while a slider is being moved. */
    changed: (key: string, value: string, live: boolean) => void;
}

const COLLAPSED_KEY = 'calcpad.inputDockCollapsed';
const read = (k: string): string | null => { try { return sessionStorage.getItem(k); } catch { return null; } };
const write = (k: string, v: string): void => { try { sessionStorage.setItem(k, v); } catch { /* storage refused */ } };

/** The panel. `root` goes into the results pane (App.vue); `update` follows each render of the input form. */
export class InputDock {
    readonly root: HTMLElement;
    private readonly list: HTMLElement;
    private readonly summary: HTMLElement;
    private readonly toggle: HTMLButtonElement;
    private readonly title: HTMLElement;
    private inputs: DockInput[] = [];
    private signature = '';

    constructor(private readonly host: DockHost) {
        this.title = el('h2', { className: 'input-dock-title', id: 'input-dock-title' });
        this.summary = el('span', { className: 'input-dock-summary', 'aria-hidden': 'true' });
        this.toggle = el('button', { type: 'button', className: 'input-dock-toggle', 'aria-controls': 'input-dock-list' });
        this.toggle.addEventListener('click', () => this.setCollapsed(!this.collapsed));
        this.list = el('div', { className: 'input-dock-list', id: 'input-dock-list' });
        const head = el('div', { className: 'input-dock-head' }, this.title, this.summary, this.toggle);
        this.root = el('section', { className: 'input-dock', 'aria-labelledby': 'input-dock-title' }, head, this.list);
        this.root.addEventListener('submit', (e) => e.preventDefault());
        this.setCollapsed(read(COLLAPSED_KEY) === '1');
    }

    get collapsed(): boolean { return this.root.classList.contains('collapsed'); }

    setCollapsed(on: boolean): void {
        this.root.classList.toggle('collapsed', on);
        this.list.hidden = on;
        this.toggle.setAttribute('aria-expanded', String(!on));
        this.paintWords();
        write(COLLAPSED_KEY, on ? '1' : '0');
    }

    /** The words, again (a language change). */
    paintWords(): void {
        this.title.textContent = td('inputDock.title', {}, 'Inputs');
        const label = this.collapsed ? td('inputDock.expand', {}, 'Show the inputs') : td('inputDock.collapse', {}, 'Hide the inputs');
        this.toggle.textContent = this.collapsed ? '▾' : '▴';
        this.toggle.setAttribute('aria-label', label);
        this.toggle.title = label;
    }

    /** The words again after a language change, the fields' accessible names and messages with them. */
    relabel(): void {
        this.paintWords();
        this.signature = '';
        this.update(this.inputs);
    }

    get items(): readonly DockInput[] { return this.inputs; }
    get hasSlider(): boolean { return this.inputs.some((i) => i.min !== undefined); }

    /** The inputs of the latest render. The fields are rebuilt only when the inputs themselves change. */
    update(inputs: DockInput[]): void {
        const sig = JSON.stringify(inputs.map(({ key, kind, label, unit, min, max, step, options }) => [key, kind, label, unit, min, max, step, options]));
        if (sig !== this.signature) {
            this.signature = sig;
            this.inputs = inputs;
            this.build();
        } else {
            this.inputs = inputs;
            // A value changed elsewhere (the Properties tab, a reload of the saved values); the field being used keeps its own.
            for (const i of inputs) {
                const f = this.list.querySelector<HTMLElement>(`.ce-field[data-name="${CSS.escape(i.key)}"]`);
                if (f && !f.contains(document.activeElement)) showValue(this.list, i.key, i, this.host.value(i.key) ?? i.value);
            }
        }
        this.paintSummary();
    }

    private build(): void {
        const active = document.activeElement;
        const was = active && this.list.contains(active) ? (active.closest('.ce-field') as HTMLElement | null)?.dataset.name : undefined;
        const words = text();
        this.list.replaceChildren(...this.inputs.map((input) => buildField(input, {
            idPrefix: 'input-dock',
            keyOf: (i) => (i as DockInput).key,
            value: (k) => this.host.value(k),
            text: words,
            check,
            changed: (i, v, live) => { this.host.changed((i as DockInput).key, v, live); this.paintSummary(); },
        })));
        if (was) this.list.querySelector<HTMLElement>(`.ce-field[data-name="${CSS.escape(was)}"] input, .ce-field[data-name="${CSS.escape(was)}"] select`)?.focus({ preventScroll: true });
    }

    /** The one line shown when the panel is collapsed: each input and its value. */
    private paintSummary(): void {
        this.summary.textContent = this.inputs.map((i) => {
            const v = this.host.value(i.key) ?? i.value;
            if (i.kind === 'check') return `${i.name} ${v === '1' ? '✓' : '✗'}`;
            if (i.options) return `${i.name} = ${i.options.find((o) => o.value === v)?.label ?? v}`;
            return `${i.name} = ${v}${i.unit ? ' ' + i.unit : ''}`;
        }).join(' · ');
    }
}
