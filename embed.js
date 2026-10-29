/* CivilKit Calcs embed loader (MIT). https://civilkit.io/calcs/embed/
   <div data-civilkit-sample="template-simply-supported-beam" data-height="560" data-theme="auto"></div>
   <script src="https://civilkit.io/calcs/embed.js" async></script>
   Each such div becomes an iframe of the sample; the iframe then grows or shrinks to fit its content. Messages from
   the embed (civilkit-results and the others) are also fired on the iframe as a "civilkit" event (detail = the message).
   data-origin and data-path point at a self-hosted copy (and the tests); messages are accepted only from that origin. */
(function () {
  var ORIGIN = 'https://civilkit.io', frames = [];
  function attr(el, n, d) { var v = el.getAttribute('data-' + n); return v == null || v === '' ? d : v; }
  function scan() {
    var divs = document.querySelectorAll('div[data-civilkit-sample]');
    for (var i = 0; i < divs.length; i++) {
      var div = divs[i], id = attr(div, 'civilkit-sample', '');
      if (!/^[\w.-]{1,100}$/.test(id)) continue;
      var origin = attr(div, 'origin', ORIGIN).replace(/\/+$/, '');
      var q = '?embed=1&sample=' + encodeURIComponent(id);
      ['theme', 'mode', 'hide'].forEach(function (k) { var v = attr(div, k, ''); if (v) q += '&' + k + '=' + encodeURIComponent(v); });
      var h = parseInt(attr(div, 'height', '560'), 10) || 560;
      var f = document.createElement('iframe');
      f.src = origin + attr(div, 'path', '/calcs/') + q;
      f.title = attr(div, 'title', 'CivilKit Calcs: ' + id);
      f.width = '100%';
      f.height = String(h);
      f.loading = 'lazy';
      f.setAttribute('sandbox', 'allow-scripts allow-same-origin allow-popups allow-popups-to-escape-sandbox');
      f.setAttribute('allow', '');
      f.style.cssText = 'display:block;width:100%;height:' + h + 'px;border:1px solid #ccc;border-radius:6px';
      f.setAttribute('data-civilkit-sample', id);
      if (div.id) f.id = div.id;
      if (div.className) f.className = div.className;
      div.parentNode.replaceChild(f, div);
      frames.push({ el: f, origin: origin });
    }
  }
  window.addEventListener('message', function (e) {
    for (var i = 0; i < frames.length; i++) {
      var fr = frames[i];
      if (e.source !== fr.el.contentWindow || e.origin !== fr.origin) continue;
      var d = e.data;
      if (!d || typeof d.type !== 'string' || d.type.indexOf('civilkit-') !== 0) return;
      if (d.type === 'civilkit-embed-height') {
        var h = Number(d.height);
        if (isFinite(h) && h >= 60 && h <= 20000) fr.el.style.height = Math.ceil(h) + 2 + 'px';
      }
      try { fr.el.dispatchEvent(new CustomEvent('civilkit', { detail: d, bubbles: true })); } catch (x) { /* old browser */ }
      return;
    }
  });
  window.CivilKitEmbed = { scan: scan };
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', scan); else scan();
})();
