# CivilKit Calcs User Guide

The user documentation of CivilKit Calcs, written as an [mdBook](https://rust-lang.github.io/mdBook/).

## Build and read

```sh
cargo install mdbook        # once
cd book
mdbook build                # writes ../target/book (a quick local build)
./build.sh                  # what is published: mdbook build, then ../tools/guide_seo.mjs (descriptions, canonicals, one h1)
mdbook serve --open         # serves on http://localhost:3000 and reloads on change
```

The search box in the toolbar searches the whole guide, and works offline in the built output.

## Check the examples

Every fenced block tagged `sheet` or `cpd` is run through the real engine, and a block that
makes the engine report an error fails the check. A block that shows an error on purpose is
tagged `text`. Python needs the page, so a sheet that holds a Python fence is tagged
`sheet python`: `check-examples.sh` compiles it and checks the Python syntax, and
`check-python.mjs` runs it in the built app (headless Chromium), answers the Python prompt and
fails on a Python error or an error line. The same script opens every sheet with input lines
(`L: ? 6 m` and its list, radio and checkbox forms, or `>> #UI`) in Input mode and counts the
controls: a sheet with `?` lines must draw exactly one control per line.

Two comment lines directly above a `sheet` fence tie an example to something checked:

```text
<!-- sample: Template simply supported beam.sheet -->   the block must equal that file of web/calcpad-web/public/sheet-samples
<!-- expect: M _max = ... = 56.26 kN · m | Bending: OK --> each fragment must appear in the result text (spaces ignored)
```

Use `--show` to print the result text to copy the fragments from.

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

The narration of the video storyboards in `videos/` is checked by `videos/lint-narration.sh`
(no digits, the word "live" or dashes, short sentences, the full text equal to the scene lines,
contiguous scene times, 60 to 90 seconds).

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
