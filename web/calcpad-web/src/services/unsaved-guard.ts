// Unsaved-changes guard. A document is "unsaved" when its text differs from what was last
// written by Save or Save As (the tab manager's dirty flag). Exports (PDF, Word, Excel) are
// copies, not saves, so they do not clear it. The browser build keeps no copy of unsaved text
// across a reload, so closing the browser tab with a dirty document loses that work; this
// module marks the window title and asks the browser to warn before it unloads.

/** The marker put in front of a window title while the active document has unsaved changes. */
export const UNSAVED_MARK = '● ';

/** `title` with the unsaved marker in front when `dirty`. */
export function titleWithUnsavedMark(title: string, dirty: boolean): string {
    return dirty ? UNSAVED_MARK + title : title;
}

/**
 * Asks the browser to confirm leaving the page while `anyDirty()` is true. Browsers show their
 * own wording; the page can only ask for the prompt. Returns a function that removes the guard.
 */
export function installBeforeUnloadGuard(anyDirty: () => boolean, target: Pick<Window, 'addEventListener' | 'removeEventListener'> = window): () => void {
    const onBeforeUnload = (e: BeforeUnloadEvent) => {
        if (!anyDirty()) return;
        e.preventDefault();
        // Older Chromium and Safari need returnValue set as well.
        e.returnValue = '';
    };
    target.addEventListener('beforeunload', onBeforeUnload as EventListener);
    return () => target.removeEventListener('beforeunload', onBeforeUnload as EventListener);
}
