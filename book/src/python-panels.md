# Interactive panels

<!-- description: Sliders, drop-downs and checkboxes in a Python block that redraw a Plotly chart, a table or text in place as you move them, without recalculating the sheet. -->

An interactive panel is a set of controls drawn above a chart, a table or a line of text, all made
by one Python block. Move a slider and one Python function runs again with the new value, and its
chart or table is redrawn in place. The rest of the sheet is not recalculated, so a panel stays
quick even when the sheet is long.

The samples in the **Panels** group show three of them: a beam deflection explorer, a pump
operating point with a play button, and a column buckling player.

## Prompt-friendly summary

A complete sheet with a slider that redraws a chart:

````sheet python
# Moment explorer

```python
import ui
import plotly.graph_objects as go
L = ui.slider("L", 6, 2, 12, step=0.5, unit="m", label="Span")
@ui.panel(L)
def moment(L):
    x = [L * i / 40 for i in range(41)]
    return go.Figure(go.Scatter(x=x, y=[12 * v * (L - v) / 2 for v in x]))
```
````

The five rules:

1. Write it in a ` ```python ` fence of a sheet, `import ui`, make the widgets with `ui.slider`,
   `ui.select`, `ui.checkbox` or `ui.player`, and decorate one function with `@ui.panel(widget, ...)`.
   The function takes one argument per widget, in the same order, and receives plain values.
2. Return a Plotly figure (do not call `fig.show()`), a string, `ui.table(rows, header=[...])`, or a
   list of these. Return the chart from the very first call too: Plotly is loaded only when the
   first result holds a chart.
3. The panel cannot read the sheet's quantities, and the sheet cannot read the panel's values. Put
   every number the panel needs in the block itself.
4. The Python is a small stand-in for numpy: `reshape`, `vstack`, `hstack` and `sort` are not there
   (`meshgrid` and `math.hypot` are). Build large lists with a loop or a comprehension: a list
   literal of more than 16,000 numbers in the code is refused.
5. Do the heavy work once in the block body and keep the panel function light: it runs on every
   move of a control.

## A panel, a sheet slider or an input form

There are three ways to put a slider in front of a reader. Choose by what has to change.

| You want | Use | What a move does |
| --- | --- | --- |
| A chart, table or text redrawn live while dragging | An interactive panel | Runs one Python function again. Nothing else is recalculated. |
| A sheet input with a slider, `L: ? 6 m [2 .. 12 step 0.5]` | [Templates with inputs](templates.md) | Recalculates the whole sheet with the new value, a moment after you stop. |
| A fill-in form of several inputs | Input mode, see [Templates with inputs](templates.md) | Recalculates the whole sheet; every quantity line follows. |

A sheet slider changes a quantity that the lines below use, with its unit, and the report shows the
working. A panel is for exploring: the chart follows your finger, but its numbers stay inside the
panel.

## The first run

A panel is part of a Python block, so the first calculation of the sheet in a tab asks "This
worksheet runs Python code" with **Run Python** and **Not now**. The block runs in a sandbox in the
page, see [Python blocks](python.md#the-sandbox). The first run takes several seconds while Python
starts and, for a chart, while Plotly loads from its CDN; after that a move of a control usually
redraws in a fraction of a second.

## A chart: the beam moment

````sheet python
# Beam moment explorer

```python
import ui
import plotly.graph_objects as go

L = ui.slider("L", 6, 2, 12, step=0.5, unit="m", label="Span")
w = ui.slider("w", 12, 1, 40, unit="kN/m", label="Load")

@ui.panel(L, w)
def moment(L, w):
    x = [L * i / 60 for i in range(61)]
    M = [w * v * (L - v) / 2 for v in x]
    fig = go.Figure(go.Scatter(x=x, y=M, mode="lines", fill="tozeroy", name="M"))
    fig.update_layout(height=320, xaxis_title="x (m)", yaxis_title="M (kN.m)")
    return [fig, "Maximum moment %.1f kN.m at midspan" % (w * L * L / 8)]
```
````

The page shows two sliders, Span and Load, each with its value and unit beside it, then the chart
and the line of text. At the start it reads `Maximum moment 54.0 kN.m at midspan`. Drag Span to
10 m and the curve and the text follow: `Maximum moment 150.0 kN.m at midspan`.

What happens, in order:

1. The block runs once, top to bottom, like any Python block.
2. At `@ui.panel(L, w)` the function `moment` is called with the starting values, 6 and 12, and its
   result is drawn under the controls.
3. Each move of a control calls `moment` again with the new values, in the same Python session, and
   only the panel is redrawn. The function's name, `moment`, is the panel's name.

## A 3D surface

A 3D trace works the same way. Here one slider sets the amplitude of a surface:

````sheet python
# Surface explorer

```python
import ui
import math
import plotly.graph_objects as go

a = ui.slider("a", 1.0, 0.2, 3.0, step=0.1, label="Amplitude")
xs = [-3 + 0.2 * i for i in range(31)]

@ui.panel(a)
def surface(a):
    Z = [[a * math.sin(x) * math.cos(y) for x in xs] for y in xs]
    fig = go.Figure(go.Surface(x=xs, y=xs, z=Z))
    fig.update_layout(height=420, scene=dict(zaxis=dict(range=[-3, 3])))
    return fig
```
````

The grid `xs` is made once, in the block body. The fixed `zaxis` range keeps the box the same size
while the surface grows and shrinks; without it, Plotly rescales the axis on every move. A rotated
view is kept when the surface is redrawn.

## A table, a drop-down and a checkbox

A panel does not need a chart. This one returns only a table:

````sheet python
# Timber joist spans

```python
import ui

d = ui.select("d", [("200 mm", 0.2), ("300 mm", 0.3), ("400 mm", 0.4)], default="300 mm", label="Depth")
b = ui.slider("b", 90, 45, 200, step=5, unit="mm", label="Width")
sw = ui.checkbox("sw", True, label="Include own weight")

@ui.panel(d, b, sw)
def spans(d, b, sw):
    Z = b / 1000 * d * d / 6
    g = 5.0 * b / 1000 * d if sw else 0.0
    rows = []
    for L in [3, 4, 5, 6]:
        M = (2.0 + g) * L * L / 8
        rows.append([L, "%.2f" % M, "%.1f" % (M / Z / 1000)])
    return ui.table(rows, header=["Span (m)", "M (kN.m)", "Stress (MPa)"])
```
````

The drop-down shows the labels (`200 mm`, ...) and the function receives the value paired with the
chosen label (`0.3`). The checkbox gives `True` or `False`. A table without a chart never loads
Plotly, so it also works offline once the page has been visited.

## A player

`ui.player` is a frame number with a **Play** and **Pause** button. Each frame is one call of the
function, and the next frame is asked for only after the last one is drawn, so a slow function plays
slower rather than piling up.

````sheet python
# Travelling wave

```python
import ui
import math
import plotly.graph_objects as go

t = ui.player("t", 0, 39, fps=8, label="Frame")
xs = [i * 0.1 for i in range(101)]

@ui.panel(t)
def wave(t):
    y = [math.sin(x - t * 0.3) for x in xs]
    fig = go.Figure(go.Scatter(x=xs, y=y, mode="lines"))
    fig.update_layout(height=260, yaxis_range=[-1.2, 1.2])
    return fig
```
````

A player never starts by itself, and it pauses when the browser tab is hidden. With `loop=False` it
stops at the last frame.

## The `ui` reference

| Call | Draws | The function receives |
| --- | --- | --- |
| `ui.slider(name, default, min, max, step=None, unit="", label=None)` | A slider with its value and unit beside it | A number between `min` and `max`, on a step. An `int` when `default`, `min`, `max` and `step` are all whole numbers, otherwise a `float`. |
| `ui.select(name, options, default=None, label=None)` | A drop-down | The value of the chosen option |
| `ui.checkbox(name, default=False, label=None)` | A checkbox | `True` or `False` |
| `ui.player(name, start, stop, fps=5, loop=True, label=None)` | A slider with Play and Pause | A whole frame number from `start` to `stop` |
| `@ui.panel(widget, ...)` | The controls, then the function's result | (a decorator) |
| `ui.table(rows, header=None)` | A table | (a return value) |
| `ui.text(s)` | A line of plain text | (a return value) |

Details that matter:

* **name** is the first argument of every widget. It must be unique within a panel. **label** is
  what the reader sees; it defaults to the name.
* **step**: without one, a slider of whole numbers moves by 1, and any other slider takes a round
  step that gives 100 to 200 steps across its range. A value is always kept between `min` and `max`
  and snapped to a step.
* **unit** is only shown beside the value. The function receives a plain number, so write the
  conversions yourself (`w * 1000` for kN/m to N/m).
* **options** of `ui.select` is a list of labels (`["Steel", "Timber"]`, the value is the label
  itself) or of `(label, value)` pairs. A `dict` works, but its order is not promised. `default` is
  a label or a value; without it the first option is chosen. A select needs two options or more.
* **fps** of a player is at most 30. A frame waits for the last one, so the real rate can be lower.
* **Return values**: a Plotly figure from `plotly.graph_objects`, a `str` or `ui.text(...)` (always
  shown as text, never as HTML), a number, `ui.table(...)`, or a list of these in the order they are
  drawn. `None` draws nothing. Anything else is a `TypeError` that names the type.
* Numbers in a `ui.table` are written with six significant digits. Format them yourself
  (`"%.2f" % M`) to fix the decimals.

## Several outputs, several panels

To drive two charts with the same slider, return both from one function: `return [fig1, fig2]`.

A block can hold several panels, each with its own `@ui.panel` and its own controls. A widget given
to two panels is drawn in each of them, and the two copies do not move together. Moving one control
reruns only its own panel's function.

## Errors

* An error during the first call is a normal Python error: one red line naming the sheet line.
* An error after a move (a division by zero at one end of a slider, say) is shown in the panel,
  with its type. The panel recovers at the next value that works.
* A function that runs for more than 10 seconds is stopped. The panel says so, and it stays out of
  date until the sheet is calculated again (edit any line, or press calculate).
* `print()` inside the panel function shows only on the first call. Return a string to show text
  that follows the controls.

## Printing and exports

Print, PDF, Word and Excel cannot run Python. They show the panel at the values the controls were
left at, without the controls, and a note under it, for example: "Interactive panel moment: Span =
10 m, Load = 12 kN/m. It is drawn in the browser; print and export show it at these values." Text
and tables are carried as they are. A chart is carried as a note naming it, as for any
[Plotly chart](python.md#plotly-offline-printing-and-exports).

The values the controls were left at are kept while the tab is open, so a recalculation (an edit
elsewhere in the sheet) starts the panel where you left it. They are not saved in the file: opening
the file again starts from the defaults.

## Limits

**A panel cannot read the sheet, and the sheet cannot read the panel.** Python runs before the
sheet's quantity lines and cannot see them, and a slider's value never reaches a quantity line.
A panel therefore holds its own data and inputs: write the section properties, loads and lengths
in the block. An `export()` from the block body works as in any block, but it is done once, before
any control moves; do not call `export()` in a panel function. When a value has to drive the
sheet's calculation, use a sheet input instead (`L: ? 6 m [2 .. 12 step 0.5]`).

**The numpy stand-in is small.** The block runs on MicroPython with a light stand-in for numpy,
not on full Python. The list of what works is in [Which numpy and
matplotlib](python.md#which-numpy-and-matplotlib). Things that surprise people:

* `numpy.reshape`, `vstack`, `hstack` and `sort` raise `NotImplementedError`. `numpy.meshgrid`,
  `hypot`, `arctan2`, `outer`, `interp`, `polyfit` and `polyval` work, as does `math.hypot`.
* A list literal of more than 16,000 numbers written out in the code is refused before it runs,
  with a hint to build the list in a loop or parse a text string. Shorter ones, even several
  thousand numbers, work.
* `np.linspace` and array arithmetic work, but plain lists and comprehensions are just as quick for
  the sizes a chart needs.

**It runs only in the browser.** The [MCP server](mcp.md) does not run Python blocks, so it cannot
evaluate a sheet with a panel. Word and Excel get the snapshot described above.

**Plotly comes from its CDN.** A chart needs a connection the first time (the controls, tables and
text work without one). Plotly is loaded only when the first result of a panel holds a chart, so a
panel that returns a chart only after some move shows a note instead of the chart. Return a chart
on the first call too.

**The usual Python limits apply**: 10 seconds per call, and an answer of at most 2 MB (a figure and
its data) per move. See [Python blocks](python.md#limits).

## Keeping it fast

The function runs on every move of a control, so keep in it only what depends on the controls. Work
that does not, such as a grid, a stiffness matrix or a solved model, belongs in the block body, which
runs once. The function can read the variables the block made before it.

````sheet python
# Precomputed surface

```python
import ui
import math
import plotly.graph_objects as go

xs = [-3 + 0.1 * i for i in range(61)]
base = [[math.sin(x) * math.cos(y) for x in xs] for y in xs]

k = ui.slider("k", 1.0, 0.1, 2.0, step=0.1, label="Scale")

@ui.panel(k)
def scaled(k):
    Z = [[k * v for v in row] for row in base]
    fig = go.Figure(go.Surface(x=xs, y=xs, z=Z))
    fig.update_layout(height=420, scene=dict(zaxis=dict(range=[-2, 2])))
    return fig
```
````

The sines and cosines of the 61 by 61 grid are worked out once; each move only multiplies.

* Keep charts to a few thousand points. The whole figure is sent on every move.
* Do not change globals in the function. A function that depends only on its arguments and on what
  the block computed before it gives the same picture for the same control values, whatever order
  the reader moved them in.
* Fix axis ranges (`yaxis_range`, `scene=dict(zaxis=dict(range=...))`) so the chart does not jump
  while dragging.
