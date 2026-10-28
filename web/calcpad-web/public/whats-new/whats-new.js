// Hover to play: a clip shows its poster until the pointer is over it (or it has keyboard focus, or is tapped on a
// touch screen), plays while it is, and goes back to the poster when it leaves. Without this script the posters show.
(function () {
  if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) return;
  var hover = window.matchMedia('(hover: hover) and (pointer: fine)').matches;
  var figures = document.querySelectorAll('figure.clip');
  Array.prototype.forEach.call(figures, function (fig) {
    var video = fig.querySelector('video');
    var frame = fig.querySelector('.frame');
    if (!video || !frame) return;
    function play() {
      frame.classList.add('playing');
      var p = video.play();
      if (p && p.catch) p.catch(function () { frame.classList.remove('playing'); });
    }
    function stop() {
      frame.classList.remove('playing');
      video.pause();
      try { video.currentTime = 0; } catch (e) { /* not loaded yet */ }
    }
    function ready() { frame.classList.add('can-play'); }
    if (video.readyState >= 1) ready(); else video.addEventListener('loadedmetadata', ready);
    if (hover) {
      frame.addEventListener('mouseenter', play);
      frame.addEventListener('mouseleave', stop);
    } else {
      frame.addEventListener('click', function () { if (frame.classList.contains('playing')) stop(); else play(); });
    }
    frame.addEventListener('focus', play);
    frame.addEventListener('blur', stop);
  });
})();
