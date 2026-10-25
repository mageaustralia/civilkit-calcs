# CivilKit Calcs

CivilKit Calcs is a browser-based engineering calculation sheet. You write a worksheet in the
CalcpadCE language (equations with units, vectors and matrices, plots, input forms, text and
tables), and the page calculates it, shows a formatted report, and exports it to PDF (by
printing), Word and Excel. Everything runs in the page: the calculation engine is Rust compiled to
WebAssembly, and no calculation request leaves the browser.

* Live demo: https://civilkit.io/calcs/ (also at https://mageaustralia.github.io/civilkit-calcs/)
* Source: https://github.com/mageaustralia/civilkit-calcs
* Version: see [CHANGELOG.md](CHANGELOG.md). Releases are numbered YY.M.N (26.10.1 was the first release of
  October 2026; this is 26.10.2) and tagged `v26.10.2`.

Status: early. The project is not affiliated with the authors of CalcpadCE.

## Credit: this is a derivative of CalcpadCE

CivilKit Calcs is built on [CalcpadCE](https://github.com/imartincei/CalcpadCE), the MIT-licensed
continuation of Calcpad by Nedelcho Ganchovski (Proektsoft) and contributors. This repository
contains:

* CalcpadCE's web frontend (`web/`, imported from `Calcpad.Web/frontend`, with changes), which
  provides the editor, sidebar, settings, the worksheet language's editor support and the report view;
* the CalcpadCE worksheet language and its example worksheets (the 101 bundled samples are copies
  of CalcpadCE's `Examples/`, see `web/calcpad-web/public/samples/NOTICE.md`);
* Rust reimplementations of CalcpadCE's calculation engine (Calcpad.Core), its editor language
  services (Calcpad.Highlighter: tokenizer, linter, prettifier, snippet library) and its Word
  export path (Calcpad.OpenXml). They follow the upstream module structure so that upstream
  changes can be followed file by file, and they are tested against upstream's own test classes
  and its expected-output files (see Tests below).

The upstream copyright and licence are kept in [LICENSE](LICENSE) and [NOTICE](NOTICE). The upstream
versions this work tracks are listed in NOTICE: the frontend 8.0.0-beta3-preview, the engine ported
from commit `0cf438f`. CalcpadCE's own version is tracked there, separately from the CivilKit Calcs version.

## Rust crates

The Rust libraries are published on crates.io (0.1.0, early; the API may change between minor versions):

* [`civilkit-calcs-engine`](https://crates.io/crates/civilkit-calcs-engine): the calculation engine.
* [`civilkit-calcs-highlighter`](https://crates.io/crates/civilkit-calcs-highlighter): tokenizer, linter, prettifier and snippets.
* [`civilkit-calcs-docx`](https://crates.io/crates/civilkit-calcs-docx): Word and Excel export.

Their Rust crate names are unchanged (`calcpad_core`, `calcpad_highlighter`, `calcpad_docx`), so `use calcpad_core::...` works
with `civilkit-calcs-engine = "0.1"` in `Cargo.toml`. The crates have their own 0.x semver, separate from the app's YY.M.N version.

## What we changed

* **Engine:** the Rust engine, compiled to WebAssembly, replaces the .NET engine and the server it
  runs in. The page answers the server API itself (`web/calcpad-web/src/wasm/transport.ts`).
  The engine wasm is 1.73 MB raw and 656 KB gzipped; it is built at opt-level `s`, which ran a heavy
  `$Map` sheet 1.5 times faster than the size-optimised build.
* **Editor services in a second wasm:** the highlighter, linter, prettifier, go-to-definition and
  the snippet library are a separate module (`calcpad-highlighter-wasm`, 351 KB gzipped) that loads
  on first use.
* **Word export:** `Export > Save Word` builds the .docx in the page (module `calcpad-docx-wasm`,
  loaded on first use). Equations are written as Office Math.
* **Excel export:** a Report sheet that mirrors the worksheet (headings and text, each equation as
  a row of name, expression, expression with values, value and unit, tables as cell ranges,
  `$Plot` as native editable charts with their data on a Plot data sheet, other drawings as
  pictures), then a Variables sheet, one sheet per vector or matrix, and an Inputs sheet.
* **`#UI` input forms:** entries, drop-downs, radio buttons, checkboxes and data grids. A chosen
  select, radio or checkbox is carried into the report, PDF and Word output.
* **Dark theme:** the preview keeps form controls, drawings and plots readable.
* **Import worksheet:** `File > Import worksheet...`, the Document tab's Import button, or dropping a `.xmcd` or `.mcdx` file on
  the page converts a worksheet of the spreadsheet-like format of another calculation tool into a new unsaved
  worksheet, locally in the page (its own WebAssembly module, loaded on first use). Text, definitions,
  evaluations, functions, matrices, units, sums, integrals, simple programs and pictures convert; what does not is
  shown as a typeset not-calculated equation or a static picture of the original, and counted in a summary. See [docs/worksheet-import.md](docs/worksheet-import.md).
* **Installable and offline:** a web app manifest and a service worker that caches the whole build (about 19 MB), so
  the page can be installed on a phone or desktop and opens and calculates without a connection. Plotly charts need the
  CDN script loaded once while online.
* **Samples browser:** 101 worksheets, all copied from CalcpadCE's examples (structural and maths),
  with groups, search and a preview. We rewrote their deprecated `? {default}` input fields as
  `#UI` declarations, and kept a conversion only where the engine renders it like the original.
* **Ribbon interface:** a tabbed ribbon (ten tabs, a quick-access strip) in place of the menu bar. Each
  palette opens a Common section above the whole snippet library, which the ribbon reads from the same
  highlighter call as the sidebar's Insert tab (1166 snippets, all reachable from the ribbon).
* **Model bindings:** a worksheet input can carry a `civilkit: bind` comment that ties it to a
  quantity of a structural model. The parser and the in-app display exist; connecting a model does
  not yet. The generic specification is in [docs/model-bindings.md](docs/model-bindings.md).
* **Worksheet scripts:** scripts in worksheets run only from a library list, under a
  Content Security Policy, with the print frame sandboxed (DEVIATIONS.md, D3).
* **Sheet syntax (our addition, not part of CalcpadCE):** the main way to write a worksheet here. Markdown
  prose, `name: expression -> unit | note` lines, LaTeX, indented `for` (with `step`), `while` and `if` blocks
  with `break` and `continue`, plots, and `>>` lines that pass raw Calcpad through. File > New and the quick-access New
  button open a sheet; New Calcpad worksheet opens the advanced syntax. A sheet compiles to the Calcpad worksheet language
  and runs on the same engine (a front end, not a second engine), so units, vectors, plots and the Word and
  Excel exports all work, and Show compiled / Convert to Calcpad give the Calcpad a person would write. The
  compiler is the crate `calcpad-sheet` in its own 39 KB gzipped wasm (`calcpad-sheet-wasm`); the preview
  renders LaTeX with the bundled KaTeX, loaded only when a sheet has a formula. The editor formats, completes,
  highlights, and goes to definition, finds references and hovers on a sheet's names; errors and lint markers
  are mapped back to sheet lines; the ribbon and Insert tab insert sheet forms. See
  [docs/sheet-syntax.md](docs/sheet-syntax.md).
* **Changelog:** a button on the Resources tab and a link in About open
  [CHANGELOG.md](CHANGELOG.md) on GitHub.
* **Known deviations** from CalcpadCE's behaviour are listed one by one, with the test that pins
  each, in [DEVIATIONS.md](DEVIATIONS.md). Not supported: raster (SkiaSharp) plots,
  `#ProjectPath`/`#LibraryPath` path roots, and compiled `.cpdz` worksheets.

Export testing: the Excel workbook opens cleanly in openpyxl and LibreOffice, but we have not tried it in
Microsoft Excel. The Word document has been checked in LibreOffice only, not in Microsoft Word. PDF output uses
the browser's print dialog, so it carries the browser's own header and footer.

## Layout

* `calcpad-core`: the engine. No threads; `#include` goes through a file-provider trait and
  `#read`/`#write` through `data_files::DataFiles` (the disk natively, the page's files in the
  browser). `openxml` reads and writes Excel cell ranges (zip and XML, no SDK). Module names follow
  the Calcpad.Core folders.
* `calcpad-cli`: the `Cli` binary, compatible with the batch mode of Calcpad.Cli
  (`Cli input.cpd output.html -b -s`, also `.docx` and `.xlsx` outputs).
* `calcpad-highlighter` and `calcpad-highlighter-wasm`: the editor language services and their
  WebAssembly binding. The snippet data is `calcpad-highlighter/src/snippets/snippets.json`.
* `calcpad-wasm`: the engine's WebAssembly binding (wasm-bindgen).
* `calcpad-docx` and `calcpad-docx-wasm`: the Word and Excel writers and their binding.
* `calcpad-import` (package `civilkit-calcs-import`, not published): translates a foreign worksheet format
  (`.xmcd`, `.mcdx`) into Calcpad text, with a report of what did not convert.
  See [Importing foreign worksheets](#importing-foreign-worksheets).
* `calcpad-sheet` and `calcpad-sheet-wasm`: the sheet syntax compiler and its binding.
* `web/`: the web app. `web/calcpad-frontend` is the shared frontend, `web/calcpad-web` the page
  (Vue, Monaco), `web/e2e` the browser checks.
* `tools/`: generators, the test runner, `build_wasm.sh`, `set_version.mjs`.
* `docs/`: [DEPLOY.md](docs/DEPLOY.md), the model-bindings specification and
  [the foreign worksheet importer](docs/worksheet-import.md).

## Importing foreign worksheets

`calcpad-import` reads the `.xmcd` and `.mcdx` files of the spreadsheet-like worksheet format of a commercial
calculation tool and writes Calcpad worksheet text for this engine, so existing sheets can be brought
over. It evaluates nothing itself; the engine does. Text (with its fonts, sizes and colours), definitions,
evaluations, functions, vectors and matrices, indexing (with the array origin shifted to Calcpad's 1), units,
sums, products, integrals, range variables, programs made of loops and conditions, the common built-in
functions, input-component values and pictures convert. The author's equations are shown as they were written
(stacked fractions, sums with bounds, subscripts, a transpose superscript) and computed out of sight, and regions
that share a row keep their side by side layout. What does not convert (plots, add-in libraries, some programs,
solve blocks, symbolic evaluation) is shown as a typeset, not-calculated equation or a static picture of the
original, and the report counts it by code.
The `.mcdx` container (a zip) is read by its own front end onto the same translator. Details, the mapping tables and how to run the batch
runner over a folder: [docs/worksheet-import.md](docs/worksheet-import.md).

```text
cargo run -p civilkit-calcs-import --example import -- sheet.xmcd
```

## Build and run

Needs Rust with the `wasm32-unknown-unknown` target, the wasm-bindgen CLI 0.2.122 (the version the
crate pins), node 22 and, optionally, `wasm-opt`.

```sh
tools/build_wasm.sh                       # web/calcpad-web/public/wasm
cd web && npm install && npm run build    # web/calcpad-web/dist
npm run preview -- --port 4173            # then open http://localhost:4173/
```

`npm run dev` serves the same with hot reload. The production build is static files only; it
runs from any folder of any static host. To build for a fixed path, set the base:
`CALCS_BASE=/calcs/ npm run build` (see [docs/DEPLOY.md](docs/DEPLOY.md)).

The command line engine: `cargo build --release`, then `target/release/Cli input.cpd output.html`.

## Tests

```sh
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
```

`cargo test` runs the xunit classes of CalcpadCE transcribed to Rust (`calcpad-core/tests/xunit_*.rs`,
regenerated with `tools/transcribe_all.sh`), the highlighter's transcribed tests and linter corpus,
and the deviation tests. The HP comparison classes are heavy; on a busy machine run
`python3 -I tools/run_tests.py [--minutes N]`, which stops every test process when the time limit
passes. Conformance against upstream's expected-output files (`*.html.stub`):

```sh
python3 -m venv .venv && .venv/bin/pip install beautifulsoup4==4.14.3
cargo build --release
.venv/bin/python -I tools/compare_stubs.py --ref ../calcpad-ref [--filter path] [--diff]
```

Browser checks (headless Chromium via Playwright, against the built `dist`):

```sh
cd web && npx playwright install chromium
node e2e/browser-check.mjs        # worksheets through the UI, input forms, files, exports, scripts, printing
node e2e/ribbon-check.mjs --url http://127.0.0.1:4173/ --lint
node e2e/verify-ribbon-snippets.mjs --cli ../target/release/Cli
node e2e/import-check.mjs         # Import worksheet: a synthetic .xmcd and .mcdx, pictures, toggles, error paths, drop, save, exports
node ../tools/set_version.mjs --check
```

## Checked at 26.10.8

On the merged tree: `cargo test --workspace` (excluding `calcpad-sheet`) passes, run single-threaded because the highlighter's
`ui_directive_highlighter` tests fail intermittently when run in parallel. `web/e2e/import-check.mjs` (63 checks),
`md-block-check`, `md-edit-check`, `update-notice-check`, `web/e2e/browser-check.mjs` (123 checks) and `web/e2e/pwa-check.mjs`
(5 checks) pass; in one run of `browser-check` the select-map check failed under load and passed alone and on a rerun. The
worksheet importer was re-measured on twelve local worksheets: no engine errors, except one in the truss. The Word and Excel
exports were checked as XML and by unit tests, not opened in Office. Not re-run for this release: `python-check`,
`sheet-check`, `tsc`, `clippy`, and `cargo fmt --check` (fails on `calcpad-sheet`).

## Checked at 26.10.7

On the release build: `web/e2e/import-check.mjs` (worksheet import through the menu, the ribbon and a drop, the error paths, and
the typeset equations), `web/e2e/browser-check.mjs` (123 checks, all bundled samples open without errors) and
`web/e2e/pwa-check.mjs` (installs, caches 419 files, boots and calculates offline) pass. `cargo test --workspace` passed
(5550 tests) before the last two small commits. Not re-run for this release: `python-check`, `sheet-check`, `tsc` (one
earlier error in `engine-worker.ts`), `clippy`, and `cargo fmt --check` (fails on `calcpad-sheet`, which was not formatted).
The Word and Excel exports of imported equations were checked as well-formed XML but not opened in Office.

## Checked at 26.10.6

`web/e2e/python-check.mjs` (169 checks: the earlier Python checks, the plane truss against hand values, linear algebra on the
engine, and the Plotly API: figure JSON, lazy loading, errors that name the unsupported call, the offline note, the Word and
Excel placeholder, a real chart render in light and dark, and clicks on the native dropdown and both sliders) and
`web/e2e/browser-check.mjs` (123 checks, all bundled samples open without errors) pass on the merged build, and `sheet-check`
passed on the Plotly branch. At 26.10.5, `cargo test` passed for `civilkit-calcs-sheet` (51 tests) and `calcpad-linalg-wasm` (13).
The Plotly chart loads Plotly.js from its CDN, so its checks need a network connection and are skipped without one.
Not re-run for this release: `cargo test` after the Plotly merge, `tsc`, `cargo fmt --check` and `clippy`, the full workspace
`cargo test`, and the ribbon, import and C-style check scripts. The Word and Excel exports have been opened in LibreOffice and
openpyxl, not in Microsoft Office.

## Contributing

Issues and pull requests are welcome. Keep changes to the engine behind a test; where behaviour
differs from CalcpadCE, record it in DEVIATIONS.md with the test that pins it. Run the gates above
before sending a change. To bump the version, see [docs/DEPLOY.md](docs/DEPLOY.md).

## Licence

MIT. See [LICENSE](LICENSE) (the upstream licence text and copyright lines are kept, with ours
added), [NOTICE](NOTICE) and [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md). Bundled fonts,
the spreadsheet widget and the sample worksheets carry their own licence files next to them.
