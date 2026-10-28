// calcpad-rs: a slider under a number input of a sheet (`L: ? 6 m slider 2..12 | span`, docs/sheet-syntax.md).
//
// The sheet compiler leaves a hidden `span.sheet-range` (data-min, data-max, data-step) after the note of such a line.
// This script, run in the Input mode frame, turns each marker into a range control that shares the number box: the box
// stays the source of truth (its own change handler posts the value), the slider only moves it and fires that change.

/** ES5 for the preview frame, run where the other injected scripts are. No "</script", no template placeholders. */
export const INPUT_SLIDER_SCRIPT = String.raw`
(function() {
  var marks = document.querySelectorAll('.sheet-range');
  if (!marks.length) return;
  var DEBOUNCE_MS = 200;
  var css = '.ck-slider{display:block;margin:2px 0 6px;max-width:24em}'
    + '.ck-slider input{width:100%;margin:0;min-height:20px;cursor:pointer;accent-color:#0a6bcf;touch-action:pan-y}'
    + '@media (hover:none){.ck-slider input{min-height:36px}}'
    + '@media print{.ck-slider{display:none}}';
  var st = document.createElement('style'); st.textContent = css; document.head.appendChild(st);
  function decimals(t) { var m = /\.(\d+)/.exec(String(t)); return m ? m[1].length : 0; }
  Array.prototype.forEach.call(marks, function(mark) {
    var line = mark.closest('[data-ui-var]');
    var box = line ? line.querySelector('input.calcpad-ui-input') : null;
    var lo = parseFloat(mark.getAttribute('data-min')), hi = parseFloat(mark.getAttribute('data-max'));
    if (!box || !(lo < hi)) return;
    var stepText = mark.getAttribute('data-step');
    // Without a step: a power of ten near a hundredth of the range, so a drag has about 100 to 1000 stops.
    var step = stepText ? parseFloat(stepText) : Math.pow(10, Math.floor(Math.log(((hi - lo) / 100)) / Math.LN10));
    var places = Math.max(decimals(stepText || String(step)), decimals(mark.getAttribute('data-min')));
    var note = line.querySelector('.sheet-note');
    var nameEl = line.querySelector('var');
    var label = ((note && note.textContent.trim()) || (nameEl && nameEl.textContent.trim()) || 'value') + ' slider';
    var unit = '';
    var eq = box.closest('.eq');
    if (eq) {
      var dvc = eq.querySelector('.dvc');
      if (dvc) unit = Array.prototype.map.call(dvc.querySelectorAll('i'), function(i) { return i.textContent; }).join('/');
      else { var u = eq.querySelector('i'); unit = u ? u.textContent : ''; }
    }
    var wrap = document.createElement('div'); wrap.className = 'ck-slider';
    var r = document.createElement('input');
    r.type = 'range'; r.min = String(lo); r.max = String(hi); r.step = String(step);
    r.setAttribute('aria-label', label + (unit ? ' (' + unit + ')' : ''));
    function show(v) { return String(parseFloat(Number(v).toFixed(places))); }
    function text() { r.setAttribute('aria-valuetext', box.value + (unit ? ' ' + unit : '')); }
    function fromBox() { var v = parseFloat(box.value); if (isFinite(v)) r.value = String(Math.min(hi, Math.max(lo, v))); text(); }
    fromBox();
    var timer = null;
    function commit() {
      if (timer) { clearTimeout(timer); timer = null; }
      box.dispatchEvent(new Event('change', { bubbles: true }));
    }
    r.addEventListener('input', function() {
      box.value = show(r.value); text();
      if (timer) clearTimeout(timer);
      timer = setTimeout(commit, DEBOUNCE_MS);
    });
    r.addEventListener('change', function() { box.value = show(r.value); text(); commit(); });
    box.addEventListener('input', fromBox);
    box.addEventListener('change', fromBox);
    wrap.appendChild(r);
    line.appendChild(wrap);
  });
})();
`;
