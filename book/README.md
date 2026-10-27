# CivilKit Calcs User Guide

The user documentation of CivilKit Calcs, written as an [mdBook](https://rust-lang.github.io/mdBook/).

## Build and read

```sh
cargo install mdbook        # once
cd book
mdbook build                # writes ../target/book
mdbook serve --open         # serves on http://localhost:3000 and reloads on change
```

The search box in the toolbar searches the whole guide, and works offline in the built output.

## Check the examples

Every fenced block tagged `sheet` or `cpd` is run through the real engine, and a block that
makes the engine report an error fails the check. A block that shows an error on purpose is
tagged `text`. Python needs the page, so a sheet that holds a Python fence is tagged
`sheet python`: `check-examples.sh` compiles it and checks the Python syntax, and
`check-python.mjs` runs it in the built app (headless Chromium), answers the Python prompt and
fails on a Python error or an error line. The same script opens every sheet with `#UI` input
lines in Input mode and counts the controls.

```sh
cargo build -p calcpad-cli -j 2      # the Cli binary, target/debug/Cli
tools/build_wasm.sh sheet            # the sheet compiler, web/calcpad-web/public/wasm
book/check-examples.sh               # all chapters
book/check-examples.sh --show book/src/units.md   # also print the result of each example as text
```

```sh
cd web && npm install && npm run build      # the built app, once
node book/check-python.mjs                   # Python examples and input forms (--dist DIR for another build)
```

Environment: `CALCPAD_CLI` is the path of the `Cli` binary (default `target/debug/Cli`, then
`target/release/Cli`), `SHEET_WASM_DIR` the folder with `calcpad_sheet_wasm.js` (default
`web/calcpad-web/public/wasm`).

## Layout

* `src/` the chapters, listed in `src/SUMMARY.md`.
* `src/reference/functions.md` is generated from the editor's snippet library by
  `gen-reference.py`; run it again when functions are added.
* `videos/` the storyboard and narration for each how-to video. The recorded files are named
  `src/videos/NN-name.mp4`, are not committed (they are ignored by git) and are linked from the
  matching chapter.
* `theme/css/civilkit.css` the accent colour.
