// calcpad-rs: a row of chips under each plot in the preview (log x, log y, 1:1, copy data, save PNG and SVG).
//
// The preview frame is sandboxed (allow-scripts only), so it can neither write the clipboard nor start a download:
// the script below builds the chips and posts a `plotChip` message, and plotChipHost carries it out on the page.
//
//  - Plotly figures (Python blocks): Log x, Log y and 1:1 relayout the figure; Copy data reads the traces; PNG and SVG
//    come from Plotly.toImage. The 3D scenes get Copy data and PNG only.
//  - Sheet `plot` lines (the engine's SVG): Copy data reads the curve back from the drawing through its axis labels
//    (so it is the plotted values to about four figures, not the engine's own numbers); Save SVG and PNG.

/** ES5 for the preview frame, run where FRAME_ID is in scope. No "</script", no template placeholders. */
export const PLOT_CHIPS_SCRIPT = String.raw`
(function() {
  var css = '.ck-chips{display:flex;flex-wrap:wrap;gap:6px;margin:2px 0 10px;font:12px/1.2 system-ui,sans-serif}'
    + '.ck-chip{border:1px solid currentColor;border-radius:999px;background:transparent;color:inherit;padding:3px 11px;cursor:pointer;opacity:.62;font:inherit}'
    + '.ck-chip:hover,.ck-chip:focus-visible{opacity:1}.ck-chip:focus-visible{outline:2px solid #3b82f6;outline-offset:1px}'
    + '.ck-chip[aria-pressed=true]{background:#0a6bcf;border-color:#0a6bcf;color:#fff;opacity:1}'
    + '@media (hover:none){.ck-chip{padding:7px 14px;font-size:13px}}'
    + '@media print{.ck-chips{display:none}}';
  var st = document.createElement('style'); st.textContent = css; document.head.appendChild(st);
  function say(m) { m.type = 'plotChip'; m.groupId = FRAME_ID; try { window.parent.postMessage(m, '*'); } catch (_e) {} }
  function chip(label, title, fn, toggle) {
    var b = document.createElement('button'); b.type = 'button'; b.className = 'ck-chip'; b.textContent = label; b.title = title;
    if (toggle) b.setAttribute('aria-pressed', 'false');
    b.addEventListener('click', function(e) { e.preventDefault(); e.stopPropagation(); fn(b); });
    return b;
  }
  function flash(b, text) { var o = b.getAttribute('data-label') || b.textContent; b.setAttribute('data-label', o); b.textContent = text; setTimeout(function() { b.textContent = o; }, 1300); }
  function row() { var d = document.createElement('div'); d.className = 'ck-chips'; return d; }
  function num(v) { return String(parseFloat(Number(v).toPrecision(7))); }

  // ---- Plotly figures
  var CARTESIAN = /^(scatter|scattergl|bar|histogram|box|violin|heatmap|contour|area)$/;
  function isGl3(t) { return /3d|surface|volume|cone|isosurface|streamtube/.test(t || ''); }
  function arr(a) { return a && a.length != null && typeof a !== 'string' ? Array.prototype.slice.call(a) : []; }
  function plotlyCsv(gd) {
    var out = ['series\tx\ty\tz'], d = gd.data || [], n = 0;
    for (var i = 0; i < d.length && n < 200000; i++) {
      var t = d[i], name = t.name || ('trace ' + (i + 1)), x = arr(t.x), y = arr(t.y), z = arr(t.z);
      if (z.length && Array.isArray(z[0])) {
        for (var r = 0; r < z.length; r++) for (var c = 0; c < z[r].length; c++) { out.push([name, x.length ? num(x[c]) : c, y.length ? num(y[r]) : r, num(z[r][c])].join('\t')); n++; }
      } else {
        var m = Math.max(x.length, y.length, z.length);
        for (var k = 0; k < m; k++) { out.push([name, x[k] == null ? '' : num(x[k]), y[k] == null ? '' : num(y[k]), z[k] == null ? '' : num(z[k])].join('\t')); n++; }
      }
    }
    return n ? out.join('\n') : '';
  }
  function plotlyChips(gd) {
    if (gd.__ckChips) return; gd.__ckChips = 1;
    var d = gd.data || [], flat = d.length > 0, three = false;
    for (var i = 0; i < d.length; i++) { if (isGl3(d[i].type)) three = true; if (d[i].type && !CARTESIAN.test(d[i].type)) flat = false; }
    var lay = gd.layout || {}, r = row();
    if (flat) {
      var lx = chip('Log x', 'Logarithmic x axis', function(b) { var on = b.getAttribute('aria-pressed') !== 'true'; b.setAttribute('aria-pressed', on); Plotly.relayout(gd, { 'xaxis.type': on ? 'log' : 'linear', 'xaxis.autorange': true }); }, true);
      var ly = chip('Log y', 'Logarithmic y axis', function(b) { var on = b.getAttribute('aria-pressed') !== 'true'; b.setAttribute('aria-pressed', on); Plotly.relayout(gd, { 'yaxis.type': on ? 'log' : 'linear', 'yaxis.autorange': true }); }, true);
      var eq = chip('1:1', 'Equal scale on both axes', function(b) {
        var on = b.getAttribute('aria-pressed') !== 'true'; b.setAttribute('aria-pressed', on);
        Plotly.relayout(gd, { 'yaxis.scaleanchor': on ? 'x' : false, 'yaxis.scaleratio': 1, 'xaxis.autorange': true, 'yaxis.autorange': true });
      }, true);
      if (lay.xaxis && lay.xaxis.type === 'log') lx.setAttribute('aria-pressed', 'true');
      if (lay.yaxis && lay.yaxis.type === 'log') ly.setAttribute('aria-pressed', 'true');
      if (lay.yaxis && lay.yaxis.scaleanchor === 'x') eq.setAttribute('aria-pressed', 'true');
      r.appendChild(lx); r.appendChild(ly); r.appendChild(eq);
    }
    r.appendChild(chip('Copy data', 'Copy the plotted values (tab separated, pastes into a spreadsheet)', function(b) {
      var t = plotlyCsv(gd); if (!t) { flash(b, 'No data'); return; } say({ action: 'copy', text: t }); flash(b, 'Copied');
    }));
    r.appendChild(chip('PNG', 'Save the chart as a PNG image', function(b) {
      Plotly.toImage(gd, { format: 'png', scale: 2 }).then(function(u) { say({ action: 'save', filename: 'plot.png', dataUrl: u }); flash(b, 'Saved'); }, function() { flash(b, 'Failed'); });
    }));
    if (!three) r.appendChild(chip('SVG', 'Save the chart as an SVG image', function(b) {
      Plotly.toImage(gd, { format: 'svg' }).then(function(u) { say({ action: 'save', filename: 'plot.svg', dataUrl: u }); flash(b, 'Saved'); }, function() { flash(b, 'Failed'); });
    }));
    (gd.parentNode || document.body).appendChild(r);
  }
  function watchPlotly() {
    var els = document.querySelectorAll('.cprs-plotly');
    for (var i = 0; i < els.length; i++) (function(el) {
      var tries = 0, t = setInterval(function() {
        tries++;
        if (el.data && el._fullLayout && typeof Plotly !== 'undefined') { clearInterval(t); plotlyChips(el); }
        else if (tries > 80) clearInterval(t);
      }, 250);
    })(els[i]);
  }

  // ---- Sheet plot lines: the engine's SVG
  var NUMBER = /^[\s−+\-]*[0-9]*\.?[0-9]+(e[+\-]?[0-9]+)?\s*$/i;
  function tickValue(t) { var s = (t || '').replace(/−/g, '-'); return NUMBER.test(s) ? parseFloat(s) : null; }
  function fit(p) {
    if (p.length < 2) return null;
    p.sort(function(a, b) { return a[0] - b[0]; });
    var a = p[0], b = p[p.length - 1];
    if (a[0] === b[0] || a[1] === b[1]) return null;
    return function(px) { return a[1] + (px - a[0]) * (b[1] - a[1]) / (b[0] - a[0]); };
  }
  function svgCsv(svg) {
    var xs = [], ys = [], ts = svg.querySelectorAll('text');
    for (var i = 0; i < ts.length; i++) {
      var cls = ts[i].getAttribute('class') || '', v = tickValue(ts[i].textContent);
      if (v === null) continue;
      if (/(^| )mid( |$)/.test(cls)) xs.push([parseFloat(ts[i].getAttribute('x')), v]);
      else if (/(^| )end( |$)/.test(cls)) ys.push([parseFloat(ts[i].getAttribute('y')) - 18, v]);
    }
    var fx = fit(xs), fy = fit(ys);
    if (!fx || !fy) return '';
    var out = ['series\tx\ty'], lines = svg.querySelectorAll('polyline');
    for (var j = 0; j < lines.length; j++) {
      var name = (lines[j].getAttribute('class') || 'Series').replace('Series', 'series '), pts = (lines[j].getAttribute('points') || '').split(/[\s]+/);
      for (var k = 0; k < pts.length; k++) {
        var xy = pts[k].split(','); if (xy.length < 2) continue;
        out.push(name + '\t' + num(fx(parseFloat(xy[0]))) + '\t' + num(fy(parseFloat(xy[1]))));
      }
    }
    return out.length > 1 ? out.join('\n') : '';
  }
  function svgText(svg) {
    var s = new XMLSerializer().serializeToString(svg);
    return s.indexOf('xmlns=') < 0 ? s.replace('<svg', '<svg xmlns="http://www.w3.org/2000/svg"') : s;
  }
  function svgChips(svg) {
    var r = row();
    r.appendChild(chip('Copy data', 'Copy the plotted values, read back from the drawing (tab separated)', function(b) {
      var t = svgCsv(svg); if (!t) { flash(b, 'No data'); return; } say({ action: 'copy', text: t }); flash(b, 'Copied');
    }));
    r.appendChild(chip('PNG', 'Save the plot as a PNG image', function(b) {
      var img = new Image();
      img.onload = function() {
        var vb = svg.viewBox && svg.viewBox.baseVal, w = vb && vb.width ? vb.width : img.width, h = vb && vb.height ? vb.height : img.height;
        var c = document.createElement('canvas'); c.width = w; c.height = h;
        var g = c.getContext('2d'); g.fillStyle = '#ffffff'; g.fillRect(0, 0, w, h); g.drawImage(img, 0, 0, w, h);
        try { say({ action: 'save', filename: 'plot.png', dataUrl: c.toDataURL('image/png') }); flash(b, 'Saved'); } catch (_e) { flash(b, 'Failed'); }
      };
      img.onerror = function() { flash(b, 'Failed'); };
      img.src = 'data:image/svg+xml;charset=utf-8,' + encodeURIComponent(svgText(svg));
    }));
    r.appendChild(chip('SVG', 'Save the plot as an SVG image', function(b) {
      say({ action: 'save', filename: 'plot.svg', dataUrl: 'data:image/svg+xml;charset=utf-8,' + encodeURIComponent(svgText(svg)) }); flash(b, 'Saved');
    }));
    svg.parentNode.insertBefore(r, svg.nextSibling);
  }
  var svgs = document.querySelectorAll('svg.plot[data-plot]');
  for (var q = 0; q < svgs.length; q++) svgChips(svgs[q]);
  watchPlotly();
})();
`;

const MAX_BYTES = 40 * 1024 * 1024;

async function copyText(text: string): Promise<void> {
    try {
        await navigator.clipboard.writeText(text);
        return;
    } catch {
        // No clipboard permission here: fall through to the selection route.
    }
    const ta = document.createElement('textarea');
    ta.value = text;
    ta.style.position = 'fixed';
    ta.style.opacity = '0';
    document.body.appendChild(ta);
    ta.select();
    document.execCommand('copy');
    ta.remove();
}

async function saveDataUrl(dataUrl: string, filename: string): Promise<void> {
    const blob = await (await fetch(dataUrl)).blob();
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    a.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
}

/** Carries out a `plotChip` message from a preview frame: copies the text or saves the image. */
export async function plotChipHost(data: { action?: unknown; text?: unknown; dataUrl?: unknown; filename?: unknown }): Promise<void> {
    if (data.action === 'copy' && typeof data.text === 'string' && data.text.length < MAX_BYTES) {
        await copyText(data.text);
    } else if (data.action === 'save' && typeof data.dataUrl === 'string' && /^data:image\/(png|svg\+xml)[;,]/.test(data.dataUrl) && data.dataUrl.length < MAX_BYTES) {
        const name = typeof data.filename === 'string' ? data.filename.replace(/[^\w. -]/g, '') : '';
        await saveDataUrl(data.dataUrl, name || 'plot.png');
    }
}
