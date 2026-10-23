# Python runtime for `#python` blocks

- `micropython/micropython.wasm` and `micropython/micropython.mjs`: MicroPython v1.24.1 (build dated 2024-11-30) compiled to
  WebAssembly with Emscripten, with its JavaScript loader. https://micropython.org, https://github.com/micropython/micropython.
  The version is the string embedded in the wasm. MicroPython is MIT licensed, Copyright (c) 2013-2024 Damien P. George;
  the licence file of the v1.24.1 release is included unchanged as `micropython/LICENSE` (copied from the upstream
  repository tag v1.24.1, which also lists third-party components of its various ports).

  The loader also carries Emscripten's runtime glue (MIT or University of Illinois/NCSA, as Emscripten is licensed).

- `lib/numpy/` and `lib/matplotlib/`: small stand-ins for the parts of numpy and matplotlib.pyplot that calculation scripts
  use. They are not the real packages and say so when a script reaches for something they lack (see docs/python-blocks.md).
  `lib/calcpad_py.py` is the host side of the block runner.

- `lib/numpy/linalg.py`, `lib/scipy/`: `numpy.linalg` and `scipy.linalg` for the stand-ins above. They are this project's own
  small modules, not the real packages; the numbers come from the Calcpad engine through `wasm/calcpad_linalg_wasm*`
  (built from `calcpad-linalg-wasm`, MIT, part of this repository), which a worksheet loads only when a block uses linear algebra.

- `lib/plotly/`: a thin `plotly.graph_objects` and `plotly.subplots` for `#python` blocks. This project's own code, not the plotly
  package: it builds the figure JSON and hands it to the page, which draws it with Plotly.js (MIT) loaded from its CDN through the
  worksheet library list, never bundled or copied here.
