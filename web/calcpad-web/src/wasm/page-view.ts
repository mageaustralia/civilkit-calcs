// calcpad-rs: page view. The report is printed by the browser (print.ts) with the paper size and margins of the PDF
// settings; this lays the preview out on pages of the same size and margins, with the automatic page breaks drawn, so a
// long report can be checked without printing it. A script put into the preview does the layout (the preview is a
// sandboxed frame, so the host cannot reach its document): it sets the page width, finds the blocks of the report and
// moves each block that would cross the bottom margin to the top of the next page, as a block does when it is printed
// with `break-inside: avoid`. A block taller than a page is left to run on. The browser may split a long paragraph or a
// table between pages where this keeps it whole, so the print dialog's own preview has the last word on the very last
// line of a page. Screen only: the style and script are put into the preview, never into a printed or exported report.
// Plain data and strings with no imports, so a Node check can load it.

export interface PageGeometry {
    /** Paper size in CSS pixels (96 to the inch). */
    width: number;
    height: number;
    top: number;
    right: number;
    bottom: number;
    left: number;
}

export interface PageViewSettings {
    format?: string;
    orientation?: string;
    marginTop?: string;
    marginRight?: string;
    marginBottom?: string;
    marginLeft?: string;
}

/** Paper sizes in millimetres, portrait. */
const PAPER_MM: Record<string, [number, number]> = {
    a3: [297, 420], a4: [210, 297], a5: [148, 210], b4: [250, 353], b5: [176, 250],
    letter: [215.9, 279.4], legal: [215.9, 355.6], ledger: [279.4, 431.8], tabloid: [279.4, 431.8],
};

const UNIT_PX: Record<string, number> = { mm: 96 / 25.4, cm: 96 / 2.54, in: 96, pt: 96 / 72, pc: 16, px: 1 };

/** A CSS length (`0.75in`, `20mm`) in pixels; the fallback when it is not one. */
export function lengthToPx(value: string | undefined, fallback: number): number {
    const m = /^\s*(\d+(?:\.\d+)?)\s*(mm|cm|in|pt|pc|px)?\s*$/i.exec(value ?? '');
    if (!m) return fallback;
    return Number(m[1]) * UNIT_PX[(m[2] ?? 'px').toLowerCase()];
}

/** The page the PDF settings describe, as print.ts's @page rule writes it (A4 portrait with 0.75in and 0.5in margins). */
export function pageGeometry(o: PageViewSettings = {}): PageGeometry {
    const [w, h] = PAPER_MM[(o.format ?? 'A4').toLowerCase()] ?? PAPER_MM.a4;
    const landscape = o.orientation === 'landscape';
    const mm = 96 / 25.4;
    return {
        width: (landscape ? h : w) * mm,
        height: (landscape ? w : h) * mm,
        top: lengthToPx(o.marginTop, 72),
        right: lengthToPx(o.marginRight, 48),
        bottom: lengthToPx(o.marginBottom, 72),
        left: lengthToPx(o.marginLeft, 48),
    };
}

/** The page number the page labels use is held in the frame; this is the id of the element that draws the pages. */
export const PAGE_VIEW_ID = 'cpd-page-view';

/** The frame's half, as plain JavaScript. `G` is the geometry, `T` the label of a page ("Page {n} of {total}"). */
function script(g: PageGeometry, label: string): string {
    return `(function(){
  var G = ${JSON.stringify(g)}, LABEL = ${JSON.stringify(label)}, GAP = 20, ID = ${JSON.stringify(PAGE_VIEW_ID)};
  var BLOCK = /^(block|table|list-item|flex|grid|flow-root)$/;
  var timer = 0;
  function isUnit(el) {
    var tag = el.tagName;
    if (/^(SCRIPT|STYLE|LINK|META|TITLE|HEAD|NOSCRIPT|TEMPLATE)$/.test(tag) || el.id === ID) return false;
    if (/^(TABLE|FIGURE|IMG|SVG|PRE|HR|CANVAS|UL|OL|BLOCKQUOTE|H[1-6]|P)$/i.test(tag)) return true;
    for (var c = el.firstElementChild; c; c = c.nextElementSibling) {
      if (c.id === ID) continue;
      var d = getComputedStyle(c).display;
      if (BLOCK.test(d) && isUnitCandidate(c)) return false;
    }
    return true;
  }
  function isUnitCandidate(c) { return !/^(SCRIPT|STYLE|LINK|META|BR|A)$/.test(c.tagName) || (c.tagName === 'A' && !c.classList.contains('lineLink')); }
  function units(root, out) {
    for (var c = root.firstElementChild; c; c = c.nextElementSibling) {
      if (getComputedStyle(c).display === 'none') continue;
      if (!isUnitCandidate(c) && c.tagName !== 'DIV') continue;
      if (c.id === ID || /^(SCRIPT|STYLE|LINK|META)$/.test(c.tagName)) continue;
      if (isUnit(c)) out.push(c); else units(c, out);
    }
    return out;
  }
  function forcedBefore(el) { var s = el.getAttribute('style') || ''; return /(page-break-before|break-before)\\s*:\\s*(always|page)/i.test(s); }
  function forcedAfter(el) { var s = el.getAttribute('style') || ''; return /(page-break-after|break-after)\\s*:\\s*(always|page)/i.test(s); }
  function layout() {
    var body = document.body, html = document.documentElement;
    if (!body) return;
    var old = document.getElementById(ID); if (old) old.remove();
    body.style.transform = '';
    var shifted = body.querySelectorAll('[data-cpd-pv]');
    for (var i = 0; i < shifted.length; i++) { shifted[i].style.marginTop = shifted[i].getAttribute('data-cpd-pv'); shifted[i].removeAttribute('data-cpd-pv'); }
    var stride = G.height + GAP, contentH = G.height - G.top - G.bottom;
    var list = units(body, []);
    // The body's top edge is where the content of the first page starts; page coordinates run from G.top above it.
    var y0 = body.getBoundingClientRect().top + window.scrollY;
    var carry = false, lastBottom = 0;
    for (var n = 0; n < list.length; n++) {
      var el = list[n], r = el.getBoundingClientRect();
      var top = r.top + window.scrollY - y0 + G.top, h = r.height;
      if (h === 0 && !forcedBefore(el) && !forcedAfter(el)) continue;
      var page = Math.floor(top / stride), limit = page * stride + G.height - G.bottom, need = 0;
      var keep = h;
      if (/^H[1-6]$/.test(el.tagName) && list[n + 1]) keep = h + Math.min(list[n + 1].getBoundingClientRect().height, contentH / 3);
      if (carry || forcedBefore(el)) need = (page + 1) * stride + G.top - top;
      else if (top < page * stride + G.top) need = page * stride + G.top - top;
      else if (top >= limit || (top + keep > limit && keep <= contentH)) need = (page + 1) * stride + G.top - top;
      carry = false;
      if (need > 0) {
        if (!el.hasAttribute('data-cpd-pv')) el.setAttribute('data-cpd-pv', el.style.marginTop || '');
        el.style.marginTop = (parseFloat(getComputedStyle(el).marginTop) + need) + 'px';
        top += need;
      }
      lastBottom = top + h;
      if (forcedAfter(el)) carry = true;
    }
    var pages = Math.max(1, Math.floor(Math.max(0, lastBottom - 1) / stride) + 1);
    var total = (pages - 1) * stride + G.height;
    var host = document.createElement('div');
    host.id = ID;
    host.setAttribute('aria-hidden', 'true');
    host.style.cssText = 'position:absolute;left:' + (-G.left) + 'px;top:' + (-G.top) + 'px;width:' + G.width + 'px;height:' + total + 'px;pointer-events:none;z-index:-1;';
    for (var p = 0; p < pages; p++) {
      var sheet = document.createElement('div');
      sheet.style.cssText = 'position:absolute;left:0;top:' + (p * stride) + 'px;width:' + G.width + 'px;height:' + G.height + 'px;background:#fff;box-shadow:0 1px 4px rgba(0,0,0,.35);box-sizing:border-box;';
      var box = document.createElement('div');
      box.style.cssText = 'position:absolute;left:' + G.left + 'px;top:' + G.top + 'px;right:' + G.right + 'px;bottom:' + G.bottom + 'px;border:1px dashed rgba(120,130,140,.28);';
      var tag = document.createElement('div');
      tag.textContent = LABEL.replace('{n}', String(p + 1)).replace('{total}', String(pages));
      tag.style.cssText = 'position:absolute;right:' + G.right + 'px;bottom:' + Math.max(4, G.bottom / 2 - 7) + 'px;font:11px/14px system-ui,sans-serif;color:#80868b;';
      sheet.appendChild(box); sheet.appendChild(tag); host.appendChild(sheet);
    }
    body.appendChild(host);
    // Fit the page to the width of the pane, never larger than life.
    var avail = Math.max(240, html.clientWidth - 24), scale = Math.min(1, avail / G.width);
    body.style.transformOrigin = '0 0';
    body.style.transform = scale < 1 ? 'scale(' + scale + ')' : '';
    body.style.setProperty('margin-left', Math.max(12 + G.left * scale, (html.clientWidth - G.width * scale) / 2 + G.left * scale) + 'px', 'important');
    body.style.setProperty('margin-top', (GAP * scale + G.top * scale) + 'px', 'important');
    html.style.minHeight = ((total + GAP * 2) * scale + G.top * scale) + 'px';
    window.__cpdPages = pages;
    try { window.parent.postMessage({ type: 'pageViewInfo', pages: pages, scale: scale }, '*'); } catch (e) {}
  }
  function later() { clearTimeout(timer); timer = setTimeout(layout, 60); }
  var ready = function() { layout(); if (document.fonts && document.fonts.ready) document.fonts.ready.then(later); };
  if (document.readyState === 'complete') ready(); else window.addEventListener('load', ready);
  window.addEventListener('resize', later);
  document.addEventListener('load', later, true);
})();`;
}

/** The style of the page view: grey desk, the report as it prints (dark text), paper width set by the script's margins. */
function style(g: PageGeometry): string {
    const content = Math.max(120, g.width - g.left - g.right);
    return `<style data-cpd-page-view>`
        + `html{background:#dfe2e6 !important;color-scheme:light}`
        + `body{position:relative !important;box-sizing:content-box !important;width:${content}px !important;max-width:none !important;`
        + `margin:0 !important;padding:0 !important;background:transparent !important;color:#000 !important;min-height:0 !important}`
        + `</style>`;
}

/** `html` with the page view's style and script added to its head (or in front, when it has none). */
export function withPageView(html: string, settings: PageViewSettings = {}, label = 'Page {n} of {total}'): string {
    const g = pageGeometry(settings);
    const add = style(g) + `<script>${script(g, label).replace(/<\/script/gi, '<\\/script')}</script>`;
    const i = html.search(/<\/head>/i);
    return i >= 0 ? html.slice(0, i) + add + html.slice(i) : add + html;
}
