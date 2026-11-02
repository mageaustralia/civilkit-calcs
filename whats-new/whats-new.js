// Hover to play: a clip shows its poster until a mouse is over it (or it has keyboard focus, or is tapped on a touch
// screen), plays while it is, and goes back to the poster when it leaves. Only one clip plays at a time. Without this
// script the posters show. Clicking a clip enlarges it (../zoom/video-zoom.js, shared with the guide).
(function () {
  var reduce = window.matchMedia ? window.matchMedia('(prefers-reduced-motion: reduce)') : { matches: false };
  var current = null;        // the clip playing now: { frame, stop }
  var pointerType = '';      // type of the latest pointerdown, so a tap is not mistaken for keyboard focus
  var pointerAt = 0;
  document.addEventListener('pointerdown', function (e) { pointerType = e.pointerType || 'mouse'; pointerAt = Date.now(); }, true);
  var fromPointer = function () { return Date.now() - pointerAt < 800; };
  var isMouse = function (e) { return e.pointerType === 'mouse' || e.pointerType === 'pen'; };

  function setup(fig) {
    var video = fig.querySelector('video');
    var frame = fig.querySelector('.frame');
    if (!video || !frame) return;
    video.muted = true;
    video.defaultMuted = true;
    var mp4 = fig.querySelector('source[type="video/mp4"]');
    var switched = false;
    var clip = { frame: frame, stop: stop };

    function ready() { frame.classList.add('can-play'); }
    function attempt() {
      var p;
      try { p = video.play(); } catch (e) { p = null; }
      if (p && p.catch) p.catch(function () { if (video.paused && current === clip) frame.classList.remove('playing'); });
    }
    function play() {
      if (reduce.matches) return;
      if (current && current !== clip) current.stop();
      current = clip;
      frame.classList.add('playing');
      if (video.error && !fallback()) { stop(); return; }
      if (video.networkState === 0 /* NETWORK_EMPTY: nothing loaded yet */) { try { video.load(); } catch (e) { /* ignore */ } }
      attempt();
    }
    function stop() {
      frame.classList.remove('playing');
      if (current === clip) current = null;
      try { video.pause(); } catch (e) { /* ignore */ }
      try { if (video.readyState > 0 && video.currentTime > 0) video.currentTime = 0; } catch (e) { /* not seekable yet */ }
    }
    function toggle() { if (current === clip) stop(); else play(); }
    // A file the browser accepts but cannot decode does not fall through to the next <source>: switch to the MP4 once.
    function fallback() {
      if (switched || !mp4 || !video.canPlayType('video/mp4; codecs="avc1.64001E"') || /\.mp4(\?|#|$)/.test(video.currentSrc)) return false;
      switched = true;
      video.src = mp4.src;
      try { video.load(); } catch (e) { /* ignore */ }
      return true;
    }

    if (video.readyState >= 1) ready(); else video.addEventListener('loadedmetadata', ready);
    video.addEventListener('error', function () {
      if (fallback()) { if (current === clip) attempt(); } else if (current === clip) stop();
    });

    frame.addEventListener('pointerenter', function (e) { if (isMouse(e)) play(); });
    frame.addEventListener('pointerleave', function (e) { if (isMouse(e) && current === clip) stop(); });
    // With the click-to-enlarge script (../zoom/video-zoom.js) a click, tap or Enter opens the clip in a lightbox instead,
    // and the preview stops while it is open. Without it a tap toggles the clip as before.
    frame.addEventListener('vz-open', function () { stop(); });
    frame.addEventListener('click', function (e) {
      if (window.CivilKitZoom) return;
      // A tap toggles; a mouse click starts a clip that is not playing (it is already playing under the pointer).
      if (fromPointer() && (pointerType === 'mouse' || pointerType === 'pen')) { if (current !== clip) play(); }
      else toggle();
      e.preventDefault();
    });
    frame.addEventListener('keydown', function (e) {
      if (window.CivilKitZoom) return;
      if (e.key === 'Enter' || e.key === ' ' || e.key === 'Spacebar') { e.preventDefault(); toggle(); }
    });
    frame.addEventListener('focus', function () { if (!fromPointer()) play(); });   // keyboard focus only
    frame.addEventListener('blur', function () { if (current === clip) stop(); });
    return clip;
  }

  var clips = [];
  Array.prototype.forEach.call(document.querySelectorAll('figure.clip'), function (fig) {
    var c = setup(fig);
    if (c) clips.push(c);
  });

  function stopCurrent() { if (current) current.stop(); }
  if (reduce.addEventListener) reduce.addEventListener('change', function () { if (reduce.matches) stopCurrent(); });
  else if (reduce.addListener) reduce.addListener(function () { if (reduce.matches) stopCurrent(); });
  document.addEventListener('visibilitychange', function () { if (document.hidden) stopCurrent(); });
  // A tapped clip scrolled out of view stops, so nothing plays unseen.
  if (window.IntersectionObserver) {
    var io = new IntersectionObserver(function (entries) {
      entries.forEach(function (en) { if (!en.isIntersecting && current && current.frame === en.target) current.stop(); });
    });
    clips.forEach(function (c) { io.observe(c.frame); });
  }
})();
