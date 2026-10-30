// Input rows that line up (preview page, screen only).
//
// The input form draws one paragraph per input (`<p data-ui-type="entry">` and the drop-down, radio and checkbox
// kinds). The name in front of a box has a different width on every line, so a run of inputs had boxes, sliders,
// units and notes at a different x on each row. This pass groups each run of consecutive input rows and gives the
// pieces of a column one shared width:
//
//   label   the name and its "=" (or the note in front of a drop-down): the widest label of the run, right aligned
//           so the "=" signs line up; capped at 40% of the row (and 16em), a longer label wraps inside its cell
//   box     a number box, a drop-down: the widest of the same kind in the run
//   unit    what follows the box and the slider: the widest of the rows that have the same kind and a slider
//
// The widths are CSS custom properties on each row; the rules that use them are inside `@media screen`, so print and
// every export keep the plain flow. The pass only wraps existing nodes in spans (no node is removed, so tab order,
// events and the values the controls post are untouched) and runs again when the pane is resized.
// A run is rows that are siblings with nothing between them; a text line, a heading or a blank line ends it.
// A run of one row is left as it is. Rows that are hidden (display none, as the docked inputs panel does) are left out.

export const INPUT_ALIGN_CSS = '<style>'
    // The gap the engine writes as a space before a unit; the same in print, where nothing else of this applies.
    + '.cprs-unit{margin-left:.25em}'
    + '@media screen{'
    + '.cprs-lbl{display:inline-block;box-sizing:border-box;width:var(--cprs-lw,auto);text-align:right;overflow-wrap:anywhere;vertical-align:baseline}'
    + '.cprs-unit{display:inline-block;box-sizing:border-box;min-width:var(--cprs-uw,0);vertical-align:baseline}'
    + 'p[data-cprs-run] input.calcpad-ui-input{box-sizing:border-box;min-width:var(--cprs-bw,0)}'
    + 'p[data-cprs-run] select.calcpad-ui-dropdown{min-width:var(--cprs-sw,0)}'
    + 'p[data-cprs-run] input.calcpad-ui-checkbox{margin-left:0}'
    + '}</style>';

/** Runs in the result frame (stringified by inputAlignScript). Nothing in here may refer to anything outside it. */
function inputAlignMain(): void {
    const KINDS: Record<string, boolean> = { entry: true, dropdown: true, radio: true, checkbox: true };

    const isRow = (el: Element | null): el is HTMLElement =>
        !!el && el.tagName === 'P' && KINDS[el.getAttribute('data-ui-type') || ''] === true;

    /** Moves the nodes `from` .. before `stop` (or to the end) into a new span; the white space at the edges stays outside. */
    function wrap(parent: Element, first: Node | null, stop: Node | null, cls: string, atStart: boolean): HTMLElement {
        const span = document.createElement('span');
        span.className = cls;
        const nodes: Node[] = [];
        for (let n = first; n && n !== stop; n = n.nextSibling) nodes.push(n);
        // The gap between the cell and what follows it is kept as plain text outside the cell.
        const last = nodes[nodes.length - 1];
        let tail = '';
        if (atStart && last && last.nodeType === 3) {
            const t = last.nodeValue || '';
            const m = /\s+$/.exec(t);
            if (m) { tail = m[0]; last.nodeValue = t.slice(0, t.length - m[0].length); }
        }
        if (!atStart && nodes[0] && nodes[0].nodeType === 3) {
            const t = nodes[0].nodeValue || '';
            const m = /^\s+/.exec(t);
            if (m) nodes[0].nodeValue = t.slice(m[0].length);
        }
        const ref = first || stop;
        parent.insertBefore(span, ref);
        for (const n of nodes) span.appendChild(n);
        if (tail) parent.insertBefore(document.createTextNode(tail), span.nextSibling);
        return span;
    }

    interface Row { p: HTMLElement; kind: string; lbl: HTMLElement | null; unit: HTMLElement | null; box: HTMLElement | null; key: string }

    function prepare(p: HTMLElement): Row | null {
        const kind = p.getAttribute('data-ui-type') || '';
        const eq = p.querySelector(':scope > span.eq');
        if (!eq) return null;
        let lbl: HTMLElement | null = null;
        let unit: HTMLElement | null = null;
        let box: HTMLElement | null = null;
        const done = p.getAttribute('data-cprs-a') === '1';
        if (kind === 'entry') {
            box = eq.querySelector('input.calcpad-ui-input');
            if (!box) return null;
            if (done) {
                lbl = eq.querySelector(':scope > .cprs-lbl');
                unit = eq.querySelector(':scope > .cprs-unit');
            } else {
                lbl = wrap(eq, eq.firstChild, box, 'cprs-lbl', true);
                const after = box.nextElementSibling && box.nextElementSibling.classList.contains('calcpad-ui-slider') ? box.nextElementSibling : box;
                unit = wrap(eq, after.nextSibling, null, 'cprs-unit', false);
            }
        } else {
            box = kind === 'dropdown' ? eq.querySelector('select') : null;
            if (done) lbl = p.querySelector(':scope > .cprs-lbl');
            else lbl = wrap(p, p.firstChild, eq, 'cprs-lbl', true);
        }
        p.setAttribute('data-cprs-a', '1');
        const hasSlider = kind === 'entry' && !!eq.querySelector('.calcpad-ui-slider');
        return { p, kind, lbl, unit, box, key: kind + (hasSlider ? '+s' : '') };
    }

    const widthOf = (el: HTMLElement | null): number => (el ? el.getBoundingClientRect().width : 0);

    function layout(run: Row[]): void {
        const vars = ['--cprs-lw', '--cprs-uw', '--cprs-bw', '--cprs-sw'];
        for (const r of run) {
            r.p.setAttribute('data-cprs-run', '1');
            for (const v of vars) r.p.style.removeProperty(v);
            if (r.lbl) r.lbl.style.whiteSpace = 'nowrap';
        }
        let lw = 0;
        const uw: Record<string, number> = {};
        let bw = 0;
        let sw = 0;
        for (const r of run) {
            lw = Math.max(lw, widthOf(r.lbl));
            if (r.unit) uw[r.key] = Math.max(uw[r.key] || 0, widthOf(r.unit));
            if (r.kind === 'entry') bw = Math.max(bw, widthOf(r.box));
            else if (r.kind === 'dropdown') sw = Math.max(sw, widthOf(r.box));
        }
        const p0 = run[0].p;
        const font = parseFloat(getComputedStyle(p0).fontSize) || 16;
        const cap = Math.min(0.4 * p0.clientWidth, 16 * font);
        lw = Math.min(Math.ceil(lw), Math.floor(cap));
        for (const r of run) {
            if (r.lbl) r.lbl.style.whiteSpace = '';
            r.p.style.setProperty('--cprs-lw', lw + 'px');
            if (r.kind === 'entry') r.p.style.setProperty('--cprs-bw', Math.ceil(bw) + 'px');
            if (r.kind === 'dropdown') r.p.style.setProperty('--cprs-sw', Math.ceil(sw) + 'px');
            if (r.unit && uw[r.key]) r.p.style.setProperty('--cprs-uw', Math.ceil(uw[r.key]) + 'px');
        }
    }

    function clear(p: HTMLElement): void {
        p.removeAttribute('data-cprs-run');
        for (const v of ['--cprs-lw', '--cprs-uw', '--cprs-bw', '--cprs-sw']) p.style.removeProperty(v);
    }

    function pass(): void {
        const runs: HTMLElement[][] = [];
        const seen = new Set<Element>();
        document.querySelectorAll('p[data-ui-type]').forEach((el) => {
            if (!isRow(el) || seen.has(el)) return;
            const run: HTMLElement[] = [];
            let first: Element = el;
            while (isRow(first.previousElementSibling)) first = first.previousElementSibling as Element;
            for (let n: Element | null = first; isRow(n); n = n.nextElementSibling) { run.push(n); seen.add(n); }
            runs.push(run);
        });
        for (const all of runs) {
            // A row that is not drawn (the docked panel hides the form's own input lines) has no width to measure.
            const run = all.filter((p) => p.getClientRects().length > 0);
            all.forEach((p) => { if (!run.includes(p)) clear(p); });
            if (run.length < 2) { run.forEach(clear); continue; }
            const rows = run.map(prepare).filter((r): r is Row => r !== null);
            if (rows.length < 2) { run.forEach(clear); continue; }
            layout(rows);
        }
    }

    let queued = false;
    function schedule(): void {
        if (queued) return;
        queued = true;
        requestAnimationFrame(() => { queued = false; pass(); });
    }

    pass();
    window.addEventListener('resize', schedule);
    if (document.fonts && document.fonts.ready) document.fonts.ready.then(schedule);
}

/** The script that goes into the preview page: after the page's own scripts (which draw the sliders) have run. */
export function inputAlignScript(): string {
    return '<script>document.addEventListener("DOMContentLoaded",function(){try{(' + inputAlignMain.toString() + ')()}catch(e){}});</script>';
}

/** The style and the script for a page that has input rows, and nothing for one that has none. */
export function inputAlign(html: string): string {
    return html.includes('data-ui-type="') ? INPUT_ALIGN_CSS + inputAlignScript() : '';
}
