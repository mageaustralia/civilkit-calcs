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
  The engine wasm is 1.77 MB raw and 655 KB gzipped; it is built at opt-level `s`, which ran a heavy
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
  evaluations, functions, matrices, units, sums, integrals, programs (loops, early exits, grown arrays), plots, stepped
  ranges, complex numbers, simple solve blocks, statistics functions, referenced worksheets and pictures convert; what does not
  is shown as a typeset not-calculated equation or a static picture of the original, and counted in a summary. Two options
  tidy the text: plain layout wrappers dropped and equations written as plain Calcpad, and prose written as Markdown. See [docs/worksheet-import.md](docs/worksheet-import.md).
* **Rich-text editing:** double-click prose in a Markdown block (`#markdown`) or in a sheet and edit it as rich text
  (headings, lists, emphasis, pipe tables); untouched text is never rewritten. See [docs/rich-text-editing.md](docs/rich-text-editing.md).
* **Units once on arrays:** a vector or matrix whose elements share a unit is shown as `[16; 12; 3] ft` rather than with the
  unit on every element; `#settings {"unitPerElement": true}` restores the old display (DEVIATIONS.md, D13).
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
* **Symbolic algebra:** `Functions > Symbolic` simplifies, expands, factors, differentiates, integrates (indefinite or
  between limits), solves an equation or a linear system of up to four unknowns, substitutes and evaluates one expression.
  It is exact rational algebra from the `caskit` crate (`1/3` stays `1/3`; a result that does not fit 128 bits is an error),
  with documented limits: no complex numbers, only `sin`, `cos`, `ln`, `exp` and `sqrt`, a small class of integrands, and
  solving of degree 1 and 2 only. A refusal is shown in words. The result is typeset and can be copied or inserted into a
  worksheet or a sheet. The module (290 KB, 122 KB gzipped) loads only when the dialog first runs something. See
  [caskit/README.md](caskit/README.md) and [docs/caskit.md](docs/caskit.md).
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
that share a row keep their side by side layout. Plots, loops with early exits, grown arrays, complex numbers, solve blocks of
one or two unknowns (solved here and kept only when the equations hold, otherwise the value the file saved), statistics
functions and worksheets that a file references also convert. What does not convert (add-in libraries, embedded objects,
symbolic evaluation, programs that use constructs not yet mapped, some matrix plots) is shown as a typeset, not-calculated equation or a static picture of the
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
node e2e/symbolic-check.mjs       # Symbolic dialog: each operation, typeset results, errors, insert into a worksheet and a sheet, themes, phone width
node ../tools/set_version.mjs --check
```

## Checked at 26.10.16

On the release tree: `cargo test --workspace` (excluding `calcpad-sheet`, single-threaded) passed 6110 tests and `calcpad-sheet` 106, with no
failures (the known flaky `HPMatrixCond2` failed once in a first run and passed on the re-run). Browser checks on the release build, served
at the root: `sheet-check`, `inputs-check`, `ribbon-check` (with `--lint`), `ribbon-layout-check`, `fence-click-check`, `diagram-ribbon-check`,
`diagram-samples-check`, `mermaid-check`, `shortcuts-check` and `browser-check` (123 checks), and `verify-ribbon-snippets` (435 snippets run,
4 skipped, 0 failed). On the build with the `/calcs/` base: `browser-check` (123 checks; a first run failed two network checks that load
libraries from CDNs and passed on the re-run), `sample-link-check`, `pwa-check` (with `PWA_BASE=/calcs/`; the precache holds 600 files) and
`learn-check` (1739 checks over the 21 landing pages, 221 quoted engine values compared with a fresh run) all passed; `whats-new-check` passed.
`seo-check` (2925 checks, 1957 internal links followed) passed on the guide built with `book/build.sh`. The user guide's 134 examples all run
(`book/check-examples.sh`). `whats-new-browser-check` passed with the clips left out (posters only); with the clips, 5 of the 25 videos
(typing-an-input-line, insert-strip, edit-in-place, plots-gallery, phone-layout) did not play in headless Chromium, so those clips are not
verified. Not run for this release: `plot3d-check`, `symbolic-check`, `import-check`, `python-check` and the other smaller checks. Not tested:
a real phone, `tsc` beyond the build, `clippy` on the whole workspace, `cargo fmt --check`.

## Checked at 26.10.15

On the release tree: `cargo test --workspace` (excluding `calcpad-sheet`, single-threaded) passed 6216 tests and `calcpad-sheet` 106, with no
failures. Browser checks on the release build, served at the root: `sheet-check`, `inputs-check`, `ribbon-check` (with `--lint`),
`ribbon-layout-check`, `fence-click-check`, `diagram-ribbon-check`, `diagram-samples-check`, `mermaid-check`, `shortcuts-check` and
`browser-check` (123 checks), and `verify-ribbon-snippets` (435 snippets run, 4 skipped, 0 failed). On the build with the `/calcs/` base:
`browser-check` (46 checks), `sample-link-check`, `pwa-check` (with `PWA_BASE=/calcs/`; the precache holds 600 files) and `learn-check`
(744 checks over the ten landing pages, 97 quoted engine values compared with a fresh run) all passed; `whats-new-check` and
`whats-new-browser-check` passed. The user guide's 134 examples all run (`book/check-examples.sh`). Not run for this release: `plot3d-check`,
`symbolic-check`, `import-check`, `python-check` and the other smaller checks. Not tested: a real phone, `tsc` beyond the build, `clippy` on the
whole workspace, `cargo fmt --check`.

## Checked at 26.10.14

On the release tree: `cargo test --workspace` (excluding `calcpad-sheet`, single-threaded) passed 6110 tests and `calcpad-sheet` 106, with no
failures. Browser checks on the release build, served at the root: `sheet-check`, `inputs-check`, `ribbon-check`, `fence-click-check`,
`browser-check` (all checks) and, on the build with the `/calcs/` base, `browser-check`, `sample-link-check`, `pwa-check` (with `PWA_BASE=/calcs/`)
and `learn-check` (744 checks over the ten landing pages, 97 quoted engine values compared with a fresh run) all passed; `whats-new-check` and
`whats-new-browser-check` passed. The user guide's 104 examples all run (`book/check-examples.sh`). Not run for this release: `plot3d-check`,
`symbolic-check`, `import-check`, `python-check` and the other smaller checks. Not tested: a real phone, `tsc` beyond the build, `clippy` on the
whole workspace, `cargo fmt --check`.

## Checked at 26.10.13

On the release tree: `cargo test --workspace` (excluding `calcpad-sheet`, single-threaded) passed 6110 tests and `calcpad-sheet` 106, with no
failures (`HPMatrixCond2` passed in that run). Browser checks on the release build, served at the root: `inputs-check`, `sheet-check`,
`ribbon-check`, `md-edit-check`, `insert-strip-check`, `fold-key-check`, `editor-nav-check`, `slider-syntax-check`, `prose-toolbar-check`,
`block-menu-check`, `block-clipboard-check`, `collapse-drag-check`, `eigen-check`, `gen-eigen-check`, `mermaid-check`, `page-break-check`,
`table-sort-check`, `to-formula-ui-check`, `units-format-check`, `unsaved-check`, `block-hidden-check`, `whats-new-check`,
`whats-new-browser-check` and `pwa-check` all passed; `browser-check` passed 149 checks on the build with the `/calcs/` base (on the root-base
build only the "under /calcs/" check fails, as before). The pure checks `block-ops`, `to-formula` and `prose-format` passed. The user guide's
104 examples all run (`book/check-examples.sh`). Not run for this release: `plot3d-check`, `symbolic-check`, `import-check`, `python-check`
and the other smaller checks. Not tested: a real phone, `tsc` beyond the build, `clippy` on the whole workspace, `cargo fmt --check`.

## Checked at 26.10.12

On the release tree: `cargo test --workspace` (excluding `calcpad-sheet`, single-threaded) passed 6074 tests and `calcpad-sheet` 80, with no
failures. Browser checks on the release build: `browser-check` (all but one: the "under /calcs/" check needs a build with that base and
was run against the root-base build), `inputs-check`, `sheet-check`, `ribbon-check` (phone checks included) and `pwa-check` (offline
boot and calculation). The user guide's 85 examples all run (`book/check-examples.sh`). Not run for this release: `plot3d-check`,
`symbolic-check`, `import-check`, `python-check` and the other smaller checks. Not tested: a real phone, `tsc` beyond the build,
`clippy` on the whole workspace, `cargo fmt --check`.

## Checked at 26.10.11

On the release tree: `cargo test --workspace` (excluding `calcpad-sheet`, single-threaded, `--no-fail-fast`) passed 6125 tests with
no failures. Browser checks on the release build: `browser-check` (123), `pwa-check` (478 of 478 files cached, offline),
`ribbon-check` (phone checks included) and `mobile-longpress-check`. On the same code before the version bump: `plot3d-check`
(every 3D sample, light and dark), `symbolic-check`, `sheet-check`, `import-check`, `md-block-check`, `md-edit-check`,
`update-notice-check`, `c-style-check`, `legacy-input-check` and `select-map-check`. `python-check` passed 171 checks and failed one,
"a wasm that does not match the glue is refused": it fails the same way without the Python-limit change, because the check's
replacement module is not served to the worker that loads it, so that guarantee is not verified. Not tested: a real phone with its
on-screen keyboard, `tsc`, `clippy` on the whole workspace, `cargo fmt --check` (fails on `calcpad-sheet`).

## Checked at 26.10.10

On the merged tree: `cargo test --workspace` (excluding `calcpad-sheet`, single-threaded, `--no-fail-fast`) passed 6101 tests; the
two failures were `HPMatrixCond2` (flaky under load, known) and an importer list test and five highlighter snippet-count tests that
the new `polyroots` function changed; both were fixed and those two crates re-run green. The rest of the workspace was not re-run
after that fix. Browser checks on the release build: `sheet-check` (all), `browser-check` (123), `pwa-check` (472 files cached, offline),
`symbolic-check` (60), `import-check`, `md-block-check`, `md-edit-check`, `update-notice-check`, `ribbon-check` (phone checks included)
and `mobile-longpress-check`. Not tested: a real phone with its on-screen keyboard, `python-check`, `tsc`, `clippy` on the whole
workspace, `cargo fmt --check` (fails on `calcpad-sheet`).

## Checked at 26.10.9

On the release build: `cargo test --workspace` (excluding `calcpad-sheet`) passed, 6051 tests, run single-threaded because the
highlighter's `ui_directive_highlighter` tests fail intermittently in parallel (and `HPMatrixCond2` once under load; it passes alone).
`web/e2e/symbolic-check.mjs` (the Symbolic dialog: eight operations, errors, insert into a worksheet and a sheet, lazy loading),
`import-check.mjs` (65 checks), `md-block-check`, `md-edit-check`, `update-notice-check`, `ribbon-check`, `browser-check.mjs`
(123 checks) and `pwa-check.mjs` (offline, 470 files cached) pass. The caskit crate has 346 tests of its own, including the
simply supported beam and cantilever derivations. Not re-run for this release: `python-check`, `sheet-check`, `tsc`, `clippy`
on the whole workspace, and `cargo fmt --check` (fails on `calcpad-sheet`). The Word and Excel exports have not been opened in Office.

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
