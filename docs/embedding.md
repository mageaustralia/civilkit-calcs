# Embedding: design notes

The user-facing description is the guide chapter `book/src/embedding.md` (served at `/calcs/guide/embedding`) and the
catalogue page `/calcs/embed/`. This file records how the embed is built and why.

## What it is

`/calcs/?embed=1&sample=<id>` shows one sample of `public/sheet-samples/manifest.json` (or `public/samples/manifest.json`)
as an input form and its live results, for an iframe on another site or on our own landing pages. Options: `theme`
(`light`, `dark`, `auto`), `mode` (`input`, `results`, `both`), `hide=title`, `embed=edit` (a text editor below) and
`allowpython=1`.

## How it is built

* **A separate entry.** `index.html` loads `src/boot.ts`, which imports `src/embed/embed.ts` when `embed` is set and
  `src/main.ts` otherwise. The embed never loads Monaco, the ribbon, the tabs, the suite bar or the update notice, and it
  never registers the service worker. The two dynamic imports live in two exported functions: written as one if/else,
  the bundler merged their preload lists and an embed preloaded the whole app (about 1 MB gzipped). The embed's style
  sheets are imported `?inline` for the same reason. The embed chunk is about 20 KB (8 KB gzipped, its styles included) plus the shared
  settings chunk and the wasm.
* **The same calculation path.** `installWasmTransport()` answers `/api/calcpad/convert` in the page exactly as for the
  app's preview: sheet compile, engine in a worker, TeX, Mermaid, the dark theme styles, the script policy. The results
  page goes into a sandboxed `srcdoc` iframe (`allow-scripts allow-popups allow-popups-to-escape-sandbox`, as the
  samples browser's preview) with a small script that reports its height, so the frame grows to its content and the
  embed's own height is the whole story.
* **Its own form.** The form is drawn from the `#UI` lines of the compiled sheet (`src/embed/inputs.ts`): the sheet
  compiler already turns `name: ? ...` lines into `#UI {json} name = value 'note`, which carries the type, the options and
  the slider bounds in a form that is easy to read. Entered values go to the engine as `uiOverrides`, the mechanism the
  app's Input mode uses, so the report shows them exactly as the app would. Drawing our own form (rather than the
  engine's Input-mode page) gives one accessible, compact form at 320 px and lets the postMessage API check values
  against the same parsed bounds and options.
* **Sheets without inputs.** A sheet with no `?` line gets its top-level number lines (`L: 9.8 m | span`, outside
  fences) turned into inputs before the compile, so the worked examples (gable frame, shear frame, principal
  stresses) can be tried. A line with an expression, a vector or a conversion is left alone.
* **Results for the host.** After each calculation, when framed, the embed asks the transport's `values` endpoint
  (one more engine run with the variables recorded) and posts `{type:'civilkit-results', values:{name:{value, unit,
  text}}}`. It is only done when framed, so a stand-alone embed pays nothing.
* **Recalculation.** One render at a time; an input during a render marks the page dirty and one more render follows.
  Superseded renders are not aborted, because aborting the running job costs a fresh engine worker (the transport
  restarts it), which is slower than finishing a render that takes tens of milliseconds.

## postMessage API

Only the embedding page may drive the embed: a message is accepted only when `event.source === window.parent`. Messages
to the host go to `location.ancestorOrigins[0]` where the browser provides it (Chromium) and to `'*'` otherwise; the
host is by definition the page that framed us and the messages carry only the sample's public values and what the
reader typed into that host's frame.

| Message | Direction |
|---|---|
| `civilkit-embed-height {height}` | embed to host, on every layout change (ResizeObserver on the embed root) |
| `civilkit-ready {sample, inputs}` | embed to host, once the form is built |
| `civilkit-results {sample, values}` | embed to host, after each calculation |
| `civilkit-error {errors}` | embed to host, for a refused set |
| `civilkit-set {values}` | host to embed: names must be inputs; numbers finite and within the slider bounds; a choice one of the option values or labels; a checkbox true, false, 1 or 0. At most 200 names a message. |
| `civilkit-get` | host to embed: repeat the last results |

## embed.js

`public/embed.js` (served at `/calcs/embed.js`, under 3 KB, no dependencies) replaces each
`<div data-civilkit-sample="ID">` with the iframe, sandboxed with `allow-scripts allow-same-origin allow-popups
allow-popups-to-escape-sandbox`, and resizes it on `civilkit-embed-height` messages whose `event.origin` is
`https://civilkit.io` and whose source is that frame. `data-origin` and `data-path` point it at a self-hosted copy (and
let `web/e2e/embed-check.mjs` run it against a local build on another port). Every embed message is re-fired on the
iframe as a `civilkit` DOM event. The sandbox needs `allow-same-origin` (the engine's workers and module scripts load from
the app's origin) and `allow-popups-to-escape-sandbox` (so Open in CivilKit Calcs opens a normal, unsandboxed tab).

## Python

Python blocks run user code, so an embed leaves them out of what it calculates unless the address has `allowpython=1`,
and shows their code with a note instead. With `allowpython=1` the app's usual question is still asked before the first
run: a host can allow Python, it cannot accept it for the reader.

## Storage, privacy, analytics

The embed writes nothing to localStorage, sessionStorage or IndexedDB, and registers no service worker (the check in
`web/e2e/embed-check.mjs` confirms all four after an embed in which a value was changed, then opens the full app in the same
browser profile). It does read the script library list (`calcpad-libraries` in localStorage) through the script policy,
as every render does. No request leaves the app's own origin: no analytics, no fonts from elsewhere.

## Headers and the service worker

`web/calcpad-web/cloudflare/_headers` sets no `X-Frame-Options` and no `frame-ancestors` for the app, so the app (and so
the embed) can be framed by any site; nothing was added. Pages headers match paths, not query strings, so a
`frame-ancestors` rule for `?embed=1` alone is not possible: adding one would apply to the whole app. If the app ever
needs a frame restriction, it has to leave `/calcs/` (the shell that serves `?embed=1`) open, or the embed moves to its own
path (for example a separate `embed.html` entry) with its own rule.

The service worker is unchanged in behaviour: a reader who already has the app installed gets the embed from the same
precache (`index.html` is matched ignoring the query), and an embed never installs it. `embed/` (the catalogue page) is
left out of the precache like `learn/`; `embed.js` is precached with the rest of the build, which does no harm.

## Landing pages and the catalogue

`tools/learn/render.mjs` adds a "Try it" section (a lazy iframe with a fixed height of 560 px, so nothing shifts, and an
Open link under it that works without the frame) to every landing page with a sample. `tools/learn/embed.mjs` writes
`public/embed/index.html` with a snippet for every sample a landing page uses; `tools/make_learn_pages.mjs` writes it with
the landing pages and lists it in the sitemap, so `--check` covers it. `web/e2e/seo-check.mjs` checks its head, its
breadcrumb (CivilKit > Calcs > Embed), its links and every snippet address.

## Not done

* No Monaco editor in `embed=edit`: a plain text box keeps the embed small. The full editor is one click away.
* No per-embed persistence: values are lost on reload, by design.
* A Calcpad worksheet (not a sheet) with plain `?` inputs and no `#UI` lines shows its results without a form.
* The loader does not throttle or cap very tall embeds; a host that wants a cap should set its own `max-height` and let
  the frame scroll.
