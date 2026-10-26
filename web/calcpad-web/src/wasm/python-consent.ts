// calcpad-rs: the one-time "this worksheet runs Python" question, asked per open tab before its first run.
// An answer of Run is remembered for the tab. Not now is remembered until the worksheet's Python code changes.

const allowed = new Set<string>();
const declined = new Map<string, string>();
let open: Promise<boolean> | null = null;

function ask(): Promise<boolean> {
    return new Promise((resolve) => {
        const back = document.createElement('div');
        back.id = 'cprs-python-consent';
        back.style.cssText = 'position:fixed;inset:0;z-index:2100;display:flex;align-items:center;justify-content:center;background:rgba(0,0,0,.4)';
        const box = document.createElement('div');
        box.setAttribute('role', 'alertdialog');
        box.setAttribute('aria-label', 'This worksheet runs Python code');
        box.style.cssText = 'max-width:min(480px,92vw);padding:18px 22px;border-radius:6px;font:13px/1.5 system-ui,sans-serif;'
            + 'background:var(--vscode-editor-background,#fff);color:var(--vscode-foreground,#222);border:1px solid var(--vscode-panel-border,#ccc);box-shadow:0 8px 28px rgba(0,0,0,.35)';
        const h = document.createElement('h3');
        h.textContent = 'This worksheet runs Python code';
        h.style.cssText = 'margin:0 0 10px;font-size:16px';
        const p = document.createElement('p');
        p.textContent = 'Calculating it runs the #python blocks in a sandboxed worker in this browser. They cannot reach the network or your files. Run them only for worksheets you trust.';
        p.style.margin = '6px 0 14px';
        const row = document.createElement('div');
        row.style.cssText = 'display:flex;gap:8px;justify-content:flex-end';
        const mk = (label: string, value: boolean, id: string) => {
            const b = document.createElement('button');
            b.textContent = label;
            b.id = id;
            b.style.cssText = 'padding:4px 14px;font:inherit;cursor:pointer';
            b.onclick = () => { done(value); };
            return b;
        };
        const no = mk('Not now', false, 'cprs-python-no');
        const yes = mk('Run Python', true, 'cprs-python-yes');
        const done = (v: boolean) => { back.remove(); document.removeEventListener('keydown', onKey, true); resolve(v); };
        const onKey = (e: KeyboardEvent) => { if (e.key === 'Escape') { e.stopPropagation(); done(false); } };
        document.addEventListener('keydown', onKey, true);
        row.append(no, yes);
        box.append(h, p, row);
        back.append(box);
        document.body.append(back);
        yes.focus();
    });
}

/** Whether the tab `scope` may run the Python of a worksheet whose Python code is `signature`. */
export async function mayRunPython(scope: string, signature: string): Promise<boolean> {
    if (allowed.has(scope)) return true;
    if (declined.get(scope) === signature) return false;
    // One question at a time; callers that arrive meanwhile share the answer.
    open ??= ask().finally(() => { open = null; });
    const yes = await open;
    if (yes) { allowed.add(scope); declined.delete(scope); } else declined.set(scope, signature);
    return yes;
}

/**
 * Asked when a block ran out of time: run the worksheet again with the longer limit? An answer of Run again is kept for the tab
 * (the runner's `useLongLimit`), so this is asked once. A slow device (a phone) can need several times what a desktop does.
 */
export function askLongerLimit(seconds: number, longSeconds: number): Promise<boolean> {
    return new Promise((resolve) => {
        const back = document.createElement('div');
        back.id = 'cprs-python-longer';
        back.style.cssText = 'position:fixed;inset:0;z-index:2100;display:flex;align-items:center;justify-content:center;background:rgba(0,0,0,.4)';
        const box = document.createElement('div');
        box.setAttribute('role', 'alertdialog');
        box.setAttribute('aria-label', 'The Python ran out of time');
        box.style.cssText = 'max-width:min(480px,92vw);padding:18px 22px;border-radius:6px;font:13px/1.5 system-ui,sans-serif;'
            + 'background:var(--vscode-editor-background,#fff);color:var(--vscode-foreground,#222);border:1px solid var(--vscode-panel-border,#ccc);box-shadow:0 8px 28px rgba(0,0,0,.35)';
        const h = document.createElement('h3');
        h.textContent = 'The Python ran out of time';
        h.style.cssText = 'margin:0 0 10px;font-size:16px';
        const p = document.createElement('p');
        p.textContent = `A Python block did not finish in ${seconds} s and was stopped. This device may simply be slower than the one the worksheet was written on. `
            + `Run it again with a ${longSeconds} s limit per block? A block that never ends is still stopped, just later.`;
        p.style.margin = '6px 0 14px';
        const row = document.createElement('div');
        row.style.cssText = 'display:flex;gap:8px;justify-content:flex-end';
        const mk = (label: string, value: boolean, id: string) => {
            const b = document.createElement('button');
            b.textContent = label;
            b.id = id;
            b.style.cssText = 'padding:4px 14px;font:inherit;cursor:pointer';
            b.onclick = () => { done(value); };
            return b;
        };
        const no = mk('Not now', false, 'cprs-python-longer-no');
        const yes = mk(`Run again with ${longSeconds} s`, true, 'cprs-python-longer-yes');
        const done = (v: boolean) => { back.remove(); document.removeEventListener('keydown', onKey, true); resolve(v); };
        const onKey = (e: KeyboardEvent) => { if (e.key === 'Escape') { e.stopPropagation(); done(false); } };
        document.addEventListener('keydown', onKey, true);
        row.append(no, yes);
        box.append(h, p, row);
        back.append(box);
        document.body.append(back);
        yes.focus();
    });
}
