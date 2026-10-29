# Python blocks

<!-- description: Run small Python programs inside a sheet: numpy-style arrays and linear algebra, matplotlib-style and Plotly-style charts and 3D plots, sandboxed. -->

<p><strong>Video:</strong> Plots from Python, Plotly and matplotlib style</p>
<video controls preload="none" poster="videos/04-plots-from-python.jpg" style="width:100%;max-width:960px" src="videos/04-plots-from-python.mp4"></video>

A sheet can hold small Python programs. They run in the page, in a sandbox, on a compact Python
interpreter (MicroPython) with small stand-ins for numpy, matplotlib and Plotly. What a block prints,
draws and exports becomes part of the sheet, and the numbers it exports can be used by the
quantity lines that follow.

Nothing is downloaded for a sheet without Python. The first time a sheet with a Python block is
calculated, the page asks "This worksheet runs Python code" (**Run Python** or **Not now**). The
answer is remembered for that tab. About 0.6 MB is fetched the first time and cached afterwards.

## The block

A Python block is a fenced code block with the language `python`:

````sheet python
# Moment diagram with matplotlib

L: 6 m | span
w: 12 kN/m | design load

```python
import numpy as np
import matplotlib.pyplot as plt

L_m, w_k = 6.0, 12.0
x = np.linspace(0, L_m, 61)
M = w_k * x * (L_m - x) / 2

print("Maximum moment", round(float(M.max()), 2), "kN.m at", round(float(x[np.argmax(M)]), 2), "m")

plt.figure(figsize=(7, 3))
plt.plot(x, M, "b-", label="M(x)")
plt.fill_between(x, M, alpha=0.2)
plt.xlabel("x (m)")
plt.ylabel("M (kN.m)")
plt.title("Simply supported beam")
plt.grid(True)
plt.legend()
plt.show()

export(M_peak=float(M.max()))
```

Mmax: M_peak * 1 kN*m
Mcheck: w * L^2 / 8 -> kN*m
````

The result shows the printed line, the figure where `plt.show()` was called, and then the two
quantity lines:

```text
Maximum moment 54.0 kN.m at 3.0 m
[figure: the parabolic moment diagram]
M_peak = 54
Mmax = M_peak · 1 kN·m = 54 · 1 kN·m = 54 kN·m
Mcheck = w·L²/8 = 12 kN/m · (6 m)² / 8 = 54 kN·m
```

The two numbers agree, which is the point of mixing: Python draws and explores, the sheet
carries the units and the check.

| In the block | In the sheet |
| --- | --- |
| `print(...)` | A monospace paragraph. |
| `plt.show()` or `plt.savefig(...)` | An inline SVG figure at that point. |
| `fig.show()` on a Plotly figure | An interactive chart at that point. |
| `export(name=value, ...)` | A definition `name = value`, a vector for a list or 1-D array, a matrix for a list of lists. |
| `export_data(name=value, ...)` | Like `export` but nothing is printed; for large arrays. |
| An exception | One red error line naming the Python line and the exception type. The rest of the sheet still calculates. |

Rules worth knowing:

* All blocks of a sheet run in order in one interpreter, before the engine. A later block sees
  the earlier blocks' variables.
* Variables are exported **explicitly**, so a loop variable never collides with a unit name such
  as `m` or `s`. Exported numbers are unitless; attach the unit in the sheet, as `Mmax` does above.
* Names in `export` are written the way the sheet spells them: `export(sigma_c=2)` is σ_c below the
  block.
* Python cannot read the sheet's quantities. Pass numbers in by writing them in the block.
* Several names in one `export` call arrive in no promised order. Use separate calls where
  order matters.

## Which numpy and matplotlib

The stand-ins are small subsets that mimic those packages. What works:

* numpy: `array`, `linspace`, `arange`, `zeros`, `ones`, `full`, arithmetic and `**`, `@`, `dot`,
  `sum`, `mean`, `max`, `argmax`, `cumsum`, `abs`, `clip`, `maximum`, `where`, `sqrt`, `exp`,
  `sin`, `cos`, `diff`, `trapz`, `eye`, boolean masks.
* numpy.linalg and scipy.linalg: `solve`, `inv`, `det`, `cholesky`, `eigh`, `svd`, `matrix_rank`,
  `cond`, `pinv`, `norm`. The numbers come from the same engine as the sheet's `lsolve`, `inverse`
  and `det`.
* matplotlib: `plot`, `scatter`, `bar`, `fill_between`, `axhline`, `axvline`, `text`, `xlabel`,
  `ylabel`, `title`, `legend`, `grid`, `xlim`, `ylim`, `figure(figsize)`, `subplots`, `tight_layout`,
  `suptitle`, `axis("equal")`, log axes, `show`, `savefig`.

Missing functions raise a named error rather than a wrong answer. Not available: `vstack`, `hstack`,
`reshape`, `meshgrid`, `sort`, `interp`, `polyfit`, `random` in numpy; `hist`, `pie`, `errorbar`,
`step`, `annotate`, `imshow`, `contour`, `twinx` in matplotlib; non-symmetric `eig` and `lstsq` in
linalg.

```text
NotImplementedError: numpy.linalg.lstsq is not available in the light Python stand-in
```

A singular matrix raises `LinAlgError: Matrix is singular.` on the Python line that made the call.

````sheet python
# Linear algebra in a block

```python
import numpy as np

K = np.array([[4.0, -2.0], [-2.0, 3.0]])
u = np.linalg.solve(K, np.array([10.0, 0.0]))
print("u =", [round(float(v), 4) for v in u])
export(u1=float(u[0]), u2=float(u[1]))
```

total: u1 + u2
````

```text
u = [3.75, 2.5]
u1 = 3.75
u2 = 2.5
total = u1 + u2 = 3.75 + 2.5 = 6.25
```

## Plotly-style charts

`import plotly.graph_objects as go` gives a small copy of the Plotly interface. A figure shown with
`fig.show()` becomes an interactive chart in the page: hover to read values, drag to zoom, use the
toolbar to save a picture.

````sheet python
# Shear and moment with Plotly

```python
import numpy as np
import plotly.graph_objects as go
from plotly.subplots import make_subplots

L_m, w_k = 6.0, 12.0
x = np.linspace(0, L_m, 61)
V = w_k * (L_m / 2 - x)
M = w_k * x * (L_m - x) / 2

fig = make_subplots(rows=2, cols=1, shared_xaxes=True, subplot_titles=["Shear V (kN)", "Moment M (kN.m)"])
fig.add_trace(go.Scatter(x=x, y=V, name="V", line_color="#1f77b4"), row=1, col=1)
fig.add_trace(go.Scatter(x=x, y=M, name="M", fill="tozeroy"), row=2, col=1)
fig.add_vline(x=L_m / 2, line_dash="dot")
fig.update_layout(title="Simply supported beam", height=480, xaxis2_title="x (m)")
fig.show()
```
````

This is a pass-through, not a second copy of Plotly. Every keyword argument is written into the
figure as given, and Plotly.js in the page checks it. Supported pieces:

| Supported | Notes |
| --- | --- |
| `go.Figure(data=, layout=, frames=)` | `data` takes traces or dicts. |
| Traces `Scatter`, `Scattergl`, `Bar`, `Histogram`, `Box`, `Violin`, `Pie`, `Heatmap`, `Contour`, `Surface`, `Scatter3d`, `Mesh3d`, `Scatterpolar`, `Waterfall`, `Indicator`, `Table`, `Candlestick` | The class only sets the type. All other keywords are the trace's own. |
| `fig.add_trace`, `add_traces`, `add_scatter`, `add_bar` | `row` and `col` on a figure from `make_subplots`. |
| `fig.update_layout`, `update_xaxes`, `update_yaxes`, `update_traces` | Recursive merge. |
| `add_shape`, `add_annotation`, `add_vline`, `add_hline`, `add_vrect`, `add_hrect` | With `row` and `col`, the lines use that subplot's axes. |
| `make_subplots(rows, cols, shared_xaxes, shared_yaxes, subplot_titles, row_heights, column_widths, vertical_spacing, horizontal_spacing)` | |
| `updatemenus` and `sliders` with precomputed steps | Dropdowns, buttons, sliders; `frames` for animation. |

"Magic underscores" work when they are keyword arguments of a call: `line_color="red"` is
`line=dict(color="red")` and `xaxis_title="x"` is `xaxis=dict(title="x")`. Inside a dict they are
not expanded, and the shim refuses them by name rather than letting Plotly drop them silently.

Things that fail with a named error: trace types not in the table, `write_html`, `write_image`,
`fig.data`, `fig.show(renderer=...)`, `make_subplots(specs=...)`, `plotly.express`, `plotly.io`,
named templates (`template="plotly_white"`; the page applies a light or dark theme itself).

```text
NotImplementedError: go.Figure.write_html is not available in the light plotly shim
```

**Controls are Plotly's own, and they are independent.** A dropdown or slider step is a fixed
update that Plotly applies; it cannot read another control or run Python. For controls that
combine, precompute one step per combination or give each control its own traces. The "Beam
explorer" sample shows both.

### Plotly offline, printing and exports

Plotly.js is not bundled. It is fetched once from its CDN when the first chart is shown, and
cached. Without a connection the chart's place shows a note: "Interactive charts need a connection
to load Plotly...". The Python solve and the rest of the sheet still run. The charts follow the
page's light or dark theme.

Print, PDF, Word and Excel cannot run Plotly.js, so the chart is not carried: a paragraph
stands in its place ("Interactive chart: TITLE. It is drawn in the browser and is not carried by
print or export."). If the printout needs the picture, draw a static copy with matplotlib as well.

## 3D plots

A Plotly `Surface`, `Scatter3d`, `Mesh3d` or a 3D line figure is a 3D scene you can rotate and
zoom. The contour lines can be projected onto the floor.

````sheet python
# A 3D surface

```python
import plotly.graph_objects as go

xs = [i * 0.25 for i in range(25)]
ys = [j * 0.25 for j in range(17)]
z = [[((x - 3) ** 2 + (y - 2) ** 2) ** 0.5 for x in xs] for y in ys]

fig = go.Figure(go.Surface(x=xs, y=ys, z=z, colorscale="Viridis"))
fig.update_layout(height=420, scene=dict(zaxis=dict(title=dict(text="distance"))))
fig.show()
```
````

The samples under **3D plots** show a plate deflection surface, a column interaction surface, a
space frame drawn in 3D lines, a mode-shape explorer with a dropdown, and a 3D bar chart of
footing reactions. The walls of the 3D axes follow the light or dark theme. The buttons under a 3D scene are
Copy data and PNG.

## Plot gallery

The **Plot gallery** sample (group "Plots and diagrams") is a gallery of four plots on a grid in the style of a
plotting demo: implicit curves (f(x, y) = 0 drawn as a contour at level zero), functions,
parametric curves, noise, and polylines with markers, a legend and a second y axis. It is drawn
in a Python block with the Plotly API, so it can be zoomed and read by hovering.

## Limits

| Limit | Value |
| --- | --- |
| Time per block | 10 seconds. The worker is stopped and restarted. |
| Time per calculation | 30 seconds. |
| Print output | 100,000 characters |
| Figures | 20 per sheet, 3 MB of SVG |
| Chart size | 4 MB of figure data, 20 figures and drawings together |
| Exported names | 500 |
| Interpreter memory | 32 MB |

**The longer time limit.** When a block runs out of its ten seconds, the page asks whether to run the
sheet again with a longer limit: 60 seconds per block and 180 seconds per calculation. This is for
a slow device such as a phone, which can need several times what a laptop needs. The answer is
kept for the tab.

## The sandbox

A block runs in a Web Worker: no page access, no cookies, no stored data. The worker's network
functions are removed after loading, so a script cannot send data anywhere, and the interpreter's
host-access modules are refused. A runaway loop or allocation is stopped by the time and memory
limits.

This is a practical guard and not a security boundary against a determined attacker. Calculating a
sheet executes its blocks, which is why the page asks before the first run. The **Samples** browser
never runs a sample's Python; its preview shows the source, and opening the sample asks.

## Errors

A Python exception shows as one red line naming the sheet line and the exception type, and
appears in the error list on that line. The rest of the sheet still calculates, so a failed block
leaves the names it never exported undefined, and the lines that use them report their own errors.

```text
Python error on line 5: ZeroDivisionError ...
```

Typical causes: a name that is not in the stand-in (a `NotImplementedError` naming it), a Plotly
keyword written in a dict with an underscore, an exported value that is `nan` or `inf`, or a block
that ran past its time limit.

## Calcpad `#python` blocks

In the Calcpad language the block is `#python` ... `#end python` on lines of their own, and a
sheet accepts that pair too.
