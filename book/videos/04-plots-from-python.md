# Video 4: Plots from Python (Plotly style and matplotlib style)

* File to record: `04-plots-from-python.mp4`
* Length: about 90 seconds
* Book page: [Python blocks](../src/python.md)
* Screen: as in video 1. The machine needs a network connection for the first Plotly chart.

## Before recording

1. New sheet, example lines deleted.
2. Do a dry run first, so that the Python runtime and the Plotly script are cached and the chart
   appears quickly in the take. A fresh browser profile would show the Python question again, which
   is what scene 2 wants. Use a second, clean profile for the take and a warm one for the dry run.

## Scenes

| Time | On screen | Type or do | Narration |
| --- | --- | --- | --- |
| 0:00 to 0:10 | Empty sheet. Caption: "Python, inside the page". | None. | S1 |
| 0:10 to 0:30 | The sheet from block A is typed or pasted. The question "This worksheet runs Python code" appears. Click **Run Python**. | Paste block A (it is long: paste, do not type). Click Run Python. | S2 |
| 0:30 to 0:48 | The matplotlib figure appears with the printed line above it. Below it the two lines of the sheet agree: fifty-four kilonewton metres. Zoom on the two lines. | Wait for the result. | S3 |
| 0:48 to 1:05 | Replace the Python block with block B. An interactive chart with two panels appears. Hover over the curve: a tooltip. Drag to zoom. | Paste block B. Hover and drag. | S4 |
| 1:05 to 1:20 | Add the third sheet (block C): a 3D surface. Drag to rotate it. | Paste block C. Drag the surface. | S5 |
| 1:20 to 1:30 | End card: "Python runs in the page. Free and open source." | None. | S6 |

## What to paste

Block A, a sheet with a matplotlib-style figure whose peak comes back into the sheet:

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

Block B, the same beam with the Plotly interface:

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

Block C, a surface:

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

What block A prints and shows:

```text
Maximum moment 54.0 kN.m at 3.0 m
[figure]
M_peak = 54
Mmax = M_peak · 1 kN·m = 54 · 1 kN·m = 54 kN·m
Mcheck = w·L²/8 = 12 kN/m · (6 m)² / 8 = 54 kN·m
```

## Narration

S1: Sometimes you want a plot that a formula cannot give. Write a Python block, right inside the sheet.

S2: The first time, the page asks permission to run Python. Say yes. The runtime is small, and it loads only when a sheet needs it.

S3: This block draws a moment diagram in the style of mat plot lib. It also hands the peak value back to the sheet. The sheet adds the unit, and checks it against the formula. Both lines say fifty-four kilonewton metres.

S4: For interactive charts, use the Plotly style. Hover over the curve to read values. Drag to zoom in. Each panel has its own axis, and the same code works in the light and the dark theme.

S5: Surfaces work too, in three dimensions. Drag to turn it around, and scroll to zoom.

S6: The Python runs in your page, in a sandbox, with no server. CivilKit Calcs is free and open source.

## Full text for the voice (one block)

Sometimes you want a plot that a formula cannot give. Write a Python block, right inside the sheet. The first time, the page asks permission to run Python. Say yes. The runtime is small, and it loads only when a sheet needs it. This block draws a moment diagram in the style of mat plot lib. It also hands the peak value back to the sheet. The sheet adds the unit, and checks it against the formula. Both lines say fifty-four kilonewton metres. For interactive charts, use the Plotly style. Hover over the curve to read values. Drag to zoom in. Each panel has its own axis, and the same code works in the light and the dark theme. Surfaces work too, in three dimensions. Drag to turn it around, and scroll to zoom. The Python runs in your page, in a sandbox, with no server. CivilKit Calcs is free and open source.

## Light marketing beats

* Scene 2: the runtime is small and loads only when needed.
* Scene 6: runs in the page, free and open source.

## Check after rendering

* The Python question is visible for at least two seconds before it is answered.
* Plotly's script loads from the network the first time. If the chart shows the offline note, cut
  and retake on a connected machine.
