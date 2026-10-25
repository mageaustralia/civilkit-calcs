// calcpad-rs: the Symbolic dialog (Functions tab). Exact algebra on one expression or equation: simplify, expand,
// factor, differentiate, integrate, solve, substitute, evaluate. The work is done by the caskit library compiled to
// its own wasm module (calcpad-caskit-wasm, run by symbolic-worker.ts); nothing is downloaded until the first
// Run. The result is shown typeset (KaTeX, loaded on first result) and as text, and can be copied or inserted
// into the worksheet in the form the document type takes. Everything runs in the page.

import { symbolicSolution, toCalcpadExpr } from './symbolic-text';
import type { SymItem, SymbolicAnswer } from './symbolic-text';

export type { SymItem, SymbolicAnswer } from './symbolic-text';

/** Computes one request in the worker; replaced if the module traps or the request takes too long. */
export class SymbolicClient {
    private worker: Worker | null = null;
    private seq = 0;
    private pending = new Map<number, { resolve: (a: SymbolicAnswer) => void; reject: (e: Error) => void }>();

    constructor(private readonly base: string, private readonly timeoutMs = 20000) {}

    private start(): Worker {
        if (this.worker) return this.worker;
        const w = new Worker(new URL('./symbolic-worker.ts', import.meta.url), { type: 'module' });
        w.onmessage = (e: MessageEvent<{ id: number; answer?: string; error?: string }>) => {
            const p = this.pending.get(e.data.id);
            if (!p) return;
            this.pending.delete(e.data.id);
            if (e.data.error !== undefined) p.reject(new Error(e.data.error));
            else {
                try { p.resolve(JSON.parse(e.data.answer ?? '') as SymbolicAnswer); } catch { p.reject(new Error('unreadable answer from the symbolic module')); }
            }
        };
        w.onerror = (e: ErrorEvent) => {
            const err = new Error(`the symbolic module could not load: ${e.message || 'unknown error'}`);
            for (const p of this.pending.values()) p.reject(err);
            this.pending.clear();
            this.reset();
        };
        this.worker = w;
        return w;
    }

    private reset(): void {
        this.worker?.terminate();
        this.worker = null;
    }

    run(op: string, expr: string, args: Record<string, unknown>): Promise<SymbolicAnswer> {
        return new Promise((resolve, reject) => {
            const id = ++this.seq;
            const timer = setTimeout(() => {
                if (!this.pending.delete(id)) return;
                this.reset();
                reject(new Error('This took too long and was stopped. Try a smaller expression.'));
            }, this.timeoutMs);
            this.pending.set(id, {
                resolve: (a) => { clearTimeout(timer); resolve(a); },
                reject: (e) => { clearTimeout(timer); reject(e); },
            });
            this.start().postMessage({ id, base: this.base, op, expr, args: JSON.stringify(args) });
        });
    }
}

/** What the dialog needs from the app. */
export interface SymbolicHost {
    /** The active Monaco editor, if any. */
    editor: () => any;
    /** The editor's file is a sheet. */
    isSheet: (editor: any) => boolean;
    /** Inserts text at the cursor (`sheet` is the text for a sheet). */
    insert: (editor: any, text: string, line: boolean, sheet?: string) => boolean;
}

type Katex = { renderToString: (tex: string, opts: Record<string, unknown>) => string };
let katex: Promise<Katex> | null = null;
/** KaTeX and its stylesheet (fonts included, served by the app), loaded the first time a result is typeset. */
function loadKatex(): Promise<Katex> {
    katex ??= (async () => {
        const [m] = await Promise.all([import('katex'), import('katex/dist/katex.min.css')]);
        return ((m as unknown as { default?: Katex }).default ?? (m as unknown as Katex));
    })();
    katex.catch(() => { katex = null; });
    return katex;
}

const OPS: { id: string; label: string; hint: string }[] = [
    { id: 'simplify', label: 'Simplify', hint: 'Collect like terms and combine powers.' },
    { id: 'expand', label: 'Expand', hint: 'Multiply out products and powers of sums.' },
    { id: 'factor', label: 'Factor', hint: 'Common factors, and polynomials in one variable.' },
    { id: 'differentiate', label: 'Differentiate', hint: 'The derivative with respect to a variable.' },
    { id: 'integrate', label: 'Integrate', hint: 'An antiderivative, or a definite integral when both limits are given.' },
    { id: 'solve', label: 'Solve for', hint: 'An equation for one variable (write = in the box), or equations for several: separate them with ; and list the variables with commas.' },
    { id: 'substitute', label: 'Substitute', hint: 'Replace variables by values or expressions, exactly. Write x = 2, b = 3*a.' },
    { id: 'evaluate', label: 'Evaluate', hint: 'A decimal number, given a number for every variable. Write x = 2, b = 3.5.' },
];

const STYLE = `
#cprs-sym-dialog{position:fixed;inset:0;z-index:2000;display:flex;align-items:center;justify-content:center;background:rgba(0,0,0,.35)}
#cprs-sym-dialog .sym-box{width:min(640px,94vw);max-height:90vh;overflow:auto;box-sizing:border-box;padding:16px 20px;border-radius:6px;font:13px/1.5 system-ui,sans-serif;
  background:var(--vscode-editor-background,#fff);color:var(--vscode-foreground,#222);border:1px solid var(--vscode-panel-border,#ccc);box-shadow:0 8px 28px rgba(0,0,0,.35)}
#cprs-sym-dialog h3{margin:0 0 8px;font-size:16px}
#cprs-sym-dialog label{display:flex;flex-direction:column;gap:2px;font-size:12px;min-width:0}
#cprs-sym-dialog .sym-row{display:flex;flex-wrap:wrap;gap:8px 12px;margin:8px 0;align-items:flex-end}
#cprs-sym-dialog .sym-row>label{flex:1 1 130px}
#cprs-sym-dialog .sym-row>label[hidden]{display:none}
#cprs-sym-dialog .sym-row>.sym-hint{flex:1 1 100%;margin:0}
#cprs-sym-dialog input,#cprs-sym-dialog select,#cprs-sym-dialog textarea{box-sizing:border-box;width:100%;padding:5px 8px;font:inherit;font-size:13px;color:var(--vscode-input-foreground,inherit);
  background:var(--vscode-input-background,#fff);border:1px solid var(--vscode-input-border,#bbb);border-radius:3px}
#cprs-sym-dialog textarea{font-family:var(--vscode-editor-font-family,ui-monospace,Consolas,monospace);resize:vertical}
#cprs-sym-dialog input:focus-visible,#cprs-sym-dialog select:focus-visible,#cprs-sym-dialog textarea:focus-visible,#cprs-sym-dialog button:focus-visible{outline:2px solid var(--vscode-focusBorder,#0060c0);outline-offset:1px}
#cprs-sym-dialog button{padding:5px 14px;font:inherit;cursor:pointer;color:var(--vscode-button-secondaryForeground,inherit);background:var(--vscode-button-secondaryBackground,rgba(128,128,128,.18));border:1px solid var(--vscode-input-border,#bbb);border-radius:3px}
#cprs-sym-dialog button.sym-primary{color:var(--vscode-button-foreground,#fff);background:var(--vscode-button-background,#0060c0);border-color:var(--vscode-button-background,#0060c0)}
#cprs-sym-dialog button:disabled{opacity:.5;cursor:default}
#cprs-sym-dialog .sym-hint{margin:2px 0 0;font-size:12px;opacity:.75}
#cprs-sym-dialog .sym-err{margin:8px 0;padding:6px 10px;border-radius:3px;color:var(--vscode-errorForeground,#b00020);border:1px solid var(--vscode-inputValidation-errorBorder,#b00020);background:var(--vscode-inputValidation-errorBackground,transparent)}
#cprs-sym-dialog .sym-out{margin:10px 0;padding:8px 10px;border:1px solid var(--vscode-panel-border,#ccc);border-radius:3px}
#cprs-sym-dialog .sym-math{overflow-x:auto;overflow-y:hidden;padding:4px 0;font-size:1.15em}
#cprs-sym-dialog .sym-math .katex-display{margin:.2em 0}
#cprs-sym-dialog .sym-ascii{margin:4px 0 0;padding:4px 6px;font:12px/1.4 var(--vscode-editor-font-family,ui-monospace,Consolas,monospace);overflow-x:auto;white-space:pre;user-select:all;background:rgba(128,128,128,.12);border-radius:3px}
#cprs-sym-dialog .sym-foot{display:flex;flex-wrap:wrap;gap:8px;justify-content:flex-end;margin-top:12px}
@media (max-width:600px){#cprs-sym-dialog .sym-box{padding:12px 14px;max-height:94vh}#cprs-sym-dialog .sym-row>label{flex-basis:100%}}
`;

/** `x = 2, b = 3*a` (or separated by ; or new lines) as a name-to-text map; commas inside brackets do not split. */
export function parseValues(text: string): { map: Record<string, string>; error: string | null } {
    const parts: string[] = [];
    let depth = 0, cur = '';
    for (const ch of text) {
        if ('([{'.includes(ch)) depth++;
        else if (')]}'.includes(ch)) depth--;
        if ((ch === ',' && depth <= 0) || ch === ';' || ch === '\n') { parts.push(cur); cur = ''; } else cur += ch;
    }
    parts.push(cur);
    const map: Record<string, string> = {};
    for (const p of parts.map(s => s.trim()).filter(Boolean)) {
        const m = /^([\p{L}][\p{L}\p{N}_]*)\s*=\s*(.+)$/u.exec(p);
        if (!m) return { map, error: `"${p}" is not a value. Write it as name = value, for example x = 2.` };
        map[m[1]] = m[2].trim();
    }
    return { map, error: null };
}

const NAME = /^[\p{L}][\p{L}\p{N}_]*$/u;

export class SymbolicDialog {
    private readonly client: SymbolicClient;

    constructor(private readonly host: SymbolicHost) {
        this.client = new SymbolicClient(new URL(import.meta.env.BASE_URL || './', document.baseURI).href);
    }

    open(): void {
        document.getElementById('cprs-sym-dialog')?.remove();
        if (!document.getElementById('cprs-sym-style')) {
            const st = document.createElement('style');
            st.id = 'cprs-sym-style';
            st.textContent = STYLE;
            document.head.append(st);
        }
        const editor = this.host.editor();
        const sheet = !!editor && this.host.isSheet(editor);

        const back = document.createElement('div');
        back.id = 'cprs-sym-dialog';
        const box = document.createElement('div');
        box.className = 'sym-box';
        box.setAttribute('role', 'dialog');
        box.setAttribute('aria-modal', 'true');
        box.setAttribute('aria-label', 'Symbolic algebra');
        back.append(box);

        const field = (cls: string, label: string, el: HTMLElement, hidden = false): HTMLLabelElement => {
            const l = document.createElement('label');
            l.className = cls;
            l.hidden = hidden;
            l.append(label, el);
            return l;
        };
        const input = (name: string, ph = '', value = ''): HTMLInputElement => {
            const i = document.createElement('input');
            i.type = 'text';
            i.name = name;
            i.placeholder = ph;
            i.value = value;
            i.autocomplete = 'off';
            i.spellcheck = false;
            i.setAttribute('autocapitalize', 'off');
            return i;
        };

        const h = document.createElement('h3');
        h.textContent = 'Symbolic algebra';
        const expr = document.createElement('textarea');
        expr.name = 'expr';
        expr.rows = 2;
        expr.placeholder = 'w*x^4/24    or    M = w*L^2/8';
        expr.spellcheck = false;
        expr.setAttribute('autocapitalize', 'off');
        expr.setAttribute('aria-describedby', 'cprs-sym-syntax');
        const syntax = document.createElement('p');
        syntax.id = 'cprs-sym-syntax';
        syntax.className = 'sym-hint';
        syntax.textContent = 'Write * for every product and ^ for powers. Functions: sin, cos, ln, exp, sqrt. Constants: pi, e. Enter runs; Shift+Enter starts a new line.';

        const op = document.createElement('select');
        op.name = 'op';
        for (const o of OPS) {
            const opt = document.createElement('option');
            opt.value = o.id;
            opt.textContent = o.label;
            op.append(opt);
        }
        const varIn = input('var', 'x', 'x');
        const orderIn = input('order', '1', '1');
        orderIn.inputMode = 'numeric';
        const lowIn = input('lower', 'optional, e.g. 0');
        const upIn = input('upper', 'optional, e.g. L');
        const valuesIn = input('values', 'x = 2, b = 3*a');
        const varLabel = document.createElement('span');
        const lVar = field('', '', varIn);
        lVar.firstChild!.replaceWith(varLabel);
        const lOrder = field('', 'Order', orderIn, true);
        const lLow = field('', 'Lower limit', lowIn, true);
        const lUp = field('', 'Upper limit', upIn, true);
        const lValues = field('sym-wide', 'Values', valuesIn, true);
        const hint = document.createElement('p');
        hint.className = 'sym-hint';
        hint.setAttribute('aria-live', 'polite');

        const run = document.createElement('button');
        run.type = 'button';
        run.className = 'sym-primary';
        run.textContent = 'Run';
        run.setAttribute('data-cprs-sym-run', '');
        const row1 = document.createElement('div');
        row1.className = 'sym-row';
        row1.append(field('', 'Operation', op), lVar, lOrder, lLow, lUp, lValues, run);

        const err = document.createElement('div');
        err.className = 'sym-err';
        err.setAttribute('role', 'alert');
        err.setAttribute('data-cprs-sym-error', '');
        err.hidden = true;

        const out = document.createElement('div');
        out.className = 'sym-out';
        out.hidden = true;
        const math = document.createElement('div');
        math.className = 'sym-math';
        math.setAttribute('data-cprs-sym-math', '');
        math.setAttribute('aria-live', 'polite');
        const ascii = document.createElement('div');
        ascii.className = 'sym-ascii';
        ascii.setAttribute('data-cprs-sym-ascii', '');
        ascii.tabIndex = 0;
        ascii.setAttribute('aria-label', 'Result as text');
        out.append(math, ascii);

        const formSel = document.createElement('select');
        formSel.name = 'form';
        const forms = sheet
            ? [['formula', 'Formula block ($$ ... $$)'], ['line', 'Quantity line (name: expression)']]
            : [['line', 'Calculation line (name = expression)'], ['comment', 'Comment line with the result']];
        for (const [v, t] of forms) {
            const o = document.createElement('option');
            o.value = v;
            o.textContent = t;
            formSel.append(o);
        }
        const nameIn = input('name', 'result', 'result');
        const insRow = document.createElement('div');
        insRow.className = 'sym-row';
        const lineHint = document.createElement('p');
        lineHint.className = 'sym-hint';
        lineHint.textContent = 'A line is calculated by the engine, so every symbol in it needs a value in the worksheet. A formula block or a comment is only shown.';
        insRow.append(field('', 'Insert as', formSel), field('', 'Name', nameIn), lineHint);
        insRow.hidden = true;

        const copy = document.createElement('button');
        copy.type = 'button';
        copy.textContent = 'Copy';
        copy.disabled = true;
        copy.setAttribute('data-cprs-sym-copy', '');
        const ins = document.createElement('button');
        ins.type = 'button';
        ins.className = 'sym-primary';
        ins.textContent = 'Insert into worksheet';
        ins.disabled = true;
        ins.setAttribute('data-cprs-sym-insert', '');
        const close = document.createElement('button');
        close.type = 'button';
        close.textContent = 'Close';
        const foot = document.createElement('div');
        foot.className = 'sym-foot';
        foot.append(copy, ins, close);

        box.append(h, field('sym-expr', 'Expression or equation', expr), syntax, row1, hint, err, out, insRow, foot);

        let answer: SymbolicAnswer | null = null;
        let nameTouched = false;
        let runSeq = 0;
        nameIn.oninput = () => { nameTouched = true; };

        const setOp = () => {
            const o = op.value;
            const meta = OPS.find(x => x.id === o)!;
            hint.textContent = meta.hint;
            const needsVar = o === 'differentiate' || o === 'integrate' || o === 'solve';
            lVar.hidden = !needsVar;
            varLabel.textContent = o === 'solve' ? 'Solve for' : 'With respect to';
            varIn.placeholder = o === 'solve' ? 'L    or    x, y' : 'x';
            lOrder.hidden = o !== 'differentiate';
            lLow.hidden = lUp.hidden = o !== 'integrate';
            lValues.hidden = !(o === 'substitute' || o === 'evaluate');
        };
        op.onchange = setOp;
        setOp();

        const showError = (msg: string) => {
            answer = null;
            out.hidden = true;
            insRow.hidden = true;
            copy.disabled = ins.disabled = true;
            err.textContent = msg;
            err.hidden = false;
        };

        const doRun = async () => {
            const o = op.value;
            const text = expr.value.trim();
            if (!text) { showError('Enter an expression or an equation first.'); expr.focus(); return; }
            const args: Record<string, unknown> = {};
            if (o === 'differentiate' || o === 'integrate' || o === 'solve') {
                const v = varIn.value.trim();
                if (!v) { showError('Name the variable.'); varIn.focus(); return; }
                if (o === 'solve') {
                    const names = v.split(/[\s,;]+/).filter(Boolean);
                    args.var = names[0];
                    args.vars = names;
                } else args.var = v;
            }
            if (o === 'differentiate') {
                const n = Number(orderIn.value.trim() || '1');
                if (!Number.isInteger(n) || n < 1) { showError('The order must be a whole number, 1 or more.'); orderIn.focus(); return; }
                args.order = n;
            }
            if (o === 'integrate') {
                if (lowIn.value.trim()) args.lower = lowIn.value.trim();
                if (upIn.value.trim()) args.upper = upIn.value.trim();
            }
            if (o === 'substitute' || o === 'evaluate') {
                const { map, error } = parseValues(valuesIn.value);
                if (error) { showError(error); valuesIn.focus(); return; }
                if (!Object.keys(map).length && o === 'substitute') { showError('Give at least one value, for example x = 2.'); valuesIn.focus(); return; }
                args.map = map;
            }
            const mine = ++runSeq;
            run.disabled = true;
            run.textContent = 'Working...';
            err.hidden = true;
            try {
                const a = await this.client.run(o, text, args);
                if (mine !== runSeq) return;
                if (!a.ok) { showError(plain(a.error)); return; }
                answer = a;
                err.hidden = true;
                ascii.textContent = a.ascii;
                math.textContent = a.ascii;
                out.hidden = false;
                insRow.hidden = false;
                copy.disabled = ins.disabled = false;
                if (!nameTouched) nameIn.value = o === 'solve' && a.items.length === 1 && a.items[0].name ? a.items[0].name : 'result';
                try {
                    const k = await loadKatex();
                    if (mine === runSeq) {
                        math.innerHTML = k.renderToString(a.latex, { displayMode: true, throwOnError: false, output: 'htmlAndMathml' });
                        math.setAttribute('data-typeset', '1');
                    }
                } catch {
                    math.removeAttribute('data-typeset');
                }
            } catch (e) {
                if (mine === runSeq) showError(e instanceof Error ? e.message : String(e));
            } finally {
                if (mine === runSeq) { run.disabled = false; run.textContent = 'Run'; }
            }
        };
        run.onclick = () => void doRun();
        expr.onkeydown = (e) => { if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); void doRun(); } };
        for (const i of [varIn, orderIn, lowIn, upIn, valuesIn]) i.onkeydown = (e) => { if (e.key === 'Enter') { e.preventDefault(); void doRun(); } };

        copy.onclick = async () => {
            if (!answer) return;
            try {
                await navigator.clipboard.writeText(answer.ascii);
                copy.textContent = 'Copied';
            } catch {
                const r = document.createRange();
                r.selectNodeContents(ascii);
                const s = getSelection();
                s?.removeAllRanges();
                s?.addRange(r);
                copy.textContent = 'Selected';
            }
            setTimeout(() => { copy.textContent = 'Copy'; }, 1500);
        };

        ins.onclick = () => {
            if (!answer) return;
            const name = nameIn.value.trim();
            if (formSel.value !== 'formula' && !NAME.test(name)) {
                err.textContent = 'Enter a name made of letters, digits and _, starting with a letter.';
                err.hidden = false;
                nameIn.focus();
                return;
            }
            const ed = this.host.editor();
            if (!ed || !ed.getModel?.()) { err.textContent = 'Open a worksheet first, then insert the result.'; err.hidden = false; return; }
            const text = symbolicSolution(answer, formSel.value as 'line' | 'comment' | 'formula', name, this.host.isSheet(ed));
            this.host.insert(ed, text.calcpad, true, text.sheet);
            err.hidden = true;
            onClose();
        };

        const onKey = (e: KeyboardEvent) => {
            if (e.key === 'Escape') { e.stopPropagation(); onClose(); return; }
            if (e.key === 'Tab') {
                const f = Array.from(box.querySelectorAll<HTMLElement>('input,select,textarea,button')).filter(x => !x.hidden && !x.closest('[hidden]') && !(x as HTMLButtonElement).disabled);
                if (!f.length) return;
                const first = f[0], last = f[f.length - 1];
                if (e.shiftKey && document.activeElement === first) { e.preventDefault(); last.focus(); }
                else if (!e.shiftKey && document.activeElement === last) { e.preventDefault(); first.focus(); }
            }
        };
        const onClose = () => { back.remove(); document.removeEventListener('keydown', onKey, true); };
        close.onclick = onClose;
        back.onmousedown = (e) => { if (e.target === back) onClose(); };
        document.addEventListener('keydown', onKey, true);
        document.body.append(back);
        expr.focus();
    }
}

/** The library's message, ready for the dialog: a sentence with a capital and a full stop. */
export function plain(msg: string): string {
    const m = msg.trim();
    if (!m) return 'Something went wrong.';
    const s = m[0].toUpperCase() + m.slice(1);
    return /[.!?]$/.test(s) ? s : `${s}.`;
}

export { toCalcpadExpr };
export type { SymItem as SymbolicItem };
