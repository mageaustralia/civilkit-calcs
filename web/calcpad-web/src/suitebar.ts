/**
 * The shared CivilKit suite bar above the ribbon, on wide screens of the browser app only.
 * Left out of store builds, an installed standalone window and phone-width layouts (the
 * element is also hidden by CSS below 900px, and the script is not even loaded until the
 * window is that wide).
 */
const WIDE = '(min-width: 900px)';

export function mountSuiteBar(): void {
    if (import.meta.env.VITE_STORE_BUILD === '1') return;
    if (typeof (window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ !== 'undefined') return;
    if (window.matchMedia('(display-mode: standalone)').matches) return;
    if ((navigator as Navigator & { standalone?: boolean }).standalone) return;
    const wide = window.matchMedia(WIDE);
    let mounted = false;
    const mount = (): void => {
        if (mounted || !wide.matches) return;
        mounted = true;
        void import('./styles/civilkit-suitebar.js').then(() => {
            const bar = document.createElement('ck-suitebar');
            bar.setAttribute('current', 'calcs');
            // The body is a flex column and the ribbon is mounted after this runs, so the order
            // keeps the bar on top rather than the document position.
            document.body.insertBefore(bar, document.body.firstChild);
        });
    };
    mount();
    wide.addEventListener('change', mount);
}
