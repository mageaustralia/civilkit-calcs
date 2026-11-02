// Click to enlarge for videos: the What's new clips (figure.clip .frame) and the guide's chapter videos (main video[controls]).
// One shared file, no dependencies, no network. A click, tap, Enter or Space on a clip opens it in a lightbox at its own
// pixel size (never larger; smaller only to fit the window), with the browser's own video controls, the title and caption,
// and Esc, a click outside or the Close button to dismiss. Left and right arrows (and the on-screen buttons) step to the
// previous or next clip on the page. Focus is trapped while it is open and returned to the clip afterwards.
(function () {
  'use strict';
  var reduce = window.matchMedia ? window.matchMedia('(prefers-reduced-motion: reduce)') : { matches: false };
  var SVG = 'http://www.w3.org/2000/svg';
  var items = [];
  var ui = null;       // the open lightbox: { back, ... }
  var index = -1;
  var restore = [];    // [element, wasInert] for the page behind the lightbox
  var prevOverflow = '';

  function el(tag, cls, text) {
    var e = document.createElement(tag);
    if (cls) e.className = cls;
    if (text != null) e.textContent = text;
    return e;
  }
  function icon(d, size) {
    var s = document.createElementNS(SVG, 'svg');
    s.setAttribute('viewBox', '0 0 24 24');
    s.setAttribute('width', size || 18);
    s.setAttribute('height', size || 18);
    s.setAttribute('aria-hidden', 'true');
    s.setAttribute('focusable', 'false');
    var p = document.createElementNS(SVG, 'path');
    p.setAttribute('d', d);
    p.setAttribute('fill', 'none');
    p.setAttribute('stroke', 'currentColor');
    p.setAttribute('stroke-width', '2');
    p.setAttribute('stroke-linecap', 'round');
    p.setAttribute('stroke-linejoin', 'round');
    s.appendChild(p);
    return s;
  }
  var EXPAND = 'M4 9V4h5M20 9V4h-5M4 15v5h5M20 15v5h-5';
  function button(cls, label, d, text) {
    var b = el('button', 'vz-btn ' + cls);
    b.type = 'button';
    b.setAttribute('aria-label', label);
    b.appendChild(icon(d, 20));
    if (text) b.appendChild(el('span', 'vz-btn-text', text));
    return b;
  }
  function order(a, b) { return a.trigger === b.trigger ? 0 : (a.trigger.compareDocumentPosition(b.trigger) & 4 ? -1 : 1); }

  function add(trigger, def) {
    var it = { trigger: trigger, video: def.video, title: def.title, caption: def.caption || '', w: def.w || 0, h: def.h || 0 };
    items.push(it);
    items.sort(order);
    trigger.addEventListener('click', function (e) { e.preventDefault(); open(items.indexOf(it)); });
    return it;
  }

  // ---- the lightbox ------------------------------------------------------------------------------------------
  function build() {
    var back = el('div', 'vz-back');
    back.setAttribute('role', 'dialog');
    back.setAttribute('aria-modal', 'true');
    back.setAttribute('aria-labelledby', 'vz-title');
    var start = el('span', 'vz-sentinel'); start.tabIndex = 0;
    var end = el('span', 'vz-sentinel'); end.tabIndex = 0;
    var box = el('div', 'vz-box');
    var bar = el('div', 'vz-bar');
    var count = el('span', 'vz-count');
    var title = el('h2', 'vz-title'); title.id = 'vz-title';
    title.setAttribute('aria-live', 'polite');
    bar.appendChild(title); bar.appendChild(count);
    var stage = el('div', 'vz-stage');
    var prev = button('vz-prev', 'Previous clip', 'M15 5l-7 7 7 7');
    var next = button('vz-next', 'Next clip', 'M9 5l7 7-7 7');
    var holder = el('div', 'vz-holder');
    stage.appendChild(prev); stage.appendChild(holder); stage.appendChild(next);
    var note = el('p', 'vz-note'); note.hidden = true;
    var cap = el('p', 'vz-caption');
    var close = button('vz-close', 'Close', 'M6 6l12 12M18 6L6 18', 'Close');
    box.appendChild(bar); box.appendChild(stage); box.appendChild(note); box.appendChild(cap); box.appendChild(close);
    back.appendChild(start); back.appendChild(box); back.appendChild(end);
    ui = { back: back, box: box, title: title, count: count, holder: holder, prev: prev, next: next, close: close, cap: cap, note: note, start: start, end: end, video: null };

    close.addEventListener('click', shut);
    prev.addEventListener('click', function () { step(-1); });
    next.addEventListener('click', function () { step(1); });
    // A click (or tap) on the backdrop or the empty parts of the box closes it; the video, buttons and text do not.
    back.addEventListener('click', function (e) {
      if (e.target === back || e.target === box || e.target === stage || e.target === holder || e.target === bar) shut();
    });
    start.addEventListener('focus', function () { focusables().pop().focus(); });
    end.addEventListener('focus', function () { focusables()[0].focus(); });
  }
  function focusables() {
    var l = [ui.prev, ui.holder.querySelector('video'), ui.next, ui.close].filter(function (x) { return x && !x.disabled; });
    return l.length ? l : [ui.close];
  }

  function show(i) {
    var it = items[i];
    index = i;
    ui.title.textContent = it.title;
    ui.count.textContent = items.length > 1 ? (i + 1) + ' of ' + items.length : '';
    ui.cap.textContent = it.caption;
    ui.cap.hidden = !it.caption;
    ui.note.hidden = true;
    ui.prev.disabled = i <= 0;
    ui.next.disabled = i >= items.length - 1;
    ui.prev.hidden = ui.next.hidden = items.length < 2;
    if (ui.video) dispose(ui.video);
    var src = it.video;
    var v = document.createElement('video');
    v.className = 'vz-video';
    v.controls = true;
    v.playsInline = true;
    v.preload = 'auto';
    v.loop = src.loop;
    v.muted = src.muted;
    if (src.getAttribute('poster')) v.poster = src.poster;
    v.setAttribute('aria-label', it.title);
    // Native size: the width the file is recorded at, so the box never scales a clip past its own pixels.
    function size(w, h) {
      if (w && h) {
        ui.holder.style.setProperty('--vz-w', w + 'px');
        ui.holder.style.setProperty('--vz-ar', w + ' / ' + h);
        ui.holder.style.setProperty('--vz-r', String(w / h));
      }
    }
    size(it.w || 1600, it.h || 900);
    v.addEventListener('loadedmetadata', function () { size(v.videoWidth, v.videoHeight); });
    v.addEventListener('error', function () { ui.note.textContent = 'This clip could not be loaded.'; ui.note.hidden = false; });
    var cur = src.currentSrc;
    if (cur) v.src = cur;
    else {
      var ss = src.querySelectorAll('source');
      if (ss.length) Array.prototype.forEach.call(ss, function (s) { v.appendChild(s.cloneNode(false)); });
      else if (src.getAttribute('src')) v.src = src.src;
    }
    var t = src.loop ? 0 : src.currentTime;
    if (t > 0) v.addEventListener('loadedmetadata', function () { try { v.currentTime = t; } catch (e) { /* not seekable */ } });
    try { src.pause(); } catch (e) { /* ignore */ }
    ui.holder.appendChild(v);
    ui.video = v;
    if (!reduce.matches) { var p; try { p = v.play(); } catch (e) { p = null; } if (p && p.catch) p.catch(function () {}); }
  }
  function dispose(v) {
    try { v.pause(); } catch (e) { /* ignore */ }
    v.removeAttribute('src');
    while (v.firstChild) v.removeChild(v.firstChild);
    try { v.load(); } catch (e) { /* ignore */ }
    if (v.parentNode) v.parentNode.removeChild(v);
  }
  function step(d) {
    var j = index + d;
    if (j < 0 || j >= items.length) return;
    var hadFocus = ui.back.contains(document.activeElement) && document.activeElement !== ui.prev && document.activeElement !== ui.next;
    show(j);
    if (!hadFocus || !document.activeElement || document.activeElement === document.body || !ui.back.contains(document.activeElement)) ui.close.focus();
  }

  function onKey(e) {
    if (!ui) return;
    if (e.key === 'Escape' || e.key === 'Esc') { e.preventDefault(); shut(); }
    else if ((e.key === 'ArrowLeft' || e.key === 'ArrowRight') && !e.altKey && !e.ctrlKey && !e.metaKey && !e.shiftKey) {
      e.preventDefault();
      step(e.key === 'ArrowLeft' ? -1 : 1);
    }
  }
  function onFocusIn(e) {
    if (ui && !ui.back.contains(e.target)) focusables()[0].focus();
  }

  var opener = null;
  function open(i) {
    if (ui || i < 0) return;
    opener = items[i];
    build();
    var it = items[i];
    it.trigger.dispatchEvent(new CustomEvent('vz-open', { bubbles: true }));
    // The page behind is inert so assistive technology and Tab stay inside the lightbox.
    restore = [];
    Array.prototype.forEach.call(document.body.children, function (c) {
      restore.push([c, c.inert]);
      try { c.inert = true; } catch (e) { /* ignore */ }
    });
    document.body.appendChild(ui.back);
    restore = restore.filter(function (r) { return r[0] !== ui.back; });
    ui.back.inert = false;
    prevOverflow = document.documentElement.style.overflow;
    document.documentElement.style.overflow = 'hidden';
    show(i);
    document.addEventListener('keydown', onKey, true);
    document.addEventListener('focusin', onFocusIn);
    ui.close.focus();
  }
  function shut() {
    if (!ui) return;
    var cur = items[index] || opener;
    if (ui.video) dispose(ui.video);
    document.removeEventListener('keydown', onKey, true);
    document.removeEventListener('focusin', onFocusIn);
    if (ui.back.parentNode) ui.back.parentNode.removeChild(ui.back);
    restore.forEach(function (r) { try { r[0].inert = r[1]; } catch (e) { /* ignore */ } });
    restore = [];
    document.documentElement.style.overflow = prevOverflow;
    ui = null;
    index = -1;
    if (cur && cur.trigger.isConnected) cur.trigger.focus();
  }

  // ---- finding the clips -------------------------------------------------------------------------------------
  function init() {
    // What's new: the frame around each clip is the button.
    Array.prototype.forEach.call(document.querySelectorAll('figure.clip'), function (fig) {
      var frame = fig.querySelector('.frame'), video = fig.querySelector('video');
      if (!frame || !video) return;
      var h3 = fig.querySelector('h3'), cap = fig.querySelector('figcaption');
      var title = h3 ? h3.textContent.trim() : 'Clip';
      frame.setAttribute('role', 'button');
      frame.setAttribute('aria-label', 'Enlarge: ' + title);
      frame.classList.add('vz-ready');
      var cue = el('span', 'vz-cue');
      cue.setAttribute('aria-hidden', 'true');
      cue.appendChild(icon(EXPAND, 16));
      frame.appendChild(cue);
      add(frame, { video: video, title: title, caption: cap ? cap.textContent.trim() : '', w: Number(video.getAttribute('width')), h: Number(video.getAttribute('height')) });
      frame.addEventListener('keydown', function (e) {
        if (e.key === 'Enter' || e.key === ' ' || e.key === 'Spacebar') { e.preventDefault(); e.stopPropagation(); open(items.findIndex(function (x) { return x.trigger === frame; })); }
      }, true);
    });
    // The guide: the chapter's video keeps its own controls and gets an Enlarge button over its corner.
    var h1 = document.querySelector('main h1');
    Array.prototype.forEach.call(document.querySelectorAll('main video[controls]'), function (video) {
      var title = (h1 ? h1.textContent.trim() : 'Video');
      var wrap = el('span', 'vz-wrap');
      video.parentNode.insertBefore(wrap, video);
      wrap.appendChild(video);
      var b = button('vz-enlarge', 'Enlarge: ' + title, EXPAND, 'Enlarge');
      wrap.appendChild(b);
      add(b, { video: video, title: title, caption: '' });
    });
  }
  window.CivilKitZoom = { add: add, open: function (i) { open(i); }, close: shut, count: function () { return items.length; } };
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', init); else init();
})();
