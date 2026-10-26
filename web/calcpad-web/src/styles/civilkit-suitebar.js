// Vendored from civilkit-site/tokens/civilkit-suitebar.js (CivilKit theme v1), copied 2026-10-26. Do not edit here.
// CivilKit suite bar and app switcher: two small custom elements, no dependencies, work in plain JS, Preact and Vue.
//   <ck-suitebar current="calcs"></ck-suitebar>      a full 36px bar: mark, name, links to the other apps
//   <ck-appswitcher current="studio"></ck-appswitcher>  just the mark with a menu, for an app that has its own top bar
// Colours come from civilkit-tokens.css (--ck-*); fallbacks keep it readable without it.
// Attributes: current (calcs | cad | studio), calcs-href, cad-href, studio-href.
const APPS = [
  { id: 'calcs', name: 'Calcs', href: '/calcs/' },
  { id: 'cad', name: 'CAD', href: '/cad/' },
  { id: 'studio', name: 'Studio', href: 'https://civilkit-studio-demo.pages.dev/' },
];
const MARK = '<svg viewBox="0 0 1024 1024" width="20" height="20" aria-hidden="true"><defs><linearGradient id="ckg" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#2D8CF0"/><stop offset="1" stop-color="#0A4FA8"/></linearGradient></defs><rect width="1024" height="1024" rx="230" fill="url(#ckg)"/><g transform="translate(0 150)"><path d="M200 420 C340 565 684 565 824 420" fill="none" stroke="#fff" stroke-width="60" stroke-linecap="round"/><path d="M200 458 L126 568 L274 568 Z M824 458 L750 568 L898 568 Z" fill="#fff" stroke="#fff" stroke-width="18" stroke-linejoin="round"/><line x1="512" y1="100" x2="512" y2="350" stroke="#FFB81C" stroke-width="58" stroke-linecap="round"/><path d="M512 468 L426 346 L598 346 Z" fill="#FFB81C" stroke="#FFB81C" stroke-width="26" stroke-linejoin="round"/></g></svg>';
const CSS = `
:host{display:block;font:var(--ck-text-m,13px)/1.2 var(--ck-font,system-ui,sans-serif)}
.bar{display:flex;align-items:center;gap:4px;height:var(--ck-bar-h,36px);padding:0 10px;background:var(--ck-chrome,#1b1f27);color:var(--ck-chrome-text,#e9ecf1);border-bottom:1px solid var(--ck-line,#2a3038);box-sizing:border-box}
.brand{display:flex;align-items:center;gap:8px;font-weight:700;letter-spacing:-.01em;margin-right:10px;color:inherit;text-decoration:none}
a.app{color:inherit;opacity:.72;text-decoration:none;padding:5px 10px;border-radius:var(--ck-radius,6px);font-weight:600}
a.app:hover{opacity:1;background:rgba(255,255,255,.08)}
a.app[aria-current=page]{opacity:1;background:rgba(255,255,255,.14)}
a:focus-visible,button:focus-visible{outline:2px solid var(--ck-focus,#4da3ff);outline-offset:1px}
button.sw{display:flex;align-items:center;gap:6px;border:0;background:transparent;color:inherit;font:inherit;font-weight:700;padding:4px 6px;border-radius:var(--ck-radius,6px);cursor:pointer}
button.sw:hover{background:rgba(255,255,255,.1)}
.menu{position:fixed;z-index:3000;min-width:170px;padding:6px;border-radius:var(--ck-radius-l,10px);background:var(--ck-raised,#1b1f27);color:var(--ck-text,#e7ebf2);border:1px solid var(--ck-line,#2a3038);box-shadow:var(--ck-shadow,0 8px 24px rgba(0,0,0,.4))}
.menu a{display:block;color:inherit;text-decoration:none;padding:8px 10px;border-radius:var(--ck-radius-s,4px)}
.menu a:hover{background:var(--ck-accent-soft,rgba(77,163,255,.16))}
.menu a[aria-current=page]{color:var(--ck-accent,#4da3ff);font-weight:700}
.menu[hidden]{display:none}`;
function hrefOf(el, a) { return el.getAttribute(a.id + '-href') || a.href; }
class SuiteBar extends HTMLElement {
  connectedCallback() {
    const cur = this.getAttribute('current') || '';
    const root = this.attachShadow({ mode: 'open' });
    const links = APPS.map(a => `<a class="app" href="${hrefOf(this, a)}"${a.id === cur ? ' aria-current="page"' : ''}>${a.name}</a>`).join('');
    root.innerHTML = `<style>${CSS}</style><div class="bar" role="navigation" aria-label="CivilKit apps"><a class="brand" href="/">${MARK}<span>CivilKit</span></a>${links}</div>`;
  }
}
class AppSwitcher extends HTMLElement {
  connectedCallback() {
    const cur = this.getAttribute('current') || '';
    const root = this.attachShadow({ mode: 'open' });
    const items = APPS.map(a => `<a href="${hrefOf(this, a)}"${a.id === cur ? ' aria-current="page"' : ''}>CivilKit ${a.name}</a>`).join('');
    root.innerHTML = `<style>${CSS}</style><button class="sw" aria-haspopup="true" aria-expanded="false" title="CivilKit apps">${MARK}</button><div class="menu" role="menu" hidden>${items}</div>`;
    const btn = root.querySelector('button'), menu = root.querySelector('.menu');
    const close = () => { menu.hidden = true; btn.setAttribute('aria-expanded', 'false'); };
    btn.addEventListener('click', e => {
      e.stopPropagation();
      const r = btn.getBoundingClientRect();
      menu.style.left = Math.max(8, r.left) + 'px'; menu.style.top = r.bottom + 6 + 'px';
      menu.hidden = !menu.hidden; btn.setAttribute('aria-expanded', String(!menu.hidden));
    });
    document.addEventListener('click', close);
    this.addEventListener('keydown', e => { if (e.key === 'Escape') { close(); btn.focus(); } });
  }
}
if (!customElements.get('ck-suitebar')) customElements.define('ck-suitebar', SuiteBar);
if (!customElements.get('ck-appswitcher')) customElements.define('ck-appswitcher', AppSwitcher);
