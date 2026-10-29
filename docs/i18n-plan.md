# Multilingual support: survey and plan

Status: plan only. Nothing in this document is built. The counts come from reading the code on the `main` branch (the
number-format work on the unreleased `locale` branch is read but not part of these counts). Where a number is a
heuristic count, it says so; the first task of phase 0 replaces every heuristic count with an exact one from an
extraction script.

## Decisions already taken

* First wave of languages: Spanish (`es`), Portuguese (`pt`, Brazilian written form first), French (`fr`), German (`de`),
  Chinese Simplified (`zh`, hreflang `zh-Hans`). English (`en`) stays the source language.
* Order of work: (1) UI strings, engine message codes and a language picker in Settings; (2) translated landing pages
  under language paths; (3) translated guide chapters.
* Engine unit names and function names stay English (`kN`, `sin`, `n_rows`). Formulas stay canonical: a worksheet or sheet
  always means the same thing in every language, so files stay portable and the engine and its tests do not change.
* Number display (1,5 versus 1.5, digit grouping) is the `locale` branch's job (`docs/localisation.md` and
  `web/calcpad-web/src/wasm/number-format.ts` on that branch). This plan builds on it and does not redo it.

## 1. Inventory

### 1.1 What is in the UI

The web app has two code bases that draw UI. `web/calcpad-web/src` is ours. `web/calcpad-frontend/src` is a shared
package (sidebar tabs, settings, metadata, export tab, message bridge) that also serves other hosts, so a change there has
to stay neutral about where its translator comes from (see 2.9).

How UI text is made matters more than how much there is. Three mechanisms are in use:

1. Vue templates with literal text and `title="..."` / `aria-label="..."` attributes (`App.vue`, `Ribbon.vue`,
   `MathKeyboard.vue`, `MdBlockEditor.vue`, `LibrariesSettings.vue`, and the sidebar tabs in the frontend package).
2. Plain TypeScript that builds DOM or strings (`main.ts` is 3,180 lines and builds the welcome page, Monaco actions and
   notices; `wasm/web-files.ts`, `symbolic.ts`, `worksheet-import.ts`, `samples.ts`, `python-consent.ts`, `sheet-ui.ts`,
   `plot-chips.ts`, `editor/nav/palette.ts`, `editor/nav/rename.ts`).
3. Data tables read at module load time (`wasm/ribbon-data.ts`, `diagram-starters.ts`, the `BLOCK_ACTIONS` list in
   `App.vue`, setting definitions in `calcpad-frontend/src/types/settings.ts` and `pdf-settings.ts`).

Heuristic counts of user-visible English strings, found with a script that counts text nodes and `title`, `aria-label`,
`placeholder` and `alt` attributes in templates, and quoted multi-word English sentences in code (it skips imports,
comments and console calls). It misses one-word labels and lower-case starts, so treat each figure as a floor.

| File | Strings | Notes |
| --- | --- | --- |
| `wasm/ribbon-data.ts` | 274 | tabs, groups, 135 buttons built by `big/small/pal/more/both/action` helpers (26+26+1+10+22+50), 43 group names, about 90 palette sections |
| `App.vue` | 117 | 50 text nodes, 48 attributes (35 `title`, 12 `aria-label`), 19 in script (block menu, mobile panes, messages); 39 `{{ }}` interpolations |
| `main.ts` | 43 | welcome/empty page (a full HTML document with a hotkey table), Monaco action labels, unsaved-draft notice; 20 are template literals |
| `services/tauri-bridge.ts` | 38 | desktop host only; out of scope for the browser build, kept out of the count below |
| `editor/MathKeyboard.vue` | 32 | 19 tooltips |
| `wasm/symbolic.ts` | 30 | symbolic dialog |
| `wasm/LibrariesSettings.vue` | 22 | settings panel for script libraries |
| `wasm/worksheet-import.ts` | 21 | import dialog and result report |
| `wasm/diagram-starters.ts` | 20 | starter diagram titles (the diagram source text itself is data) |
| `editor/MdBlockEditor.vue` | 19 | toolbar tooltips |
| `editor/completions.ts` | 16 | |
| `wasm/Ribbon.vue` | 16 | |
| `wasm/web-files.ts` | 16 | open/save/recent dialogs |
| `wasm/plot-chips.ts` | 15 | |
| `wasm/transport.ts` | 14 | "needs a server" notices for 501 endpoints, print page |
| `editor/nav/rename.ts`, `palette.ts` | 24 | rename plan text, palette text |
| 15 smaller files | about 70 | python consent, sheet blocks, samples dialog, hover, server manager |
| **calcpad-web total (36 files)** | **about 790** | |
| `calcpad-frontend` Settings tab | 106 | 72 text nodes, 31 attributes |
| `calcpad-frontend` Metadata tab | 101 | 67 text nodes |
| `calcpad-frontend` `metadata-comment.ts` | 48 | property descriptions |
| `calcpad-frontend` `types/settings.ts`, `pdf-settings.ts` | 67 | setting names and help text |
| `calcpad-frontend` Export, Variables, Insert, Files, TOC, Errors, Formatting tabs | about 115 | |
| `calcpad-frontend` message bridge, ui-directive, others | about 75 | |
| **calcpad-frontend total (25 files)** | **about 510** | |
| **Detected total** | **about 1,300** | likely 1,800 to 2,500 after exact extraction |

Shapes that make strings harder than plain text:

* Interpolated: 56 template literals in calcpad-web script, 17 in calcpad-frontend, about 190 `{{ }}` expressions across the
  templates (many are not text, for example class bindings). Realistically 100 to 150 strings carry a parameter.
* Plural: few. Found 5 `=== 1 ? '' : 's'` sites plus a local `plural(n, one, many)` helper in `editor/nav/rename.ts` used
  3 times (for example "1 place on 1 line"). Plural handling is needed but is a small feature, not a large one.
* Shortcut hints are appended to text: `deskKey(' (Ctrl+Shift+B)')` is used 5 times in `App.vue`, and `Ctrl+` appears 65 times in
  the source. Hints must become a parameter of the sentence, not a suffix. `deskKey` swaps Ctrl for Cmd on Apple platforms.
* Dates and times: `App.vue` formats a console timestamp with the fixed locale `en-US`; `web-files.ts` uses the browser
  default for recent files.
* Welcome page: an HTML document built in `main.ts` with two conditional hotkey tables and four outbound links.

Engine-rendered HTML shown in the preview is covered in 1.2. Monaco's own built-in text (find widget, context menu, command
palette) is English and is not part of these counts; see risks.

### 1.2 What the Rust engine says

**Catalogue of messages.** `calcpad-core/src/messages.rs` holds 205 named constants (generated from an upstream resource file by
`tools/gen_messages.py`), about 1,300 words. 83 take positional placeholders (`{0}`, `{1}`), 2 contain the function and variable
markers `%F` and `%V`, 5 contain HTML (the pause notice and the "Found N errors on lines" header). 197 distinct constants are
referenced from other crates. The constant name is already a stable identifier, which is what makes codes feasible.

**How they are raised.** `calcpad-core/src/exceptions.rs` has 137 constructor functions (`InvalidSymbol`, `InvalidUnits`, and so
on) that call `MathParserException::new(fmt(messages::X, &[args]))`. The exception stores only the final string
(`message: String`). 217 lines across the crates mention `messages::`. The text then travels in two places: as the `message`
of the error list the wasm layer returns (`{"html": ..., "errors": [...]}`), and inline in the rendered HTML in
`<span class="err">` and `<p class="err">` elements.

**Not in the catalogue.** About 200 more capitalised English string literals exist in `calcpad-core/src` outside the catalogue
(measured with a pattern, so some are not messages); examples are in the eigenvalue solvers, the statistics module and the
plotter ("Raster (PNG) plots are not supported..."). Other crates, same pattern: 55 in `calcpad-sheet`, 75 in the highlighter's
linter, 16 across import, docx and wasm. A real audit is the first task of the engine phase.

**Linter.** The highlighter's linter already has codes: 66 distinct `CPD-nnnn` codes at 77 `add_error` / warning call sites, with
the message built by `format!` at the call site. The code is in the diagnostic JSON next to the English message, so the web
layer can already key on it; the arguments are not separated out yet.

**Report chrome from the engine.** The Word and Excel writers (`calcpad-docx`) write fixed English words: sheet names and
column headings ("Report", "Variables", "Name", "Value", "Unit", "Description", "Expression", "Note", "Plot data"), "With the
values", "Error in". The Word writer sets only ASCII and high-ANSI fonts (`w:rFonts w:ascii w:hAnsi`), with no East Asian font.

**Code that reads engine text.** Searched the web code for pattern matches against engine message text: none found. The
web code remaps line numbers in HTML and JSON, never parses a message. That removes the biggest risk for the codes approach.
Tests do assert English: roughly 30 Rust assertions compare message text and 584 `contains("...")` calls exist in
Rust tests; they keep working because the English text stays in the engine (see 2.4). Three e2e files assert engine error
text (5 lines).

**Snippet library.** `calcpad-highlighter/src/snippets/snippets.json` is the data behind the Insert tab, the ribbon palettes,
completions and hover: 972 snippets, 959 distinct descriptions (4,289 words), 285 with Markdown documentation, 86 categories.
This is user-facing English that lives in the engine data, served through the `/api/calcpad/snippets` call. It is also the
source of the generated reference chapter of the guide.

### 1.3 Samples and templates

* `public/samples`: 107 worksheets (11 groups); `public/sheet-samples`: 44 sheets (5 groups, including 7 templates).
  Manifests carry `title`, `description`, `group`, `folder` for each: about 4,100 words of title and description in total.
* The sample contents are Calcpad text or sheet text: names, units and formulas are canonical, the prose (headings, comments,
  notes after a bar, input labels) is English. Prose inside a sample is free text and safe to translate; names and formulas must
  not change.
* Sample ids are used by links (`?sample=<id>`), by the landing pages and by `e2e/sample-link-check.mjs`, so ids stay English.
* "Compiled hashes": the only hashes found are the SHA-384 pins of the script libraries a scripted sample may load
  (`library-defaults.json`, checked in `browser-check.mjs`). They pin library files, not sample text, so translating prose
  does not touch them. There is no content hash of sample or compiled text. Any golden compiled output added in future must be
  stored per language.

### 1.4 Guide (`book/src`)

31 Markdown pages, 45,675 words. By area: sheet 8 pages (6,985 words), calcpad 3 (3,640), getting-started 3 (2,349), appendix 3
(1,536), plus single chapters (editing, exporting, importing, plots, python, samples, symbolic, templates, troubleshooting, units,
vectors-matrices, introduction). `reference/functions.md` (6,513 words) is generated from the snippet library by
`book/gen-reference.py`, so it is translated through the snippet catalogue, leaving about 39,000 words of hand-written prose.
`book/build.sh` runs mdBook and then `tools/guide_seo.mjs` (description, canonical, og:url, breadcrumbs). Code blocks in the
guide are run by `book/check-examples.mjs`. Guide videos (`book/src/videos`) are English narration with no captions file; that
is a separate item.

### 1.5 Landing pages and generators

* `tools/make_learn_pages.mjs` writes `public/learn/<slug>/index.html` for 21 landing pages, the hub `public/learn/index.html`,
  `public/sitemap.xml` and `public/robots.txt`. Data is in `tools/learn/pages.mjs` (1,078 lines, 17,844 words including
  code); the page template is `tools/learn/render.mjs`; the quoted numbers come from real engine runs
  (`tools/learn/engine.mjs`).
* Structure per page: `slug, label, group, title, description, h1, lead (HTML), sections [[heading, html]], faq [[q, a]],
  related, runs, open, video`. Prose is HTML strings with placeholders `[[run:key]]` and `{{code:run}}`. The template also owns
  chrome strings: skip link, nav labels, breadcrumbs, footer, "Questions", "Related pages", button labels, hub group intros
  and the JSON-LD descriptions. Estimated 14,000 words of translatable prose in `pages.mjs` plus about 400 words of chrome.
* The pages contain no script, link `learn.css`, are `lang="en"`, and the service worker leaves `learn/` out of its precache.
* `web/e2e/learn-check.mjs` re-renders and compares the committed files; `seo-check.mjs` checks titles, descriptions,
  canonicals, one h1, JSON-LD, sitemap and link forms. Both are English-only.

### 1.6 What's new, llms files, manifest, service worker

* What's new: `public/whats-new/index.html` (about 9,000 words) is generated by `tools/make_whats_new.mjs` from `CHANGELOG.md`
  (about 6,500 words) and `whats-new/clips.json` (178 lines of per-clip sentences). It is a release log. Plan: keep it English
  only, state so on the translated shell pages. Translating release notes forever is a recurring cost with little search value.
* llms files: `public/llms.txt` (12 KB, an index with one sentence per chapter and per landing page) and `public/llms-full.txt`
  (283 KB, about 51,000 words, the whole guide plus the sheet syntax reference), written by `tools/make_llms_txt.mjs`.
* `public/manifest.webmanifest`: one file with English `name`, `short_name` and `description`. A manifest cannot vary per user
  language in a static host; the install name stays "CivilKit Calcs" (a product name) and only `description` is English.
  Not worth a per-language manifest in wave one.
* Service worker (`pwa-plugin.ts`): precaches the whole build at install (`cache.add` of every file except `whats-new/`, `learn/`,
  sitemap, robots, llms), and caches any other same-origin GET it sees at runtime (`cache.put` after a network hit). So a
  language catalogue that is not in the install list is still cached after first use, and works offline for that visitor on the
  next visit. If catalogues are bundled into hashed JS chunks they would be precached for all five languages, wasting install size;
  design 2.3 avoids that.

### 1.7 End-to-end checks that assert English text

`web/e2e` has 61 files. Counting text-based selectors and assertions (`getByText`, `getByRole(name)`, `has-text`, `text=`,
`[title=...]`, `[aria-label=...]`, `getByLabel`, `getByTitle`, `getByPlaceholder`): **32 files, about 145 lines**. Largest:
`import-check.mjs` 19, `browser-check.mjs` 17, `sheet-check.mjs` 11, `select-map-check.mjs` 8, `inputs-check.mjs` 7,
`table-sort-check.mjs` 6, `plot-chips-check.mjs` 6, `fold-key-check.mjs` 6, `editor-nav-check.mjs` 6, then 23 files with 1 to 5
each. Another group asserts English page content rather than UI: `learn-check`, `seo-check`, `whats-new-check`,
`whats-new-browser-check`. These checks run with the Playwright default browser language (`en-US`), so detection by browser
language (2.5) gives English with no change to them; pin `?lang=en` in a shared helper for safety. A new `i18n-check.mjs`
boots each language, which also covers the case where the checks are run on a machine with a non-English browser language.

## 2. Recommended technical design

### 2.1 Library choice: a small in-house `t()`

Recommendation: write about 150 lines of in-house code (`web/calcpad-web/src/i18n/`), not `vue-i18n`.

Reasons, in order of weight:

1. **The app mixes Vue and plain TypeScript.** More than half of the strings (the ribbon data, dialogs built by DOM code, the
   welcome page, Monaco actions, notices) are outside any Vue component. `vue-i18n` works there through its global composer, but
   its reactivity and component-level API are the part you pay for and would not use. A module-level `t(key, params)` that reads
   a Vue `shallowRef` catalogue is reactive inside templates (Vue tracks the ref read) and plain everywhere else.
2. **Size and offline.** The app already ships a large editor and several wasm modules. A full `vue-i18n` runtime with its
   message compiler is in the tens of kilobytes gzipped (not measured here; no builds were run for this survey); the in-house
   version is about 1 KB. Not decisive alone, but nothing is gained for it.
3. **Plurals without ICU.** We need `one` / `other` for English, Spanish, Portuguese, French, German, and only `other` for
   Chinese; Spanish, Portuguese and French also have a `many` category for large round numbers. `Intl.PluralRules` (built into
   every browser the app supports) returns the category, so the catalogue needs suffixed keys (`key_one`, `key_many`,
   `key_other`) and no message-format parser.
4. **No extra build step or CSP exposure.** Precompiling messages needs a vite plugin and generated code; the in-house format is
   data (JSON) with `{name}` substitution only.
5. **The extraction and check scripts are simpler** when the format is trivial JSON that we control.

What is given up: ICU select/gender and nested rich text. Neither is needed in wave one. Where a sentence contains a link or
a bold word, the catalogue holds a tiny fixed markup (`<b>`, `<kbd>`, `<a>` with a named slot `{link}`) rendered by one helper
`tHtml()` that escapes parameters; the catalogue is trusted content we ship, parameters are escaped.

If the plan changes and ICU becomes necessary (for example a language with complex agreement), swapping the implementation behind
the same `t()` signature to `vue-i18n` or `@formatjs` is a contained change.

### 2.2 API

```ts
t(key: Key, params?: Record<string, string | number>, fallback?: string): string   // plain text
tn(key: Key, n: number, params?): string          // plural: picks key_one / key_many / key_other with Intl.PluralRules
tHtml(key: Key, params?): string                   // catalogue markup, parameters escaped
getLanguage(): LangCode; setLanguage(lang: LangCode, opts?): Promise<void>; onLanguageChanged(cb): () => void
```

* Vue: `app.config.globalProperties.$t = t` and `useI18n()` that returns `t`. Templates write `:title="$t('app.sidebar.resize')"`.
  Changing language swaps the catalogue ref; templates re-render.
* **`Key` is a union type generated from `en.json`**, so a typo is a compile error where the project type-checks, and the check
  script (4.3) enforces it everywhere else.
* The optional third argument is the English fallback. It lets data tables stay as they are. Example: `Ribbon.vue` renders
  `t('ribbon.' + btn.id + '.label', {}, btn.label)`, and `ribbon-data.ts` stays plain English data that `verify-ribbon-snippets.mjs`
  can still load under node. The extraction script derives the ribbon keys from the data, so no one writes them by hand.
* Module-level constants that hold English text and are read once (`BLOCK_ACTIONS`, `MOBILE_PANES`, the settings definitions) must
  hold a key and resolve it at render time, or the language change will not reach them. This is the main mechanical refactor.

### 2.3 Catalogue format, loading and offline

* One flat JSON file per language: `web/calcpad-web/public/i18n/<lang>.json` for non-English, and English bundled in the main
  chunk as `src/i18n/en.json` (no flash and no network dependency in the default language). Flat keys with dots.
* Non-English catalogues are fetched at startup before first render when the stored or detected language is not English (one
  request, an estimated 40 to 70 KB raw, 12 to 20 KB gzipped for about 2,000 strings), and on language change.
* Public folder, not hashed chunk: `pwa-plugin.ts` precaches the build file list, so add `i18n/` to its exclusion list (as for
  `learn/`) and the runtime caching in the existing fetch handler stores the visited language after first use. Offline works
  for the languages a person has used online once. Say so in the Settings help text (the same limitation applies today to
  Plotly, noted in the landing pages).
* A catalogue carries a header `{"_meta": {"lang": "es", "version": 3, "source": "en@<hash>"}}`. A missing key falls back to
  English, then to the key's last segment in development, never to an empty string.
* The cache name already follows the content hash of the precached files; catalogues are fetched with the app version in the
  query string (`?v=<app version>`) so a new release refetches.
* Engine-side catalogues (messages, snippets, linter) are separate files, `public/i18n/<lang>.engine.json`, loaded lazily when
  the first engine result needs them, so a person who never triggers an error never downloads them.

### 2.4 Key naming

`area.component.element` in lower camel segments, stable and meaning-based, never the English words:

* `app.tabs.close`, `app.sidebar.resizeHint`, `ribbon.insert.matrix.label`, `ribbon.insert.matrix.title`, `dialog.import.title`,
  `toast.savedDraft_one`, `settings.language.label`, `kbd.ctrl`.
* Engine messages reuse the existing constant name: `msg.Invalid_symbol_0`, `msg.Variable_0_does_not_exist`. Linter: `lint.CPD-3301`.
* Samples: `sample.<id>.title`, `sample.<id>.description`. Snippets: key by a stable id (see 2.6).
* Placeholders are named, not numbered, in the UI catalogue (`{count}`, `{name}`, `{key}`), because translators reorder words.
  Engine messages keep `{0}`, `{1}` since the engine produces positional arguments and the strings are generated from the
  same upstream resource.
* A shortcut hint is a parameter: `"app.sidebar.collapse": "Click to expand the sidebar {hint}"`. The hint itself is
  `Ctrl+Shift+B` (or `Cmd+Shift+B`), formatted by `deskKey`. Key names stay `Ctrl`, `Shift`, `Alt` in all five languages except
  German keyboards, which print `Strg`; use a small `kbd.ctrl` key so `de` can say `Strg`, and leave the rest.

### 2.5 Detection, storage and URL override

Same shape as the number-format setting on the `locale` branch, so the two settings feel alike:

1. `?lang=es` applies to that page view only and stores nothing (like `?numfmt=`). Useful for the landing pages' "Open in the app"
   links and for tests.
2. `localStorage` key `calcpad-language`, wrapped in try/catch as elsewhere (storage can be blocked).
3. First visit: the first entry of `navigator.languages` that maps to a supported language (`es-MX` to `es`, `pt-PT` to `pt`,
   `zh-CN`/`zh-SG` to `zh`). Anything else is English. Traditional Chinese regions (`zh-TW`, `zh-HK`) fall back to English until a
   Traditional catalogue exists (open question).
4. Settings has a Language selector (a new `LanguageSettings.vue`, injected like `NumberFormatSettings.vue` and
   `LibrariesSettings.vue`), listing each language in its own name: English, Espanol, Portugues, Francais, Deutsch, Chinese
   (written in Chinese characters). The change applies at once, no reload.
5. `document.documentElement.lang` is set to the active language, so screen readers, hyphenation, `:lang()` CSS and browser
   spell-check act correctly.
6. Interplay with the `locale` branch: its "Automatic" number format reads `navigator.language`. Keep that: the UI language and
   the number format are separate choices (a Spanish-language UI used in Mexico wants a decimal point; in Spain a comma).
   Make `languageNow()` there stay browser-based, and show the number-format sample in the new language's words. Merge `locale`
   first; `i18n` touches `setup.ts`, `transport.ts` and `main.ts`, which that branch also touches.
7. Dates and times use `Intl.DateTimeFormat(lang)` and replace the fixed `en-US` in `App.vue`.

### 2.6 Engine strings: codes plus parameters, translated at the edge

Recommended: **the engine keeps producing English and adds a code and its arguments; the web layer swaps the text.** This is
the same pattern the `locale` branch uses for numbers (rewrite at the transport edge, copy everything else byte for byte).

Why not compile languages into the wasm: `messages.rs` constants are `&'static str` and the wasm size budget is tight; five
language tables would grow every download for people who use one; the Rust tests (584 `contains` calls, about 30 equality checks)
assert English; and a language switch would need an engine restart.

How:

1. `MathParserException` gets `code: Option<&'static str>` and `args: Vec<String>` next to `message`. A helper
   `msg_err(ID, TEMPLATE, &[args])` returns the English string and records the code (the constant name) and arguments. The 137
   constructors in `exceptions.rs` change mechanically (the template constant and the name already exist). About 80 other call
   sites that build `MathParserException::new(format!(..))` or `fmt(messages::..)` directly get the same helper; the 13 in
   `calcpad-core` that use inline English text first move their text into `messages.rs`.
2. The wasm JSON gains `code` and `args` on each entry of `errors`, and the rendered HTML gets `data-code="Invalid_symbol_0"
   data-args='["x"]'` on the `err` span or paragraph. The visible text remains the English message, so any consumer that ignores
   the attributes behaves as today (Word, Excel and print of an English session are unchanged).
3. In `transport.ts`, after the engine answers and before the page is shown, printed or handed to a writer, a function
   `localiseMessages(html, errors, lang)` replaces the text of each element that has `data-code` with
   `fmt(catalogue['msg.'+code], args)`. When the language is English it does nothing. If a code is missing from the catalogue,
   the English text stays.
4. Arguments that are themselves words ("number", "vector", unit-kind names passed as the second argument of
   `InvalidLiteral`) are a known wrinkle: list those arguments, give them their own codes (`arg.number`), and let the edge
   translate an argument that matches a code. Numeric arguments pass through the locale branch's number formatting.
5. The five messages with HTML keep their markup in the catalogue; the catalogue check (4.3) verifies the tag set matches English.
6. Linter: add `args` to the diagnostic JSON for the 66 `CPD-` codes (the message is built by `format!` today, so each site changes
   to pass its arguments separately). The editor's problems panel and Monaco markers use `lint.<code>` with a fallback to the English
   `message`.
7. Snippets: translate at the same edge. The `/api/calcpad/snippets` answer is rewritten by `transport.ts` using
   `public/i18n/<lang>.engine.json`, keyed by a stable snippet id. Snippets have no id field today; use `Category + "|" + Insert`
   (unique enough, to be confirmed by the extractor, which fails on a collision) and add an `Id` field in the next library change.
   Insert text is never translated.
8. Word and Excel chrome ("Report", "Variables", column headings): pass a small string table into the docx and xlsx wasm calls
   (one JSON argument), or translate the same fixed words in the web layer before calling. The first option is cleaner and
   changes only `calcpad-docx` and `calcpad-docx-wasm` once.
9. All of this needs a wasm rebuild and release. It should not be started while a release build is running.

Feasibility verdict: **codes are feasible and mostly mechanical** for the 205 catalogue messages and the 66 linter codes. The
uncatalogued literals (about 200 in core, 55 in sheet, 16 elsewhere) need a triage first; engine errors that stay English in wave
one are acceptable if the UI says nothing wrong about them.

### 2.7 Things that must not be translated

Unit names, function names, keywords (`#if`, `#for`), operators, variable names inserted by snippets, sheet syntax, example
formulas, file names and sample ids, library names, URLs. The highlighter, completions and hover keys stay English. Ribbon
buttons that insert text insert the canonical English snippet (the label and tooltip are translated, the inserted text is not).
Some sheet snippets include English prose, such as a note after `|` or a heading; those inserts are code-like examples and
stay English in wave one (a person can type their own prose); the plan lists them in the risk table.

### 2.8 CJK and fonts

* **Editor and preview.** The shipped fonts are JuliaMono (editor) and DejaVu, Jost and a vector font (report). Latin-1 and Latin
  Extended letters used by Spanish, Portuguese, French and German are covered; a visual pass in all four languages must confirm
  it. None of the fonts has Chinese glyphs. Add a CJK fallback to the font stacks under `:lang(zh)`: system faces first
  (PingFang SC, Hiragino Sans GB, Microsoft YaHei, Noto Sans CJK SC, Source Han Sans SC), generic `sans-serif` last. Do not ship a
  web font for the UI in wave one (a Simplified Chinese subset is several MB).
* **Monaco.** Mixed-width text in comments can drift the cursor when the first font has no glyph for a character. Test Chinese
  in a prose comment and a `#` heading; the option `disableMonospaceOptimization` is the likely remedy, and the editor font
  setting should take the same CJK fallbacks.
* **Print and PDF.** PDF is the browser's print path, using the page's fonts plus system fallback. A reader on any common desktop
  has a Chinese system font, so printed Chinese works without embedding. If a reproducible PDF becomes a requirement, embed a
  subset font in the report head (`report-fonts.ts` already embeds as data URLs); estimate 0.5 to 1.5 MB for a common-character
  subset, loaded only when the document contains Chinese.
* **Word.** Add `w:eastAsia` to the run fonts and a `w:lang` with the document language in `calcpad-docx/src/writer.rs` and
  `package.rs` (today only `w:ascii` and `w:hAnsi` are set). Word substitutes an installed Chinese font when none is named.
* **Word plots and any SVG the engine rasterises** use `resvg` with only the fonts the page hands in (`add_font`); Chinese text in a
  plot title or axis label would render as empty boxes. Wave one: leave axis text as typed, and document that Chinese plot labels
  need the font hand-off. The Chinese subset above, passed to `add_font` only when Chinese is present, closes it.
* **Excel** is unaffected (cells hold Unicode; the viewer chooses the font).
* Right to left: not needed in wave one. For later: set `dir` from the language, convert physical CSS (`left`, `margin-left`) to
  logical properties, and keep formulas, numbers and code left to right with `unicode-bidi: isolate`. The catalogue and `t()`
  carry no direction assumption.

### 2.9 Where the shared frontend package gets its translator

`calcpad-frontend` is imported by `calcpad-web` and serves other hosts, so it must not import `calcpad-web`. Add a tiny module
there, `i18n-hook.ts`, exporting `setTranslator(fn)` and `t(key, params, fallback)` that defaults to returning the fallback
(its English). The web app calls `setTranslator` at startup. Every frontend string then needs a key and its English as the
fallback argument, so the package stays self-contained and other hosts see no change. Upstream merges stay manageable
because each edit is a local `t('...', {}, 'English')` wrap. Expect merge friction in `CalcpadSettingsTab.vue` and
`CalcpadMetadataTab.vue` (about 200 strings together); do those two last.

### 2.10 Pseudo-locale for development and QA

Add a hidden language `xx` that wraps every string as `[!! Ëñglïsh ténd ténd !!]` with roughly 35 percent padding. It finds untranslated
strings (they appear plain), hard-coded strings, concatenation (broken brackets), truncation (German is typically 20 to 35
percent longer than English) and non-escaped parameters. Only `?lang=xx` reaches it. It is the main safeguard for the ribbon
layout, where button widths are fixed and `ribbon-layout-check.mjs` already exists.

## 3. SEO and URL plan for translated pages and guide

### 3.1 URL scheme

Single app shell, language by setting; pages and guide by path.

* App: stays at `/calcs/` for every language (language by `?lang=` / setting / browser). No `/calcs/es/` copy of the app: that
  would duplicate the wasm and bundle for no search benefit and split the service worker scope.
* Landing pages: `/calcs/<lang>/learn/<slug>/` and hub `/calcs/<lang>/learn/`. English stays at `/calcs/learn/...`.
* Guide: `/calcs/<lang>/guide/<chapter>` (same extensionless forms as English).
* What's new: English only (see 1.6). A translated shell is not generated.
* Language codes in the path: `es`, `pt`, `fr`, `de`, `zh`. In `hreflang`: `es`, `pt-BR` (decide, see open questions), `fr`, `de`,
  `zh-Hans`, plus `x-default` pointing to the English page.
* Slugs: wave one keeps the English slug under the language prefix (one mapping, links and open-sample ids stay valid).
  Localised slugs could help long-tail queries slightly; adding a `slugs` field per language is a later option, and a redirect
  map must then exist. Listed as an open question.

### 3.2 Head tags per page

Every translated page carries: `<html lang="es">`, its own `<title>` and meta description, canonical to itself, and a full
alternate set on every language version: one `<link rel="alternate" hreflang="..." href="...">` per language including itself,
plus `x-default`. Open Graph `og:locale` and `og:locale:alternate`. JSON-LD `inLanguage`. The same set is added to the English
page, so alternates are reciprocal (a Google requirement). Pages that have no translation yet are simply left out of the alternate
set and out of the sitemap; no page is published as a machine-translated copy of English without review (see 4).

### 3.3 Sitemaps

`sitemap.xml` currently lists the app, hub, guide chapters and landing pages. Move to a sitemap index:
`sitemap.xml` (index) pointing to `sitemap-en.xml`, `sitemap-es.xml`, and so on, each listing that language's URLs with
`<xhtml:link rel="alternate" hreflang>` entries so the alternates are also declared in the sitemap. `robots.txt` lists the index.
Generated by `make_learn_pages.mjs` (it already writes the sitemap and robots), from the set of languages whose overlay exists.

### 3.4 Generating landing pages per language

* Add `tools/learn/i18n/<lang>.mjs`, exporting for each slug the translated fields only: `label, title, description, h1, lead,
  sections, faq, openLabel, cta`. Structure, runs, related, groups and video stay in `pages.mjs`.
* `renderPage(page, runs, lang)` merges the overlay over the base page, with chrome strings from `tools/learn/i18n/chrome.<lang>.json`
  (skip link, nav, breadcrumbs, footer, "Questions", "Related pages", button labels, hub intros).
* Numbers: the engine run values are the same in every language. For `es`, `pt`, `fr`, `de` render them in the language's
  decimal format by calling the pure part of `number-format.ts` from the `locale` branch (`localiseHtml`, no browser
  dependency; node already imports `.ts` data files for `verify-ribbon-snippets.mjs`). Prose that quotes numbers inline is
  written by the translator and checked (4.3) to contain the same digit sequence modulo format.
* `learn-check.mjs` and `seo-check.mjs` loop over languages: same structure, canonical per language, reciprocal alternates, no
  untranslated placeholder markers, every internal link resolves in its own language or falls back to English explicitly.
* The "Open this template" link goes to `/calcs/?sample=<id>&lang=<lang>` so the app opens in the page's language.

### 3.5 Generating the guide per language

Two ways were weighed.

1. **Per-language copy of `book/src`** (`book/i18n/<lang>/src`), built by running mdBook once per language with
   `MDBOOK_BOOK__LANGUAGE`, `MDBOOK_BOOK__SRC`, `MDBOOK_OUTPUT__HTML__SITE_URL=/calcs/<lang>/guide/` and
   `MDBOOK_BUILD__BUILD_DIR` set from `book/build.sh`. Simple, but a changed English paragraph silently leaves a stale
   translation and there is no signal for what changed.
2. **Gettext catalogues** with the `mdbook-i18n-helper` pair of preprocessors (`mdbook-xgettext` extracts messages to a `.pot`;
   `mdbook-gettext` applies a per-language `.po` at build). Untranslated or changed paragraphs show up as missing or fuzzy,
   `msgmerge` carries translations across edits, and translation tools and machine-translation pipelines accept PO natively.

Recommendation: option 2 for the guide (31 pages, 39,000 words that change with every release, where drift is the real
problem), option 1's file layout for landing pages (they are data, not Markdown). Verify at the start of phase 6 that the helper
works with the pinned mdBook version; the build box needs `gettext` tools. Required in either case:

* `book/build.sh` loops `en` plus languages that have a catalogue, runs `tools/guide_seo.mjs --dir` per output with the language
  (canonical, alternates, `og:locale`, breadcrumb names in that language), and writes into `target/book/<lang>/`.
* `guide_url.mjs` takes a `lang` and returns `/calcs/<lang>/guide/<path>`; `guideChapters()` reads the translated chapter titles
  from the catalogue.
* **Code blocks must be identical to English.** `check-examples.mjs` runs the examples of the English guide; add a check that every
  fenced block in a translated page equals the English block byte for byte (comments inside a block may be translated only if
  explicitly allowed, default no).
* The search index mdBook builds (`elasticlunr`) has no tokenizer for Chinese; searching works on whole words only. For `zh`,
  either disable search or accept it (open question).
* `reference/functions.md` is generated per language from the snippet catalogue by `gen-reference.py --lang`.

### 3.6 llms files per language

`make_llms_txt.mjs` writes `public/llms.txt` and `llms-full.txt` for English. Add `public/<lang>/llms.txt` and
`llms-full.txt` generated from that language's built guide and overlay pages, each beginning with a line that names the English
file as authoritative. Do this last; the English files remain the ones the host copies to its root. They stay out of the
precache as now.

### 3.7 Worker and hosting

The site is static files deployed as a Pages project root with the app under `/calcs/` (see `docs/DEPLOY.md`). The Worker that
routes the host's paths is not in this repository, so nothing here was checked against it. Requirements to give its owner:

* Pass `/calcs/<lang>/...` through to the static project unchanged; no rewrite and no trailing-slash change for the language
  prefix.
* **No redirect based on `Accept-Language` or IP**: crawlers fetch from one place and would never see the other languages, and
  people would be bounced away from a link they chose. Language suggestion belongs in the page (an optional dismissible banner
  linking to the translated version), not in the router.
* Serve the per-language sitemaps and llms files from the same prefix; add the sitemap index to the host's root `robots.txt`
  (which is owned elsewhere).
* Cache headers for `/calcs/i18n/*`: short or revalidated cache (catalogue files are not hashed); `_headers` gets one line.

### 3.8 Interplay with the SEO work

`docs/seo-plan.md` plans 30 more English landing pages. Rules to keep the two efforts from fighting:

* The English page set is the source. Add a language only for a page that is stable (published for at least one cycle, not in
  the "next 30" queue). Start with the ten pilot pages and the hub.
* The three intents (replacement, task, how-to) do not all translate. Task pages (a calculator for a beam, a footing, a pipe) and
  how-to pages translate directly. Comparison pages name other products and make claims that need local review; treat them as
  last and optional per language.
* `make_learn_pages.mjs` is shared by both efforts. Coordinate on one change: split `pages.mjs` fields into "structure" and
  "text" before either effort adds many pages, so adding a page produces a stub in every overlay (reported by the coverage
  script, not an error until the language is published).
* The sitemap, `learn-check`, `seo-check` and the hub's `numberOfItems` JSON-LD all change under 3.3 and 3.4. Agree who owns that
  commit; the SEO work should merge first.
* Search Console: add a property view per language prefix and watch "Alternate page with proper canonical" and hreflang errors.

## 4. Translation workflow

### 4.1 Extraction and coverage

`tools/i18n/extract.mjs` (node, no dependencies):

* Walks `web/calcpad-web/src` and `web/calcpad-frontend/src`, finds `t('...')`, `tn('...')`, `tHtml('...')`, `$t('...')`
  literals, plus the generated sources (ribbon keys from `ribbon-data.ts`, setting keys from `settings.ts`, `msg.*` from
  `messages.rs`, `lint.*` from the linter code table, `sample.*` from both manifests, snippet keys from `snippets.json`).
* Writes `en.json` (source) and `engine.en.json`; for each other language writes `coverage` as translated, missing, stale
  (English changed since translation, tracked by a hash of the source string stored in `_meta`), and total.
* `npm run i18n:report` prints a table per language and per area; this is the exact replacement for the heuristic counts above.
* A one-time codemod helps phase 1: for each template text node or attribute it proposes a key and rewrite. Review by hand;
  do not trust it for strings with markup.

### 4.2 Machine-translation drafts and human review

* Drafts: produced from the English catalogue with a language model, in batches by area, with the glossary (below) and the
  placeholder rules in the prompt, and the context field of each key (where it appears, character limit for buttons). The draft
  step is a script (`tools/i18n/draft.mjs`) that writes only keys that are missing and marks them `"_draft": true` in a sidecar
  file, never overwriting a reviewed value.
* **A native-speaker review is required before a language is published, and engineering terms need a reviewer who works in that
  engineering field.** Machine output is fluent but picks the wrong term for specialist words (for example the word for "member",
  "section", "restraint", "load case", "deflection" or "bearing" differs by country and by discipline, and a general dictionary
  choice reads as foreign to an engineer). A bilingual non-engineer cannot catch this; budget a paid or volunteer reviewer
  per language for the UI, a second pass for the landing pages, and the guide separately.
* **Glossary** `tools/i18n/glossary/<lang>.tsv`: columns `en`, `target`, `note`, `status` (reviewed / proposed). 150 to 250
  terms (member, node, section, span, load, moment, shear, deflection, stress, bearing pressure, restraint, template, worksheet,
  sheet, variable, units, input form, report, export, ribbon, preview, and so on). It is built first, reviewed first, injected
  into every draft prompt, and used by a check that flags a target string that contains the English term where the glossary has a
  translation (soft warning).
* Regional choice recorded in the glossary header: Spanish neutral Latin American with Spain differences noted (open question),
  Portuguese Brazilian, French France, German standard (informal or formal address: choose formal `Sie`), Chinese Simplified with
  mainland engineering terms.
* Publication gate per language: coverage 100 percent for the UI keys, glossary reviewed, a reviewer's sign-off recorded in
  `_meta.reviewed_by` (a name or handle, not an email), the pseudo-locale pass clean, and a screenshot pass of the ribbon, settings,
  the import dialog and the welcome page at phone width.
* Until reviewed, a language may be selectable only through `?lang=` (hidden in the picker), or shown with a "translation in
  progress" label. Never label a machine draft as final.

### 4.3 Check script plan

`tools/i18n/check.mjs`, run by the same release checks as the other generators (`--check` style), fails the build on:

1. **Missing keys:** a `t()` key used in code but absent from `en.json`; a key in `en.json` absent from a language that is marked
   published (unpublished languages report only).
2. **Unused keys:** a key in any catalogue that no source references (taking into account the generated key families).
3. **Placeholder mismatch:** the set of `{name}` / `{0}` placeholders differs between English and the translation; also unmatched
   braces; in `tHtml` strings, the tag set and link slots differ.
4. **Plural completeness:** a `_one` without `_other`; a language missing a category its `Intl.PluralRules` can return for the
   counts we use.
5. **Forbidden changes:** a translation altered inside a code span, a unit name or a function name from the protected list.
6. **Length warning:** a translated button label more than 140 percent of the English length (warn, not fail).
7. **Duplicate source strings** with different keys (warn; suggests a shared key).
8. For landing pages and the guide: same number of sections, same placeholder markers (`[[run:..]]`, `{{code:..}}`), same links
   (internal hrefs identical after language-prefixing), fenced code identical, same count of headings, and every number that
   appears in the English paragraph appears in the translation in the language's format.
9. A hard-coded string detector in the source: a Vue text node or `title` that is not a `t()` call and is not on an allow list.
   Start as a warning with a ratchet (count may only go down) and become a failure when the count is zero.

## 5. Phased task list

Size key: S is under a day, M is two to four days, L is about a week or more of focused work. Estimates assume one developer who
knows the code and exclude waiting for reviewers.

| Phase | Work | Size | Release contains |
| --- | --- | --- | --- |
| 0 Foundation | `i18n/` module (`t`, `tn`, `tHtml`, detection, storage, `lang` attribute, loader), `en.json` and typed keys, pseudo-locale, `extract.mjs`, `check.mjs`, `LanguageSettings.vue`, frontend `i18n-hook.ts`, `i18n-check.mjs` | M | Language picker with English only (pseudo hidden). No visible change otherwise |
| 1a Web UI extraction | Convert the ~790 strings of calcpad-web; ribbon keys derived from data; `BLOCK_ACTIONS`, welcome page, dialogs, Monaco actions; date/time via `Intl`; fix concatenation sites (shortcut hints, plurals) | L | Same UI in English; every string behind a key |
| 1b Frontend extraction | The ~510 strings in calcpad-frontend (sidebar tabs, settings, metadata, export) through the hook; settings names and help text | L | Same UI in English. Do the two largest tabs last |
| 2 Engine codes | `code`/`args` in `MathParserException` and the wasm JSON; HTML `data-code`; 137 constructors; triage of ~270 uncatalogued literals; linter `args`; snippet id; docx/xlsx chrome table; `localiseMessages` at the edge; Rust tests for code stability; wasm rebuild | L | Engine output with codes, English unchanged. Can run alongside phase 1 but needs its own wasm release window |
| 3 First UI translations | Glossary, MT drafts for es/pt/fr/de/zh for UI and engine messages, review, screenshot pass, CJK font stacks, Monaco CJK test, Word `eastAsia` | M (plus reviewer time) | Languages that pass the gate become selectable in Settings. Others stay hidden |
| 4 Content overlays | Snippet descriptions and documentation (959 + 285), sample titles and descriptions (151) as engine catalogues; samples dialog search over translated text | M | Insert tab, completions, hover and samples dialog in the chosen language |
| 5 Landing pages | `pages.mjs` split into structure and text; per-language overlays and chrome files; render with `lang`; hreflang and alternates; sitemap index; checks looped by language; pilot with 10 pages in one language first | L | `/calcs/<lang>/learn/...` for ready languages; sitemap index; English pages gain alternates |
| 6 Guide | Gettext pipeline (or per-language copy), `build.sh` loop, `guide_seo` and `guide_url` language-aware, code-block identity check, per-language functions reference, search decision for `zh`, translation of ~39,000 words per language | L (pipeline) then L per language | `/calcs/<lang>/guide/...` for chapters in a language, starting with getting-started and units |
| 7 Small items | llms per language (S), What's new note on translated shells (S), manifest left English (none), translated video captions (later, M) | S | `/calcs/<lang>/llms.txt` |
| 8 Chinese in exports | Subset CJK font handed to `resvg`/PDF only when needed; Word language attributes; plot label test | M | Chinese plot labels and reproducible PDF |
| 9 Optional | Translated sample worksheets (44 sheets with prose, 7 templates), Traditional Chinese, RTL groundwork, localised landing slugs | L each | By demand |

Suggested release sequence: phase 0 and 1 land as invisible refactors, English-only, behind the existing checks (each release
is a no-op for users and proves the key coverage); phase 2 ships in a wasm release; phase 3 turns the first reviewed languages
on (suggest Spanish and German first or the two with reviewers ready); phases 4 to 6 follow per language.

## 6. Risks

| Risk | Where | Mitigation |
| --- | --- | --- |
| String concatenation builds sentences | `rename.ts` (count text from `plural()` pieces), `main.ts` unsaved-draft notice (`draft${s}`), `deskKey()` suffixes, `'Ran with ' + n + ' errors'` in the message bridge | Rewrite as whole-sentence keys with parameters; grep for `' + ` near UI sinks; pseudo-locale exposes the rest |
| E2E checks assert English text | 32 files, ~145 lines (1.7) | Default browser language keeps English; shared helper pins `?lang=en`; add `i18n-check.mjs` that boots each language and checks a handful of anchors by `data-testid`; prefer adding `data-testid` or `data-cmd` to controls that tests find by label |
| Text matched by regexes or compared to constants | engine HTML remapping only touches numbers and attributes (checked), but `Calculating` and similar strings are matched by 4 e2e lines; the doc-index and nav code match Calcpad keywords (must stay English) | Keep keywords and protected tokens in a list the check script knows; no code branches on translated text |
| Ribbon and highlighter snippets insert English into sheets | ribbon `text` / `sheet` inserts, some with English notes or headings; diagram starters have English node labels | Inserts stay canonical English in wave one; label and tooltip translate. Add per-language starter diagrams later as data |
| Compiled sample hashes | Only script-library SHA-384 pins exist (1.3) | Translating sample prose does not touch them; if golden compiled output is added later, keep it per language. Translated sample copies, if made, go through the same `check-examples` run |
| Fixed-width UI breaks with longer text | ribbon buttons, tab strip, sidebar tabs, block menu, German and French | Pseudo-locale with padding; `ribbon-layout-check` run in `de`; allow wrapping or ellipsis with `title` on the big buttons |
| Module-level constants hold English | `BLOCK_ACTIONS`, settings definitions, ribbon helpers | Resolve at render time through keys; the check script lists the remaining hard-coded sites |
| Shared frontend package friction | `calcpad-frontend` also serves other hosts and is merged with upstream | Hook with English fallback; wrap strings locally; do the two largest tabs last; keep a `git diff` ratio small |
| Merge conflicts with the `locale` branch | `setup.ts`, `transport.ts`, `main.ts`, Settings panel | Merge `locale` first; the language setting reuses its pattern |
| Merge conflicts with SEO work | `make_learn_pages.mjs`, `pages.mjs`, sitemap, `seo-check.mjs` | Section 3.8: split data, agree owner and order |
| Machine translation of engineering terms | all languages | Reviewer per language with field knowledge, glossary first, publish gate (4.2) |
| Stale translations | every release changes English | Source-hash tracking in the catalogue, coverage report in the release checks, PO fuzzy marks for the guide; stale is a warning for unpublished and a visible "English fallback" for published |
| Monaco built-in UI stays English | find, replace, command palette, context menus | Accept in wave one and say so in the help; evaluate a Monaco language pack later (not verified here) |
| Chinese search in the guide | mdBook search tokenizer | Decide: disable on `zh` or accept; do not promise search |
| Chinese in Word plots and PDF | `resvg` has no CJK font by default | Phase 8; document the limit until then |
| Bundle and offline size | catalogues grow with five languages | Public JSON per language, excluded from the install precache, cached at runtime; measured in phase 0 |
| Offline expectation | a language never opened online will not be cached | State it in Settings help; no per-language precache toggle in wave one |
| Trust: unreviewed numbers or claims in translated landing pages | comparison and "what this is not" notes | Translate the "not a design-code check" note verbatim with review; comparison pages last and optional |
| Dates, units and decimal separators in prose | translated landing pages quote engine numbers | Use the shared number format core; check script compares digits |
| Public repository hygiene | catalogues, glossary, commit messages | No internal names, hosts or paths; reviewer attribution by handle only |

## 7. Open questions for the owner

1. Portuguese: Brazilian only (`pt-BR`), or neutral `pt` with a note for Portugal? Affects the hreflang value and the glossary.
2. Spanish: neutral Latin American or Spain? (`es` alone cannot serve both engineering vocabularies well.) One catalogue with a
   regional glossary note is assumed.
3. Who reviews each language, and is paid review budgeted? The gate in 4.2 cannot be met without it.
4. Traditional Chinese: fall back to English, to Simplified, or build later? Default in this plan is English.
5. Localised slugs for landing pages (better local queries, more work and a redirect map) or English slugs under the language
   prefix (assumed)?
6. Where does the Worker that fronts the host live, and who can add the pass-through rule, the sitemap index and the root
   `robots.txt` line? Not visible from this repository.
7. Is it acceptable that What's new and the release notes stay English only?
8. Should a language be allowed to ship partially (UI translated, engine messages English) with an honest notice, or is every
   published language held to 100 percent coverage? The plan assumes 100 percent of UI keys and best-effort for engine messages
   not yet coded.
9. Is first-visit auto-switching to the browser language the intended behavior, or should the app stay English until a person
   picks a language (safer for people who keep a non-English OS but work in English)?
10. Chinese guide search: disable, or accept whole-word-only?
11. Should the desktop host (`tauri-bridge.ts`, 38 strings) be included? It is excluded from these counts.
