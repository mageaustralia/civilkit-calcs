"""A thin plotly.graph_objects for a worksheet's Python block.

This is a pass-through, not a re-implementation of plotly. Every keyword argument is accepted and written into the figure
JSON as given (numbers, strings, bools, None, lists, tuples, dicts and arrays); Plotly.js in the report page validates it.
Nothing is checked here beyond what would otherwise be dropped without a word:

  * `line_color="red"` style names (plotly's "magic underscores") are expanded to `line=dict(color="red")` when they are
    keyword arguments of a call. In a dict the nested form has to be written, and a key such as "line_color" inside a dict
    raises ValueError naming the key, because Plotly.js would ignore it.
  * anything this module does not provide (a trace type, a Figure method or attribute, a make_subplots option, a
    fig.show() option) raises NotImplementedError naming it.

`fig.show()` sends the figure to the report; the page draws it with Plotly.js. NaN and infinity become null, and numbers
are written with 9 significant digits.
"""
import json
import _cp_host

__all__ = ["Figure", "Layout", "Frame", "Scatter", "Scattergl", "Bar", "Histogram", "Box", "Violin", "Pie", "Heatmap",
           "Contour", "Surface", "Scatter3d", "Mesh3d", "Scatterpolar", "Waterfall", "Indicator", "Table", "Candlestick"]

# Names that really contain an underscore in Plotly's schema; every other underscore is a nesting separator.
_GENUINE = ("paper_bgcolor", "plot_bgcolor", "error_x", "error_y", "error_z")
# Keys whose value is the author's own data; their inner keys are not checked.
_FREE = ("meta",)
_INF = float("inf")

_TRACES = {
    "Scatter": "scatter", "Scattergl": "scattergl", "Bar": "bar", "Histogram": "histogram", "Box": "box", "Violin": "violin",
    "Pie": "pie", "Heatmap": "heatmap", "Contour": "contour", "Surface": "surface", "Scatter3d": "scatter3d",
    "Mesh3d": "mesh3d", "Scatterpolar": "scatterpolar", "Waterfall": "waterfall", "Indicator": "indicator", "Table": "table",
    "Candlestick": "candlestick",
}


def _no(module, name):
    """The exception for a name this module does not provide."""
    if name.startswith("_"):
        return AttributeError("module '" + module + "' has no attribute '" + name + "'")
    return NotImplementedError(module + "." + name + " is not available in the light plotly shim (traces: " + ", ".join(sorted(_TRACES)) + "; also Figure, Layout, Frame)")


def __getattr__(name):
    raise _no("plotly.graph_objects", name)


# ---- turning what the author passed into plain JSON-able Python ---------------------------------------------------------

class _Obj:
    """A trace, Layout or Frame: a dict with a name. Attributes cannot be read or assigned (plotly's property tree is not here)."""

    def __init__(self, arg=None, **kw):
        who = "go." + type(self).__name__
        d = {}
        if arg is not None:
            if isinstance(arg, _Obj):
                arg = arg._d
            if not isinstance(arg, dict):
                raise TypeError(who + ": the first argument must be a dict or another " + type(self).__name__ + ", not " + type(arg).__name__)
            _merge(d, _plain(arg, who))
        _merge(d, _expand(kw, who))
        t = getattr(type(self), "_type", None)
        if t:
            if "type" in kw:
                raise TypeError(who + ": 'type' is set by the class")
            d["type"] = t
        object.__setattr__(self, "_d", d)

    def update(self, patch=None, **kw):
        who = "go." + type(self).__name__ + ".update"
        if patch is not None:
            _merge(self._d, _plain(patch, who))
        _merge(self._d, _expand(kw, who))
        return self

    def to_plotly_json(self):
        return _plain(self._d, "go." + type(self).__name__)

    to_dict = to_plotly_json

    def __setattr__(self, name, value):
        raise AttributeError("go." + type(self).__name__ + ": cannot set '" + name + "' on a plotly object in the light shim; pass it as a keyword argument or call update(" + name + "=...)")

    def __getattr__(self, name):
        if name.startswith("_"):
            raise AttributeError(name)
        raise NotImplementedError("go." + type(self).__name__ + "." + name + " is not available in the light plotly shim; keep the values in your own variables")


class Layout(_Obj):
    pass


class Frame(_Obj):
    pass


def _make_trace(cls_name, type_name):
    return type(cls_name, (_Obj,), {"_type": type_name})


for _n in _TRACES:
    globals()[_n] = _make_trace(_n, _TRACES[_n])
del _n


def _plain(v, path, free=False):
    """A copy of v made of None, bool, int, float, str, list and dict only; NaN and infinity become None."""
    if v is None or v is True or v is False or isinstance(v, str):
        return v
    if isinstance(v, int):
        return v
    if isinstance(v, float):
        return None if (v != v or v == _INF or v == -_INF) else v
    if isinstance(v, (list, tuple)):
        return [_plain(e, path, free) for e in v]
    if isinstance(v, dict):
        out = {}
        for k in v:
            if not isinstance(k, str):
                raise TypeError(path + ": dictionary keys must be strings, not " + repr(k))
            if not free and "_" in k:
                _check_key(k, path)
            out[k] = _plain(v[k], path + "." + k, free or k in _FREE)
        return out
    if isinstance(v, _Obj):
        return _plain(v._d, path, free)
    if hasattr(v, "tolist"):
        return _plain(v.tolist(), path, free)
    raise TypeError(path + ": a " + type(v).__name__ + " cannot be sent to Plotly (use numbers, strings, bools, None, lists, tuples, dicts and numeric arrays)")


def _check_key(k, path):
    for seg in k.split("."):
        if "_" in seg and seg not in _GENUINE:
            parts = seg.split("_")
            raise ValueError(path + ": the key '" + k + "' is written with underscores inside a dict, where Plotly.js would ignore it. Underscore names such as "
                             + "line_color are expanded only when they are keyword arguments of a call; in a dict write the nested form, e.g. "
                             + "{'" + parts[0] + "': {'" + "_".join(parts[1:]) + "': ...}}")


def _merge(dst, src):
    """Recursive update: dicts merge, anything else replaces."""
    for k in src:
        s = src[k]
        d = dst.get(k)
        if isinstance(s, dict) and isinstance(d, dict):
            _merge(d, s)
        else:
            dst[k] = s


def _expand(kw, who):
    """The keyword arguments of a call as a nested dict; `line_color=x` becomes {'line': {'color': x}}."""
    out = {}
    for k in kw:
        val = _plain(kw[k], who + " " + k, k in _FREE)
        if "_" in k and k not in _GENUINE:
            parts = k.split("_")
            for p in parts:
                if p == "":
                    raise ValueError(who + ": the keyword '" + k + "' is not a plotly property name")
            for p in reversed(parts):
                val = {p: val}
            _merge(out, val)
        else:
            _merge(out, {k: val})
    return out


def _dump(v, o):
    if v is None:
        o.append("null")
    elif v is True:
        o.append("true")
    elif v is False:
        o.append("false")
    elif isinstance(v, str):
        o.append(json.dumps(v))
    elif isinstance(v, int):
        o.append(str(v))
    elif isinstance(v, float):
        o.append("%.9g" % v)
    elif isinstance(v, list):
        o.append("[")
        first = True
        for e in v:
            if first:
                first = False
            else:
                o.append(",")
            _dump(e, o)
        o.append("]")
    else:
        o.append("{")
        first = True
        for k in v:
            if first:
                first = False
            else:
                o.append(",")
            o.append(json.dumps(k))
            o.append(":")
            _dump(v[k], o)
        o.append("}")


def _ax(letter, k):
    return letter if k == 1 else letter + str(k)


def _axkey(letter, k):
    return letter + "axis" + ("" if k == 1 else str(k))


def _traces_in(data, who):
    if data is None:
        return []
    if isinstance(data, (_Obj, dict)):
        data = [data]
    out = []
    for t in data:
        if isinstance(t, _Obj):
            t = t._d
        if not isinstance(t, dict):
            raise TypeError(who + ": a trace must be a go.Scatter, go.Bar... or a dict, not " + type(t).__name__)
        out.append(_plain(t, who))
    return out


# ---- the figure ------------------------------------------------------------------------------------------------------------

_FIG_METHODS = ("add_trace, add_traces, add_scatter / add_bar / add_heatmap / ..., update_layout, update_xaxes, update_yaxes, update_traces, "
                "add_shape, add_vline, add_hline, add_vrect, add_hrect, add_annotation, show, to_dict, to_json")


class Figure:
    def __init__(self, data=None, layout=None, frames=None, **kw):
        if kw:
            raise TypeError("go.Figure() got unsupported argument(s) " + ", ".join(sorted(kw)) + " (it takes data, layout and frames)")
        object.__setattr__(self, "_data", [])
        object.__setattr__(self, "_layout", {})
        object.__setattr__(self, "_frames", [])
        object.__setattr__(self, "_grid", None)
        if isinstance(data, dict) and ("data" in data or "layout" in data) and layout is None and frames is None:
            data, layout, frames = data.get("data"), data.get("layout"), data.get("frames")
        self._data.extend(_traces_in(data, "go.Figure(data=)"))
        if layout is not None:
            self.update_layout(layout)
        if frames is not None:
            for f in frames:
                self._frames.append(_plain(f, "go.Figure(frames=)"))

    def _set_grid(self, rows, cols, layout):
        object.__setattr__(self, "_grid", (rows, cols))
        _merge(self._layout, layout)

    def __setattr__(self, name, value):
        raise AttributeError("go.Figure: cannot set '" + name + "' in the light plotly shim; use update_layout(...), update_traces(...) or add_trace(...)")

    def __getattr__(self, name):
        if name.startswith("_"):
            raise AttributeError(name)
        if name.startswith("add_") and name[4:].capitalize() in _TRACES:
            cls = globals()[name[4:].capitalize()]

            def add(row=None, col=None, **kw):
                return self.add_trace(cls(**kw), row=row, col=col)
            return add
        if name in ("data", "layout", "frames"):
            raise NotImplementedError("go.Figure." + name + " is not available in the light plotly shim: build the figure with add_trace / update_layout and keep the values in your own variables")
        raise NotImplementedError("go.Figure." + name + " is not available in the light plotly shim (provided: " + _FIG_METHODS + ")")

    def __getitem__(self, key):
        raise NotImplementedError("go.Figure[...] is not available in the light plotly shim")

    # -- which subplot --
    def _cells(self, row, col, who, every=False):
        """Subplot numbers for row and col. None (both) is one place outside a grid; with every=True a missing or 'all' means all."""
        g = self._grid
        if g is None:
            if row not in (None, "all") or col not in (None, "all"):
                raise ValueError(who + ": row and col need a figure made with plotly.subplots.make_subplots")
            return [None]
        if not every and (row is None) != (col is None):
            raise ValueError(who + ": give both row and col, or neither")
        rs = range(1, g[0] + 1) if row in (None, "all") else [row]
        cs = range(1, g[1] + 1) if col in (None, "all") else [col]
        out = []
        for r in rs:
            for c in cs:
                if not (isinstance(r, int) and isinstance(c, int) and 1 <= r <= g[0] and 1 <= c <= g[1]):
                    raise ValueError(who + ": row=" + repr(r) + ", col=" + repr(c) + " is outside the " + str(g[0]) + " by " + str(g[1]) + " grid")
                out.append((r - 1) * g[1] + c)
        return out

    # -- traces --
    def add_trace(self, trace, row=None, col=None):
        who = "Figure.add_trace"
        t = _traces_in(trace, who)
        if len(t) != 1:
            raise TypeError(who + " takes one trace (use add_traces for several)")
        t = t[0]
        if row is not None or col is not None:
            if row is None or col is None:
                raise ValueError(who + ": give both row and col, or neither")
            k = self._cells(row, col, who)[0]
            t["xaxis"] = _ax("x", k)
            t["yaxis"] = _ax("y", k)
        self._data.append(t)
        return self

    def add_traces(self, data, rows=None, cols=None):
        ts = data if isinstance(data, (list, tuple)) else [data]
        for i, t in enumerate(ts):
            self.add_trace(t, row=None if rows is None else rows[i], col=None if cols is None else cols[i])
        return self

    def update_traces(self, patch=None, selector=None, row=None, col=None, overwrite=False, **kw):
        who = "Figure.update_traces"
        if overwrite:
            raise NotImplementedError(who + "(overwrite=True) is not available in the light plotly shim")
        if selector is not None and not isinstance(selector, dict):
            raise NotImplementedError(who + ": selector must be a dict such as dict(name='M') in the light plotly shim")
        sel = _plain(selector, who) if selector else {}
        upd = {}
        if patch is not None:
            _merge(upd, _plain(patch, who))
        _merge(upd, _expand(kw, who))
        cell = None
        if row is not None or col is not None:
            k = self._cells(row, col, who)[0]
            cell = (_ax("x", k), _ax("y", k))
        for t in self._data:
            if any(t.get(s) != sel[s] for s in sel):
                continue
            if cell and (t.get("xaxis", "x"), t.get("yaxis", "y")) != cell:
                continue
            _merge(t, _plain(upd, who))
        return self

    # -- layout --
    def update_layout(self, patch=None, **kw):
        who = "Figure.update_layout"
        upd = {}
        if patch is not None:
            _merge(upd, _plain(patch, who))
        _merge(upd, _expand(kw, who))
        if isinstance(upd.get("template"), str):
            raise ValueError(who + ": template='" + upd["template"] + "': named templates are not available; the report page applies a light or dark theme itself (pass nothing, or a dict)")
        _merge(self._layout, upd)
        return self

    def _update_axes(self, letter, patch, kw, selector, row, col):
        who = "Figure.update_" + letter + "axes"
        if selector is not None:
            raise NotImplementedError(who + "(selector=...) is not available in the light plotly shim")
        upd = {}
        if patch is not None:
            _merge(upd, _plain(patch, who))
        _merge(upd, _expand(kw, who))
        for k in self._cells(row, col, who, True):
            key = _axkey(letter, k or 1)
            cur = self._layout.get(key)
            if cur is None:
                cur = self._layout[key] = {}
            _merge(cur, _plain(upd, who))
        return self

    def update_xaxes(self, patch=None, selector=None, row=None, col=None, **kw):
        return self._update_axes("x", patch, kw, selector, row, col)

    def update_yaxes(self, patch=None, selector=None, row=None, col=None, **kw):
        return self._update_axes("y", patch, kw, selector, row, col)

    # -- shapes and annotations --
    def _put(self, listname, item, who):
        self._layout.setdefault(listname, []).append(item)

    def add_shape(self, patch=None, row=None, col=None, **kw):
        who = "Figure.add_shape"
        s = {}
        if patch is not None:
            _merge(s, _plain(patch, who))
        _merge(s, _expand(kw, who))
        for k in ([None] if row is None and col is None else self._cells(row, col, who)):
            one = _plain(s, who)
            if k is not None:
                one.setdefault("xref", _ax("x", k))
                one.setdefault("yref", _ax("y", k))
            self._put("shapes", one, who)
        return self

    def add_annotation(self, patch=None, row=None, col=None, **kw):
        who = "Figure.add_annotation"
        s = {}
        if patch is not None:
            _merge(s, _plain(patch, who))
        _merge(s, _expand(kw, who))
        for k in ([None] if row is None and col is None else self._cells(row, col, who)):
            one = _plain(s, who)
            if k is not None:
                one.setdefault("xref", _ax("x", k))
                one.setdefault("yref", _ax("y", k))
            self._put("annotations", one, who)
        return self

    def _ruler(self, who, kind, a, b, row, col, kw, exclude):
        if exclude is not None:
            raise NotImplementedError(who + "(exclude_empty_subplots=...) is not available in the light plotly shim")
        for k in kw:
            if k.startswith("annotation"):
                raise NotImplementedError(who + "(" + k + "=...) is not available in the light plotly shim; add the label with add_annotation")
        base = _expand(kw, who)
        for k in self._cells(row, col, who, True):
            X = _ax("x", k or 1)
            Y = _ax("y", k or 1)
            if kind == "v":
                s = {"type": "line", "x0": a, "x1": a, "y0": 0, "y1": 1, "xref": X, "yref": Y + " domain"}
            elif kind == "h":
                s = {"type": "line", "x0": 0, "x1": 1, "y0": a, "y1": a, "xref": X + " domain", "yref": Y}
            elif kind == "vr":
                s = {"type": "rect", "x0": a, "x1": b, "y0": 0, "y1": 1, "xref": X, "yref": Y + " domain"}
            else:
                s = {"type": "rect", "x0": 0, "x1": 1, "y0": a, "y1": b, "xref": X + " domain", "yref": Y}
            _merge(s, _plain(base, who))
            self._put("shapes", _plain(s, who), who)
        return self

    def add_vline(self, x, row="all", col="all", exclude_empty_subplots=None, **kw):
        return self._ruler("Figure.add_vline", "v", x, None, row, col, kw, exclude_empty_subplots)

    def add_hline(self, y, row="all", col="all", exclude_empty_subplots=None, **kw):
        return self._ruler("Figure.add_hline", "h", y, None, row, col, kw, exclude_empty_subplots)

    def add_vrect(self, x0, x1, row="all", col="all", exclude_empty_subplots=None, **kw):
        return self._ruler("Figure.add_vrect", "vr", x0, x1, row, col, kw, exclude_empty_subplots)

    def add_hrect(self, y0, y1, row="all", col="all", exclude_empty_subplots=None, **kw):
        return self._ruler("Figure.add_hrect", "hr", y0, y1, row, col, kw, exclude_empty_subplots)

    # -- output --
    def _spec(self, config=None):
        d = {"data": self._data, "layout": self._layout}
        if self._frames:
            d["frames"] = self._frames
        if config is not None:
            d["config"] = _plain(config, "Figure.show(config=)")
        return d

    def to_plotly_json(self):
        return _plain(self._spec(), "go.Figure")

    to_dict = to_plotly_json

    def to_json(self):
        o = []
        _dump(self._spec(), o)
        return "".join(o)

    def show(self, config=None, **kw):
        if kw:
            raise NotImplementedError("Figure.show(" + ", ".join(sorted(kw)) + "=...) is not available in the light plotly shim (only config=); the figure is always shown in the report")
        o = []
        _dump(self._spec(config), o)
        _cp_host.plotly("".join(o))
