// calcpad-rs: sorting a sheet's tables in the preview. Each column heading of a rendered table
// (one with a <thead>) becomes a button: a click or tap sorts the rows ascending, then
// descending, then back to the order of the source. It is view state only: the source is not
// edited, printing puts the rows back in source order first, and print and export render the
// page again from the source anyway. The host keeps the state per document tab so a
// re-rendered preview sorts the same tables again (App.vue, 'tableSort' messages).

/** Sort state: table key to [column, direction], direction 1 ascending or -1 descending. */
export type TableSortState = Record<string, [number, number]>;

/**
 * The order to show rows in, as indexes into `cells`, for one column's cell texts. Numbers
 * (a leading number, as in "12.5 kN", "1,200" or "-3") sort by value when every non-empty cell has one,
 * otherwise text sorts naturally ("a2" before "a10"). Empty cells go last either way, and equal
 * cells keep their source order. Self-contained: the preview script embeds its source.
 */
export function sortOrder(cells: string[], dir: number): number[] {
    const num = (s: string): number => {
        const m = /^[\s(]*([+\-−]?(?:(?:\d{1,3}(?:,\d{3})+|\d+)(?:\.\d*)?|\.\d+)(?:[eE][+\-]?\d+)?)/.exec(s);
        return m ? parseFloat(m[1].replace(/−/, '-').replace(/,/g, '')) : NaN;
    };
    const text = cells.map(c => c.trim());
    const numeric = text.every(t => t === '' || !isNaN(num(t))) && text.some(t => t !== '');
    const idx = text.map((_t, i) => i);
    return idx.sort((a, b) => {
        const ea = text[a] === '', eb = text[b] === '';
        if (ea || eb) return ea === eb ? a - b : ea ? 1 : -1;
        const c = numeric ? num(text[a]) - num(text[b]) : text[a].localeCompare(text[b], undefined, { numeric: true, sensitivity: 'base' });
        return c ? c * dir : a - b;
    });
}

/** The script that makes the headings sort buttons, for a page in the frame `frameId`. */
export function tableSortScript(frameId: string, state: TableSortState): string {
    const css = '.ck-sort{all:unset;cursor:pointer;display:inline-flex;align-items:baseline;gap:.25em;font:inherit;color:inherit;border-radius:3px}'
        + '.ck-sort:focus-visible{outline:2px solid #4a90d9;outline-offset:1px}'
        + '.ck-sort-i{display:inline-block;min-width:.8em;font-size:.75em;opacity:.35}'
        + '.ck-sort:hover .ck-sort-i,th[aria-sort] .ck-sort-i{opacity:.9}'
        + '@media (pointer:coarse){.ck-sort{min-height:32px;align-items:center}}'
        + '@media print{.ck-sort-i{display:none}}';
    const body = [
        "document.addEventListener('DOMContentLoaded', function() {",
        '  var FRAME_ID = ' + JSON.stringify(frameId) + ';',
        '  var STATE = ' + JSON.stringify(state).replace(/</g, '\\u003c') + ';',
        '  var sortOrder = ' + String(sortOrder) + ';',
        "  var st = document.createElement('style'); st.textContent = " + JSON.stringify(css) + '; document.head.appendChild(st);',
        '  var tables = [], seen = {};',
        "  document.querySelectorAll('table').forEach(function(t) {",
        '    var head = t.tHead, body = t.tBodies[0];',
        "    if (!head || !body || body.rows.length < 2 || t.closest('.calcpad-ui-datagrid, .jss_container')) return;",
        '    var ths = head.rows[head.rows.length - 1].cells;',
        "    var name = Array.prototype.map.call(ths, function(c) { return c.textContent.trim(); }).join('|');",
        "    seen[name] = (seen[name] || 0) + 1;",
        "    var tb = { el: t, body: body, rows: Array.prototype.slice.call(body.rows), key: name + '#' + seen[name], ths: ths, col: -1, dir: 0 };",
        '    tables.push(tb);',
        '    Array.prototype.forEach.call(ths, function(th, i) {',
        "      var b = document.createElement('button'); b.type = 'button'; b.className = 'ck-sort';",
        '      while (th.firstChild) b.appendChild(th.firstChild);',
        "      var ic = document.createElement('span'); ic.className = 'ck-sort-i'; ic.setAttribute('aria-hidden', 'true');",
        '      b.appendChild(ic); th.appendChild(b);',
        "      b.addEventListener('click', function(e) { e.stopPropagation(); cycle(tb, i); });",
        "      b.addEventListener('dblclick', function(e) { e.stopPropagation(); });",
        '    });',
        '    var s = STATE[tb.key];',
        '    if (s && s[0] < ths.length) { tb.col = s[0]; tb.dir = s[1]; }',
        '    apply(tb);',
        '  });',
        '  function cycle(tb, i) {',
        '    if (tb.col !== i) { tb.col = i; tb.dir = 1; } else if (tb.dir === 1) tb.dir = -1; else { tb.col = -1; tb.dir = 0; }',
        '    if (tb.dir) STATE[tb.key] = [tb.col, tb.dir]; else delete STATE[tb.key];',
        "    try { window.parent.postMessage({ type: 'tableSort', state: STATE, groupId: FRAME_ID }, '*'); } catch (_e) {}",
        '    apply(tb);',
        '  }',
        '  function apply(tb, plain) {',
        '    var order = tb.rows.map(function(_r, i) { return i; });',
        '    if (tb.dir && !plain) order = sortOrder(tb.rows.map(function(r) { var c = r.cells[tb.col]; return c ? c.textContent : \'\'; }), tb.dir);',
        '    order.forEach(function(i) { tb.body.appendChild(tb.rows[i]); });',
        '    Array.prototype.forEach.call(tb.ths, function(th, i) {',
        '      var on = !plain && tb.dir && i === tb.col;',
        "      if (on) th.setAttribute('aria-sort', tb.dir > 0 ? 'ascending' : 'descending'); else th.removeAttribute('aria-sort');",
        "      var b = th.querySelector('.ck-sort'); if (!b) return;",
        "      b.querySelector('.ck-sort-i').textContent = on ? (tb.dir > 0 ? '\\u25b2' : '\\u25bc') : '\\u21c5';",
        "      var label = b.textContent.slice(0, -1).trim();",
        "      b.title = on ? (tb.dir > 0 ? 'Sorted ascending: click for descending' : 'Sorted descending: click for the source order') : 'Sort by this column (view only, the source is unchanged)';",
        "      b.setAttribute('aria-label', label + ', ' + b.title);",
        '    });',
        '  }',
        // A printed page is the report: rows in source order, as written.
        "  window.addEventListener('beforeprint', function() { tables.forEach(function(tb) { apply(tb, true); }); });",
        "  window.addEventListener('afterprint', function() { tables.forEach(function(tb) { apply(tb); }); });",
        '});',
    ].join('\n');
    return '<' + 'script>' + body + '</' + 'script>';
}

/** `html` with the sort script added at the end of its body. */
export function injectTableSort(html: string, frameId: string, state: TableSortState): string {
    if (!/<thead\b/i.test(html)) return html;
    const script = tableSortScript(frameId, state);
    const i = html.toLowerCase().lastIndexOf('</body>');
    return i < 0 ? html + script : html.slice(0, i) + script + html.slice(i);
}
