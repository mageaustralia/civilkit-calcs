// The clipboard for whole blocks of a sheet: Copy and Cut put a block's source text on the system clipboard and keep it here too,
// so Paste still works where the browser will not let a page read the system clipboard.

let remembered = '';

/** Keeps `text` as the last copied block. */
export function rememberBlock(text: string): void { remembered = text; }

/** The last copied block ('' when none). */
export function rememberedBlock(): string { return remembered; }

/** Puts `text` on the system clipboard; falls back to a hidden textarea and execCommand where the async API is missing or refused. */
export async function copyToSystem(text: string): Promise<boolean> {
    try {
        await navigator.clipboard.writeText(text);
        return true;
    } catch { /* no permission, or an insecure page */ }
    const ta = document.createElement('textarea');
    ta.value = text;
    ta.setAttribute('readonly', '');
    ta.style.cssText = 'position:fixed;top:0;left:0;width:1px;height:1px;opacity:0';
    const had = document.activeElement as HTMLElement | null;
    document.body.appendChild(ta);
    ta.select();
    let ok = false;
    try { ok = document.execCommand('copy'); } catch { /* not allowed */ }
    ta.remove();
    had?.focus?.();
    return ok;
}

/** Copy: remember the block and put it on the system clipboard. */
export function copyBlockText(text: string): Promise<boolean> {
    remembered = text;
    return copyToSystem(text);
}

/** What Paste inserts: the system clipboard if it can be read and has text, otherwise the last copied block. */
export async function pasteSource(): Promise<string> {
    try {
        const t = await navigator.clipboard.readText();
        if (t && t.trim()) return t;
    } catch { /* unreadable: use the remembered block */ }
    return remembered;
}

/** Like pasteSource, but never asks the browser for permission: the system clipboard is read only when it is already allowed. */
export async function peekClipboard(): Promise<string> {
    try {
        const st = await navigator.permissions.query({ name: 'clipboard-read' as PermissionName });
        if (st.state === 'granted') return await pasteSource();
    } catch { /* no Permissions API for the clipboard */ }
    return remembered;
}
