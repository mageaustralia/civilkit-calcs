# Python blocks (spike)

A worksheet can hold `#python` blocks. The page runs them on MicroPython (a 0.4 MB wasm) in a worker and mixes their results into
the worksheet: printed text, matplotlib figures as inline SVG, interactive Plotly charts, and numbers handed back as Calcpad variables. There is no 27 MB
full-Python download: MicroPython and small numpy and matplotlib stand-ins load only when a worksheet contains `#python`.

This is a spike on the `python-block` branch. It works end to end; the sections below say what it does not do.

## Syntax

```
"Beam
#python
import numpy as np
import matplotlib.pyplot as plt

x = np.linspace(0, 6, 61)
M = 12.0 * x * (6 - x) / 2
print("Maximum moment", float(M.max()))
plt.plot(x, M)
plt.show()

export(M_max=float(M.max()))
#end python
M_Ed = M_max*1kN*m
```

- `#python` on a line by itself (any case, blanks around are fine) opens a block. `#end python` on a line by itself closes it.
  No nesting. A block left open is an error line.
- All blocks of a worksheet run in order in one interpreter, so a later block sees the earlier blocks' variables and imports.
- Blocks run **before** the engine and unconditionally: a `#python` inside an `#if` still runs, and a block in an `#include`d file
  is not seen. Python is run only on the worksheet text itself.

### In a sheet

A sheet writes the block as a fenced code block with the language `python`; the sheet compiler turns it into the `#python` block
above and keeps the sheet's line numbers:

````
```python
import numpy as np
export(total=float(np.linspace(0, 1, 5).sum()))
```
total * 2
````

The fence's lines are not sheet syntax and are not renamed, formatted or highlighted as a sheet (the Python keeps its own
indentation, relative to the fence's block). A fence left open is an error on the sheet's last line and is closed there. A
Python error names the sheet line. In a sheet the ribbon's `#python` entries insert a `python` fence.

### What a block produces

| In the block | In the worksheet |
|---|---|
| `print(...)` | a monospace paragraph, HTML-escaped |
| `plt.show()` or `plt.savefig(...)` | an inline `<svg>` figure (SVG, never a raster image), drawn at the point where the figure was shown |
| `fig.show()` on a `plotly.graph_objects` figure | an interactive chart drawn by Plotly.js at that point (see "Interactive charts with the Plotly API") |
| `export(name=value, ...)` or `export("name", value)` | a Calcpad definition line per name: `name = 5.5`, `name = [1; 2; 3]` (a list or 1-D array), `name = [1; 2\| 3; 4]` (a list of lists or 2-D array) |
| an exception | one red error line naming the Python line and the exception type, and an entry in the error list on that line; the rest of the worksheet still calculates |

`export_data(name=value, ...)` is `export` for data: the names are defined but nothing is printed. Use it for large arrays that a
chart or script reads (the definitions are wrapped in `#hide` ... `#show`, so a `#hide` you opened yourself around the block is ended).

Variables are exported **explicitly**, not automatically: loop variables and arrays used for plotting would otherwise flood the
worksheet and collide with Calcpad names and units (`m`, `s`, `L`). `export` is a builtin; no import is needed. Values must be
numbers, bools, lists of numbers or numeric arrays; `nan` and `inf` are refused. Names must be letters (any script), digits and
underscores, not starting with a digit or underscore. Exported numbers are unitless: write `L = L_m*1m` after the block.
Several names in one `export` call arrive in an unspecified order (MicroPython does not keep keyword order); use separate calls
where the order matters.

### Interactive charts with the Plotly API

```python
import plotly.graph_objects as go
from plotly.subplots import make_subplots

fig = make_subplots(rows=2, cols=1, shared_xaxes=True, subplot_titles=["Moment", "Shear"])
fig.add_trace(go.Scatter(x=x, y=M, name="M", line_color="#1f77b4"), row=1, col=1)
fig.add_trace(go.Scatter(x=x, y=V, name="V"), row=2, col=1)
fig.add_vline(x=5, line_dash="dot")
fig.update_layout(title="Beam", height=520, xaxis2_title="x (m)")
fig.show()
```

A worksheet needs no HTML and no JavaScript for an interactive chart. `plotly.graph_objects` (also spelled `plotly.graph_objs`) and
`plotly.subplots` are small pure-Python modules under `public/py/lib/plotly/` (about 24 KB, 6 KB gzipped). They are fetched only for a
block whose source mentions `plotly`; a block that does not never downloads them.

**It is a pass-through, not a second copy of plotly.** Every keyword argument is accepted and written into the figure JSON as given;
Plotly.js in the page validates it. Numbers, strings, bools, `None`, lists, tuples, dicts and numeric arrays are sent (arrays become
lists, NaN and infinity become `null`, numbers are written with 9 significant digits). `dict(...)` and `{}` are both fine.

| Supported | Notes |
|---|---|
| `go.Figure(data=, layout=, frames=)` | `data` takes traces or dicts |
| `go.Scatter`, `Scattergl`, `Bar`, `Histogram`, `Box`, `Violin`, `Pie`, `Heatmap`, `Contour`, `Surface`, `Scatter3d`, `Mesh3d`, `Scatterpolar`, `Waterfall`, `Indicator`, `Table`, `Candlestick` | the class only sets `type`; all other keywords are the trace's own |
| `go.Layout`, `go.Frame` | dict wrappers |
| `fig.add_trace(trace, row=, col=)`, `add_traces`, `add_scatter(...)`, `add_bar(...)` and the like | `row` and `col` need a figure from `make_subplots` and set the trace's `xaxis` and `yaxis` |
| `fig.update_layout(...)`, `update_xaxes(...)`, `update_yaxes(...)`, `update_traces(...)` | recursive merge, as in plotly; `row`/`col` pick subplot axes; `update_traces(selector=dict(name="M"))` matches by equality |
| `fig.add_shape(...)`, `add_annotation(...)`, `add_vline(x)`, `add_hline(y)`, `add_vrect(x0, x1)`, `add_hrect(y0, y1)` | with `row`/`col` (default all subplots) the lines use that subplot's axes |
| `fig.show(config=dict(...))` | sends the figure to the report |
| `fig.to_dict()`, `fig.to_json()` | |
| `plotly.subplots.make_subplots(rows, cols, shared_xaxes, shared_yaxes, subplot_titles, row_heights, column_widths, vertical_spacing, horizontal_spacing)` | shared axes are `True`, `"columns"`, `"rows"` or `"all"` |
| `updatemenus` (dropdowns, buttons) and `sliders` with precomputed steps | any `method` (`restyle`, `update`, `relayout`, `animate`); `frames` for `animate` |

Underscore names ("magic underscores") are expanded when they are keyword arguments of a call: `line_color="red"` is `line=dict(color="red")`,
`xaxis_title="x"` is `xaxis=dict(title="x")`, `marker_line_width=1` is `marker=dict(line=dict(width=1))`, and `paper_bgcolor`, `plot_bgcolor`
and `error_x/y/z` stay as they are. Inside a dict they are not expanded, so a key such as `"line_color"` there raises `ValueError` naming it
(Plotly.js would drop it without a word). A property name that Plotly does not have is not checked here: Plotly.js ignores it, as it does
in a plain browser.

**What fails, by name.** Anything the shim does not provide raises `NotImplementedError` (or the error shown) naming it:

| You write | You get |
|---|---|
| `go.Sunburst` (any trace type not in the table), `from plotly.graph_objects import Foo` | `NotImplementedError: plotly.graph_objects.Sunburst is not available in the light plotly shim (traces: ...)` |
| `fig.write_html(...)`, `fig.write_image(...)`, `fig.data`, `fig.layout`, any other attribute | `NotImplementedError: go.Figure.write_html is not available ...` |
| `fig.show(renderer=...)` or any option other than `config` | `NotImplementedError: Figure.show(renderer=...) is not available ...` |
| `make_subplots(specs=..., secondary_y=..., ...)` | `NotImplementedError: plotly.subplots.make_subplots(specs=...) is not available ... (supported: ...)` |
| `add_vline(x, annotation_text=...)` | `NotImplementedError ... add the label with add_annotation` |
| `update_layout(template="plotly_white")` | `ValueError: ... named templates are not available; the report page applies a light or dark theme itself` |
| `go.Scatter(x=[object()])` | `TypeError: go.Scatter x: a object cannot be sent to Plotly ...` |
| `fig.foo = 1`, `trace.y = ...` | `AttributeError ... cannot set 'foo'` (a plotly object is not a property tree here) |
| `import plotly.express`, `plotly.io` | `ImportError: no module named 'plotly.express'` |

**Where it is drawn.** `fig.show()` sends the figure JSON through the same path as a matplotlib figure and the page draws it with Plotly.js at
that point of the report. Plotly.js is **not bundled**: it is the "Plotly" entry of the worksheet library list (Settings > Libraries), loaded
from its CDN (`cdn.plot.ly`, version 2.26.0) only when a figure is on the page, with the same pinned URL, `integrity` hash and
`crossorigin="anonymous"` as a worksheet's own `<script src>`; the page's Content-Security-Policy is unchanged (it allows exactly that file,
and chart scripts make no network requests). The first figure of a page downloads about 3.6 MB.

- **Offline.** Without a network connection (or when Plotly was removed from the library list) the chart's place shows a note instead of a
  blank: "Interactive chart: TITLE. Interactive charts need a connection to load Plotly...". The Python solve and everything else still run.
- **Light and dark.** The page applies a Plotly template: transparent paper and plot area, text, grid, zero line, menus, sliders and modebar
  in colours for the page's light or dark theme. A property the figure sets itself (`paper_bgcolor`, `font`, an axis `gridcolor`, ...) wins.
  The chart does not get the white card the dark theme paints behind other drawings.
- **Print, PDF, Word and Excel.** None of them can run Plotly.js, so the figure is not carried. The worksheet shows a paragraph where the
  chart stood: "Interactive chart: TITLE. It is drawn in the browser and is not carried by print or export." (the title is the figure's
  `layout.title`). If a printout needs the picture, draw a static copy with matplotlib as well, as the other Python samples do. The Code
  ("Unwrapped") view lists the text the engine reads.
- **Size.** The figure travels inside the worksheet text (base64), capped at 4 MB of figure JSON per run and 20 figures and drawings together.
  Precomputed slider steps are the big part: the beam sample's 31 load factor steps make its figure 190 kB. A control that rescales a diagram
  needs one set of y values per step.

**Controls are Plotly's own, and they are independent.** A dropdown or slider step is a fixed `restyle`, `update` or `relayout` (or an
`animate` of a frame) that Plotly applies when it is used; it cannot read the state of another control or run Python, so a figure with a
menu, a slider for a load factor and another for a position has to give each control its own traces and leave the combinations
unexpressed, or precompute them (one step per combination). The beam sample does the first: the menu chooses which uniform load case is
shown, the factor slider restyles the y data of those traces, and the position slider moves a separate point-load trace. For controls that
must combine freely or compute a readout on every change, write the chart as raw HTML and JavaScript (next section).

### Interactive panels (prototype)

`import ui` gives a block sliders, drop-downs, checkboxes and players whose changes run one Python function again and redraw its chart,
table or text in place, without recalculating the sheet:

```python
import ui
L = ui.slider("L", 6, 2, 12, unit="m")

@ui.panel(L)
def beam(L):
    return "span %g m" % L
```

The design, the semantics (state, time limits, exports, saved values), the security model and the three example sheets are in
[interactive-panels-design.md](interactive-panels-design.md). Print, Word and Excel carry the panel's first output and a note with the
values it was left at, not the widgets.

### Python results driving a chart written in JavaScript (raw)

The raw route is still available, and the sample "Beam Explorer: Python Results in Raw JS Plotly" shows it. `export_data(M_u=..., x_s=...)`
defines vectors and matrices; in a Calcpad text line inside a `#val` ... `#equ` region, `'var X = 'x_s';` prints the value as a JavaScript
array literal (`[1.5, 2, 3.25]`, and `[[1, 2], [3, 4]]` for a matrix). A `<script>` there can therefore read the Python results directly,
with Plotly or d3 loaded from the worksheet library list (the page's policy allows exactly those files, with their integrity hashes, and
no network requests from scripts). Write the script with double quotes only: a single quote in a Calcpad text line switches between text
and expression.

What is not possible either way: a control in the chart cannot re-run the Python. Python runs once, before the engine, when the worksheet is
calculated. A chart can switch between results the solve already produced (a menu, a slider over precomputed cases); with a script it can
also recompute anything cheap from the exported numbers. The raw sample labels which is which: its load factor slider multiplies the
exported diagrams (exact for a linear analysis) for whichever case is chosen, and its point load slider picks one of 15 solves done in Python.
Use the raw route only for what the native controls cannot do (controls that combine, a computed readout); it needs the `<script>` library
entry and shows nothing in Word or Excel.

### One way only

Python cannot read Calcpad variables yet. Doing it cheaply is possible: run the engine on the text before the block, read the
numeric variables from its answer, and put them into the block's globals. It costs one extra engine pass per block and needs
the engine to return variable values for a partial worksheet, so it is not built.

## Linear algebra

`numpy.linalg` and `scipy.linalg` are real: the numbers come from the Calcpad engine, the same code that evaluates `lsolve`,
`inverse`, `det`, `cholesky`, `eigenvals` and `svd` in a Calcpad line, so a Python block and a Calcpad line give one answer
(`calcpad-linalg-wasm`, a separate wasm module the worker loads on the first block that mentions `linalg`, `scipy`, `dot`,
`matmul` or `@`; a block that does none of these never downloads it).

```python
import numpy as np
K = np.array([[4.0, -2.0], [-2.0, 3.0]])
u = np.linalg.solve(K, np.array([10.0, 0.0]))     # [3.0, 2.0]
export(u1=float(u[0]), u2=float(u[1]))
```

| Call | Notes |
|---|---|
| `np.linalg.solve(a, b)` | `b` a vector or a matrix of columns |
| `np.linalg.inv(a)`, `det(a)` | |
| `np.linalg.cholesky(a)` | the lower factor `L`, `a = L @ L.T`; reads the lower triangle, as numpy does |
| `np.linalg.eigh(a, UPLO="L")`, `eigvalsh(a)` | symmetric matrices; eigenvalues ascending, eigenvectors as columns |
| `np.linalg.eig(a)`, `eigvals(a)` | symmetric matrices only (the engine's eigen solver is symmetric); ascending order, which numpy does not promise |
| `np.linalg.svd(a, full_matrices=False)` | `u, s, vt`; `full_matrices=True` only for square `a`; `compute_uv=False` gives `s` |
| `np.linalg.matrix_rank(a)`, `cond(a)`, `pinv(a)`, `norm(x, ord)` | `norm` takes no `axis`; `cond` is the 2-norm one |
| `scipy.linalg.solve`, `inv`, `det`, `eigh`, `svd` | same engine; `assume_a` is accepted and does not change the answer |
| `scipy.linalg.cholesky(a, lower=False)` | the upper factor by default, as scipy does |
| `scipy.linalg.cho_factor`, `cho_solve` | `cho_solve((c, lower), b)` |
| `scipy.linalg.solve_triangular(a, b, lower=, trans=)` | reads only the stated triangle; `unit_diagonal=True` is not available |

What differs from numpy, and what you see:

- Eigenvector and singular vector signs are the engine's, and numpy's LAPACK may choose the opposite sign. Check results that do
  not depend on the sign (`a @ v` against `w * v`), not the signs themselves.
- A singular or badly scaled matrix raises `LinAlgError` (a subclass of `ValueError`, as in numpy) with the engine's message,
  for example `LinAlgError: Matrix is singular.`, and the worksheet shows it on the Python line that made the call. A matrix that
  is not positive definite does the same for `cholesky`. NaN or inf in an input is `LinAlgError: Array must not contain infs or NaNs`.
- Anything else raises `NotImplementedError` naming the function (`numpy.linalg.lstsq is not available in the light Python
  stand-in`, `numpy.linalg.eig of a non-symmetric matrix (the engine's eigen solver is symmetric only; ...)`). There is no
  silent substitute. Stacked matrices (3-D input) are not supported.
- Large products use the engine too: from about 46 by 46 times 46 by 46 (100 000 multiply-adds), `a @ b` and `np.dot` of float
  matrices are done by the engine, with the same values as the Python loop (checked value for value). Smaller products stay in Python.
- Values are float64 and unitless. Matrices cross to the engine as packed 64-bit floats in hexadecimal text (exact; decimal text
  took seconds for a 240 by 240 matrix).
- The module is loaded before a block runs, from the block's text. A call that Python builds by name (`getattr(np, "lin" + "alg")`)
  finds no module loaded and raises `LinAlgError: the linear algebra module is not loaded (write numpy.linalg or scipy.linalg in the block itself)`.
  If the download fails, or the module does not match the page (a stale cached file), the error is
  `LinAlgError: the linear algebra module did not load: ...`; the wasm is always revalidated (see below) and checks an interface
  version, so it is refused rather than answering differently.

### Speed

Solving `A x = b` for a dense `n` by `n` matrix, best of three, in Node on a busy 8-core Mac (load average about 10, so absolute
numbers are rough; the ratios are the point). "Pure Python" is a hand-written Gaussian elimination, which is what a script would
need without linalg. The engine column is `np.linalg.solve` including packing, the call and unpacking:

| n | pure Python | `np.linalg.solve` (engine) | ulab `inv` and `dot` (measured, not shipped) |
|---|---|---|---|
| 60 | 186 ms | 20 ms | 1 ms |
| 120 | 1 465 ms | 87 ms | 6 ms |
| 240 | 11 143 ms | 503 ms | 52 ms |

The same matrix product, 60 by 60: 761 ms in the Python loop, about 220 ms through the engine; 120 by 120: 7.0 s in the loop,
about 1.0 s through the engine, 5 ms in ulab.

### Why not ulab

[ulab](https://github.com/v923z/micropython-ulab) (MIT) is a numpy-like C module for MicroPython. It was built and measured
(v6.12.1 in MicroPython v1.24.1, float64) and is **not shipped**:

- It is 10 times faster than the engine bridge, but it is a different numpy: at most 2-D arrays, no `@`, no `.sum()` method, no
  `cumsum`, `prod`, `var`, `average`, `isclose`, `ones_like` or `abs`, and different indexing and printing. Hiding that behind the
  stand-in without visible seams would mean keeping the stand-in's own array class anyway and converting on every call.
- The wasm grows: a MicroPython 1.24.1 built with the current Emscripten and ulab (complex, fft, linalg and file input and
  output switched off) is about 820 KB, against the 389 KB wasm now shipped (the shipped one is an earlier toolchain's build;
  the same MicroPython and settings, built without ulab, are 588 KB), and the garbage collector needs the asyncify transform, so
  the interpreter cannot be made smaller by dropping it.
- It would be a second source of numerics next to the engine.

The engine is the one source of numerics for Calcpad lines, sheets and Python blocks, and the download stays at the shipped
size plus 138 KB gzipped, only when linear algebra is used. If matrix work at n of a few hundred in Python becomes common, a
ulab build is the faster route, at the cost above.

## The editor

`calcpad-highlighter` treats a block as not Calcpad (`src/python_blocks.rs`): its lines are not tokenized, linted or prettified;
only the `#python` and `#end python` lines are marked as keywords. A name the block exports is defined for the lines after the
block, at the line of its `export` call (the linter sees `name = 0` there), so later lines are not flagged as undefined. Names
built dynamically (`export(**d)`) cannot be seen and will be flagged. An unclosed block is an error (`CPD-3801`). The prettifier
leaves the block byte for byte.

## How it runs

```
editor -> /api/calcpad/convert (also docx, xlsx)
          transport.ts: find blocks -> ask once per tab -> python-runner.ts -> python-worker.ts (MicroPython)
          replace each block by its results -> engine -> map line numbers back -> page
```

- A block's replacement keeps the block's place in the worksheet, but not its line count. The engine's answer is renumbered
  afterwards (`data-source-line`, `data-ui-line`, error links, the error list) so click-to-source, `#UI` write-back and error
  markers use the worksheet's own line numbers (`remapPage`, `remapErrors` in `python-blocks.ts`).
- Preview, Report, Print PDF, Word and Excel all go through the same transport, so all of them see the substituted result, figures
  included. The Code ("Unwrapped") view lists the substituted text.
- Results are cached by the text of the blocks (last 8), so typing in Calcpad lines does not rerun Python. Each calculation that
  does run gets a fresh interpreter.
- Limits: 10 s per block (the worker is killed and restarted), 30 s per calculation (a block that runs out of time asks once per tab whether to run again with 60 s per block and 180 s per calculation, for a slower device such as a phone), 100 000 characters of print output, 20 figures,
  3 MB of SVG, 500 exported names, 32 MB interpreter heap.

### Lazy loading

Nothing of Python is fetched for a worksheet without `#python` (`e2e/python-check.mjs` checks the network log). With one, after the
confirmation below: the worker chunk, `py/micropython/micropython.{mjs,wasm}` and the seven files under `py/lib` (the plotly files are in
`py/files.json` too, and are fetched only when a block mentions plotly).

| File | Raw | Gzip |
|---|---|---|
| `micropython.wasm` | 389 485 | 181 710 |
| `micropython.mjs` | 104 975 | 30 027 |
| `lib/numpy/_array.py` + `__init__.py` | 56 513 | 15 537 |
| `lib/matplotlib/*`, `calcpad_py.py`, `_route.py` | 18 993 | 5 960 |
| `lib/plotly/*.py`, only for a block that mentions plotly | 23 665 | 6 263 |
| `python-worker` chunk (includes the SVG chart code) | 16 030 | 6 263 |
| `python` chunk (runner, consent dialog, block text code) | 4 204 | 1 810 |
| **Total on first use** | **about 590 000 (0.56 MB)** | **about 240 000** |
| `numpy/linalg.py`, `scipy/*.py` (with the files above, when a block imports them) | 10 211 | about 3 500 |
| `wasm/calcpad_linalg_wasm_bg.wasm` and its glue, only for a block that mentions linalg, scipy, dot, matmul or `@` | 352 257 | 139 880 |

(The seven Python files together gzip to 20 036 bytes. The sizes are of the build on this branch.)

## Consent and sandbox

Calculating a worksheet executes its blocks, so the first run in each open tab asks "This worksheet runs Python code" (Run Python
or Not now). Run is remembered for that tab. Not now is remembered until the worksheet's Python code changes; blocks then show a
"Python was not run" line and nothing loads. The Samples browser never runs a sample's Python: a Python sample's preview shows
its source, and opening it asks as above.

What stops a script:

- It runs in a Web Worker: no DOM, no cookies, no `localStorage`.
- MicroPython's own `js` and `jsffi` modules (which would hand it the JavaScript global object) are refused by an import guard
  installed before the script runs; the guard and the host object live in closures a script cannot reach. `_thread` and the
  module that carries the host object are refused too.
- After loading, the worker's `fetch` serves only the interpreter's wasm from memory, and `XMLHttpRequest`, `WebSocket`,
  `EventSource`, `indexedDB`, `caches`, `BroadcastChannel`, `Worker` and `SharedWorker` are removed, and `importScripts` refuses.
- Messages from the worker carry a per-run token; the files MicroPython can open are an in-memory file system holding only the stand-in modules.
- A runaway loop or allocation is stopped by the timeout or the heap limit.

What it does not do, honestly:

- It is not a security boundary against a determined attacker. The guard is a Python-level check on `import`; a way around it that
  we have not found would give a script the worker's global object. The worker is same-origin with the app, and the app page sets no
  Content-Security-Policy, so a worker CSP is not there to back this up. The Cloudflare deployment adds one to the worker script itself
  (`cloudflare/_headers`: same-origin `connect-src` and `script-src`), which stops a script that got past the guard from sending data to
  another host or importing code from one. A worker keeps the policy of the response that served its script, so GitHub Pages, which
  cannot set headers, and a local `vite preview` have no such backstop.
- The existing CSP of the rendered worksheet is unchanged: inline SVG needs nothing from it. A Plotly chart adds one `<script src>` for the
  pinned Plotly.js file, which the policy already allows for the library list, and a small inline script that draws the figure. Worker and wasm are served from the app's
  own origin, no CDN.
- A script can use the CPU for its time budget and the memory for its heap.

## Compatibility with typical calculation scripts

Probed by running small scripts on the shipped stand-ins (24 probes; the numbers are what is in `public/py/lib`).

| Area | Status | Notes |
|---|---|---|
| numpy: `array`, `shape`, `T`, `sum`, `mean`, `max`, `argmax`, `cumsum`, `abs`, `clip`, `maximum`, `where`, `sqrt`, `exp`, `sin`, `cos` | works | numpy's answers; slices are views |
| `linspace`, `arange`, `zeros`, `ones`, `full`, element assignment, boolean masks | works | |
| array arithmetic and `**`, `@`, `np.dot`, scalar on either side | works | float on the left needed a fix (below) |
| `np.diff`, `np.trapz` / `trapezoid`, `np.eye` | works | added in the spike (about 25 lines) |
| `np.linalg`: `solve`, `inv`, `det`, `cholesky`, `eigh`, `eigvalsh`, `svd`, `matrix_rank`, `cond`, `pinv`, `norm`; `scipy.linalg`: `solve`, `inv`, `det`, `cholesky`, `cho_factor`, `cho_solve`, `solve_triangular`, `eigh`, `svd` | works | computed by the Calcpad engine, see "Linear algebra" below |
| `np.linalg.eig` of a non-symmetric matrix, `lstsq`, `qr`, `lu`, `matrix_power`, stacked (3-D) matrices | missing | `NotImplementedError` naming the function |
| `np.vstack`, `hstack`, `reshape`, `meshgrid`, `sort`, `unique`, `interp`, `polyfit`, `random`, `cross`, `outer` | missing | raise `NotImplementedError` naming the function |
| math, f-strings, comprehensions, lambdas, dicts, classes, `time`, `random` | works | MicroPython 1.24 |
| `float(0-d array)` | missing | removed so that `2.5 * array` works |
| matplotlib: `plot` (format strings, labels), `xlabel`, `ylabel`, `title`, `legend`, `grid`, `xlim`, `ylim`, `figure(figsize)`, `show`, `savefig` | works | |
| `scatter`, `text`, `axis("equal")`, log axes, `subplots`, `ax.plot`, `set_title`, `tight_layout`, `suptitle` | works | |
| `bar`, `fill_between`, `axhline`, `axvline` | works | added in the spike |
| `hist`, `pie`, `errorbar`, `step`, `annotate`, `imshow`, `contour`, `colorbar`, `set_xticks`, `semilogy`, `twinx`, 3-D, polar | missing | |
| `plotly.graph_objects`, `plotly.graph_objs`, `plotly.subplots.make_subplots` | works (thin) | pass-through to Plotly.js, see "Interactive charts with the Plotly API"; `plotly.express`, `plotly.io`, other trace types, `write_html`: missing, named errors |
| `input()` | refused | there is no keyboard |
| `import js`, `jsffi`, network, host files | refused | see the sandbox |

Missing features by importance for calculation worksheets: (1) `vstack`/`hstack`/`reshape`/`meshgrid`/`sort`;
(2) `hist`, `annotate`, `twinx`, `set_xticks`, `step`; (3) `interp`/`polyfit`; (4) `lstsq`, `qr`, non-symmetric `eig`; (5) `contour`/`imshow`/`colorbar`; (6) `errorbar`, `pie`.

## What was changed in the copied stand-ins

The numpy and matplotlib stand-ins are the author's own small subsets that mimic those packages' APIs (the real, MIT-licensed
packages are not copied). MicroPython's licence is in `THIRD_PARTY_LICENSES.md`. Changes made for this repository:

- error messages and docstrings reworded to stand alone;
- the figure recorder (`_chart.py`) now hands a plain specification to the page, whose own SVG code (`src/wasm/python-chart.ts`)
  draws it; contours and colour bars were removed from the recorder rather than left half-working; `bar`, `fill_between`, `axhline`,
  `axvline` and `suptitle` were added;
- `ndarray.__float__` and `__int__` were removed: MicroPython's float operators call them on the right operand and give up when
  they raise, so `2.5 * array` failed;
- `diff`, `trapezoid`/`trapz` and `eye` were added.

## Not done

- Python reading Calcpad variables.
- Automatic export of variables.
- A friendly-sheet ` ```python ` fence (that front end is not on this base).
- A Python-aware highlighter for the block body (it is shown as plain text).
- Auto-run is not persisted anywhere: opening a worksheet runs nothing until the editor calculates it, and the question comes first.
