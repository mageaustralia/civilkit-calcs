// "A new version is available": the service worker (sw.js) swaps in a new build in the background, but a
// tab keeps running the build it loaded. This tells the person, and lets them reload when it suits them
// (never on its own, so unsaved work is not lost).

const ID = 'cprs-update-notice';

function show(): void {
    if (document.getElementById(ID)) return;
    const box = document.createElement('div');
    box.id = ID;
    box.setAttribute('role', 'status');
    box.style.cssText = 'position:fixed;left:12px;bottom:34px;z-index:3000;display:flex;align-items:center;gap:10px;max-width:min(460px,calc(100vw - 24px));' +
        'padding:8px 12px;border-radius:4px;font:13px/1.4 system-ui,sans-serif;' +
        'background:var(--vscode-notifications-background,#252526);color:var(--vscode-notifications-foreground,#ccc);' +
        'border:1px solid var(--vscode-focusBorder,#007fd4);box-shadow:0 4px 14px rgba(0,0,0,.4)';
    const text = document.createElement('span');
    text.textContent = 'A new version of CivilKit Calcs is available. Save your work, then reload.';
    const reload = document.createElement('button');
    reload.textContent = 'Reload';
    reload.className = 'modal-btn primary';
    reload.setAttribute('data-cprs-update-reload', '');
    reload.onclick = () => location.reload();
    const later = document.createElement('button');
    later.textContent = 'Later';
    later.className = 'modal-btn';
    later.onclick = () => box.remove();
    box.append(text, reload, later);
    document.body.appendChild(box);
}

/** Watches a registered service worker and shows the notice when a newer build has taken over. */
export function watchForUpdates(reg: ServiceWorkerRegistration): void {
    const check = (w: ServiceWorker | null) => {
        if (!w) return;
        // The first install (no worker controlling this tab yet) has nothing to reload for.
        const hadController = !!navigator.serviceWorker.controller;
        w.addEventListener('statechange', () => {
            if (w.state === 'activated' && hadController) show();
        });
    };
    check(reg.installing);
    reg.addEventListener('updatefound', () => check(reg.installing));
    // A tab left open for days still notices: look for a new build now and then and on coming back.
    const look = () => { reg.update().catch(() => { /* offline */ }); };
    setInterval(look, 30 * 60 * 1000);
    document.addEventListener('visibilitychange', () => { if (document.visibilityState === 'visible') look(); });
}
