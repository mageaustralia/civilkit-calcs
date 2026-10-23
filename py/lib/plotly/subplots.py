"""plotly.subplots.make_subplots: a grid of cartesian subplots (rows, cols, shared axes, titles, sizes)."""
import plotly.graph_objects as _go

_SUPPORTED = "rows, cols, shared_xaxes, shared_yaxes, subplot_titles, row_heights, column_widths, vertical_spacing, horizontal_spacing"


def _ax(letter, k):
    return letter if k == 1 else letter + str(k)


def _shared(v, what, allowed):
    if v in (False, None):
        return None
    if v is True:
        return allowed[0]
    if v in allowed:
        return v
    raise ValueError("plotly.subplots.make_subplots: " + what + "=" + repr(v) + " is not supported (use True, False or " + repr(allowed[-1]) + ")")


def make_subplots(rows=1, cols=1, shared_xaxes=False, shared_yaxes=False, subplot_titles=None, row_heights=None,
                  column_widths=None, vertical_spacing=None, horizontal_spacing=None, **kw):
    if kw:
        raise NotImplementedError("plotly.subplots.make_subplots(" + ", ".join(sorted(kw)) + "=...) is not available in the light plotly shim (supported: " + _SUPPORTED + ")")
    if not (isinstance(rows, int) and isinstance(cols, int) and rows >= 1 and cols >= 1):
        raise ValueError("plotly.subplots.make_subplots: rows and cols must be positive integers")
    sx = _shared(shared_xaxes, "shared_xaxes", ("columns", "all"))
    sy = _shared(shared_yaxes, "shared_yaxes", ("rows", "all"))
    vs = 0.3 / rows if vertical_spacing is None else vertical_spacing
    hs = 0.2 / cols if horizontal_spacing is None else horizontal_spacing
    rh = list(row_heights) if row_heights is not None else [1.0] * rows
    cw = list(column_widths) if column_widths is not None else [1.0] * cols
    if len(rh) != rows or len(cw) != cols:
        raise ValueError("plotly.subplots.make_subplots: row_heights needs one value per row and column_widths one per column")
    th = sum(rh)
    tw = sum(cw)
    hts = [h / th * (1 - vs * (rows - 1)) for h in rh]
    wds = [w / tw * (1 - hs * (cols - 1)) for w in cw]
    titles = list(subplot_titles) if subplot_titles is not None else []
    if len(titles) > rows * cols:
        raise ValueError("plotly.subplots.make_subplots: more subplot_titles than subplots")
    lay = {}
    notes = []
    for r in range(1, rows + 1):
        top = 1 - sum(hts[:r - 1]) - vs * (r - 1)
        bot = top - hts[r - 1]
        for c in range(1, cols + 1):
            k = (r - 1) * cols + c
            left = sum(wds[:c - 1]) + hs * (c - 1)
            right = left + wds[c - 1]
            xa = {"domain": [round(left, 6) + 0.0, round(right, 6) + 0.0], "anchor": _ax("y", k)}
            ya = {"domain": [round(bot, 6) + 0.0, round(top, 6) + 0.0], "anchor": _ax("x", k)}
            if sx and r < rows:
                xa["showticklabels"] = False
                xa["matches"] = _ax("x", (rows - 1) * cols + c) if sx == "columns" else "x"
            if sy and c > 1:
                ya["showticklabels"] = False
                ya["matches"] = _ax("y", (r - 1) * cols + 1) if sy == "rows" else "y"
            lay["xaxis" + ("" if k == 1 else str(k))] = xa
            lay["yaxis" + ("" if k == 1 else str(k))] = ya
            if k <= len(titles) and titles[k - 1]:
                notes.append({"text": str(titles[k - 1]), "showarrow": False, "x": round((left + right) / 2, 6), "xanchor": "center",
                              "xref": "paper", "y": round(top, 6), "yanchor": "bottom", "yref": "paper", "font": {"size": 14}})
    if notes:
        lay["annotations"] = notes
    fig = _go.Figure()
    fig._set_grid(rows, cols, lay)
    return fig
