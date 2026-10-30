"""Interactive panels for a worksheet's Python block (docs/interactive-panels-design.md).

    import ui
    L = ui.slider("L", 6, 2, 12, unit="m")
    w = ui.slider("w", 12, 1, 40, unit="kN/m")

    @ui.panel(L, w)
    def beam(L, w):
        ...
        return fig            # a plotly figure, text, ui.table(...), or a list of them

The block runs once, as always. For every `@ui.panel` the page draws the widgets and the first result. When a widget changes, the
page sends the new values back and ONLY the panel's function runs again, in the interpreter that is still alive; the sheet is not
recalculated. The function should depend on its arguments and on values the block already computed, not on state it changes.

This module is the Python side. It speaks to the page through `_cp_host` (panel, panelOut, seed). Everything it receives from the
page is checked here again: a value is converted by its widget, clamped to the widget's range, and refused when it does not fit.
"""
import json
import _cp_host

__all__ = ["slider", "select", "checkbox", "player", "panel", "table", "text"]

_panels = {}
_seed = None


def _seed_for(pid):
    global _seed
    if _seed is None:
        try:
            _seed = json.loads(_cp_host.seed())
        except Exception:
            _seed = {}
    s = _seed.get(pid)
    return s if isinstance(s, dict) else {}


def _finite(v, what):
    if isinstance(v, bool) or not isinstance(v, (int, float)):
        raise TypeError("ui: " + what + " must be a number, not " + type(v).__name__)
    if v != v or v in (float("inf"), float("-inf")):
        raise ValueError("ui: " + what + " is " + str(v))
    return v


def _nice_step(span):
    """A step of 1, 2 or 5 times a power of ten that gives 100 to 200 steps over the range."""
    s = span / 100.0
    p = 1.0
    while s >= 10:
        s /= 10
        p *= 10
    while s < 1:
        s *= 10
        p /= 10
    for m in (1, 2, 5, 10):
        if s <= m:
            return m * p
    return 10 * p


class _Widget:
    kind = ""

    def __init__(self, name, label):
        if not isinstance(name, str) or not name:
            raise TypeError("ui: the first argument of a widget is its name, a string")
        self.name = name
        self.label = label or name

    def spec(self):
        raise NotImplementedError

    def coerce(self, v):
        """The page's raw value, made into a legal one (or ValueError)."""
        raise NotImplementedError

    def value_of(self, stored):
        """What the function receives for the stored (wire) value."""
        return stored


class _Slider(_Widget):
    kind = "slider"

    def __init__(self, name, default, lo, hi, step, unit, label):
        _Widget.__init__(self, name, label)
        for k, v in (("default", default), ("min", lo), ("max", hi)):
            _finite(v, name + " " + k)
        if not lo < hi:
            raise ValueError("ui.slider(" + repr(name) + "): min must be below max")
        whole = isinstance(default, int) and isinstance(lo, int) and isinstance(hi, int) and (step is None or isinstance(step, int))
        if step is None:
            step = 1 if whole else _nice_step(hi - lo)
        _finite(step, name + " step")
        if step <= 0:
            raise ValueError("ui.slider(" + repr(name) + "): step must be positive")
        self.lo, self.hi, self.step, self.unit, self.whole = lo, hi, step, unit or "", whole
        self.default = self.coerce(default)

    def coerce(self, v):
        v = _finite(v if not isinstance(v, bool) else None, self.name)
        v = min(max(v, self.lo), self.hi)
        v = self.lo + round((v - self.lo) / self.step) * self.step
        v = min(max(v, self.lo), self.hi)
        if self.whole:
            return int(round(v))
        return float("%.10g" % v)

    def spec(self):
        return {"kind": "slider", "name": self.name, "label": self.label, "min": self.lo, "max": self.hi, "step": self.step,
                "unit": self.unit}


class _Player(_Slider):
    kind = "player"

    def __init__(self, name, start, stop, fps, loop, label):
        _Slider.__init__(self, name, start, start, stop, 1, "", label)
        _finite(fps, name + " fps")
        if not 0 < fps <= 30:
            raise ValueError("ui.player(" + repr(name) + "): fps must be above 0 and at most 30")
        self.fps, self.loop = fps, bool(loop)

    def spec(self):
        d = _Slider.spec(self)
        d["kind"] = "player"
        d["fps"] = self.fps
        d["loop"] = self.loop
        return d


class _Select(_Widget):
    kind = "select"

    def __init__(self, name, options, default, label):
        _Widget.__init__(self, name, label)
        if isinstance(options, dict):
            pairs = [(k, options[k]) for k in options]
        else:
            pairs = []
            for o in options:
                pairs.append((str(o[0]), o[1]) if isinstance(o, (list, tuple)) else (str(o), o))
        if len(pairs) < 2:
            raise ValueError("ui.select(" + repr(name) + "): needs two options or more")
        self.labels = [p[0] for p in pairs]
        self.values = [p[1] for p in pairs]
        self.default = 0
        if default is not None:
            if default in self.labels:
                self.default = self.labels.index(default)
            elif default in self.values:
                self.default = self.values.index(default)
            else:
                raise ValueError("ui.select(" + repr(name) + "): default " + repr(default) + " is not one of the options")

    def coerce(self, v):
        if isinstance(v, bool) or not isinstance(v, int) or not 0 <= v < len(self.labels):
            raise ValueError("ui.select(" + repr(self.name) + "): no such option")
        return v

    def value_of(self, stored):
        return self.values[stored]

    def spec(self):
        return {"kind": "select", "name": self.name, "label": self.label, "options": self.labels}


class _Checkbox(_Widget):
    kind = "checkbox"

    def __init__(self, name, default, label):
        _Widget.__init__(self, name, label)
        self.default = bool(default)

    def coerce(self, v):
        if not isinstance(v, bool):
            raise ValueError("ui.checkbox(" + repr(self.name) + "): expects true or false")
        return v

    def spec(self):
        return {"kind": "checkbox", "name": self.name, "label": self.label}


def slider(name, default, lo, hi, step=None, unit="", label=None):
    """A slider. The function receives a number between lo and hi (an int when all of default, lo, hi and step are ints)."""
    return _Slider(name, default, lo, hi, step, unit, label)


def select(name, options, default=None, label=None):
    """A drop-down. options: a list of labels, or of (label, value) pairs; a dict works but its order is not promised here."""
    return _Select(name, options, default, label)


def checkbox(name, default=False, label=None):
    """A checkbox; the function receives True or False."""
    return _Checkbox(name, default, label)


def player(name, start, stop, fps=5, loop=True, label=None):
    """A frame number from start to stop (ints) with play and pause. The next frame is asked for only after the last one is drawn."""
    return _Player(name, start, stop, fps, loop, label)


class table:
    """A small table to return from a panel: ui.table([[1, 2], [3, 4]], header=["a", "b"])."""

    def __init__(self, rows, header=None):
        self.rows = [[_cell(c) for c in r] for r in rows]
        self.header = [str(h) for h in header] if header else None


def _cell(c):
    if isinstance(c, float):
        return "%.6g" % c
    return str(c)


class text:
    """Plain text (never HTML) to return from a panel."""

    def __init__(self, s):
        self.s = str(s)


def _items(ret, who):
    """The JSON texts of what a panel function returned."""
    if ret is None:
        return []
    if isinstance(ret, (list, tuple)):
        out = []
        for r in ret:
            out.extend(_items(r, who))
        return out
    if hasattr(ret, "to_json") and hasattr(ret, "add_trace"):
        return ['{"t":"plotly","fig":' + ret.to_json() + "}"]
    if isinstance(ret, table):
        return [json.dumps({"t": "table", "head": ret.header, "rows": ret.rows})]
    if isinstance(ret, (str, text)):
        return [json.dumps({"t": "text", "s": ret if isinstance(ret, str) else ret.s})]
    if isinstance(ret, (int, float)) and not isinstance(ret, bool):
        return [json.dumps({"t": "text", "s": _cell(ret)})]
    raise TypeError(who + ": the function returned a " + type(ret).__name__ + "; return a plotly figure, text, ui.table(...) or a list of them")


class _Panel:
    def __init__(self, pid, widgets, fn):
        self.id, self.widgets, self.fn = pid, widgets, fn

    def stored(self, values):
        """Wire values, checked widget by widget; a missing name keeps the default."""
        out = {}
        for w in self.widgets:
            if values is not None and w.name in values:
                out[w.name] = w.coerce(values[w.name])
            else:
                out[w.name] = w.default
        return out

    def run(self, stored):
        args = [w.value_of(stored[w.name]) for w in self.widgets]
        return _items(self.fn(*args), "ui.panel " + self.id)

    def result(self, stored):
        return '{"panel":' + json.dumps(self.id) + ',"values":' + json.dumps(stored) + ',"out":[' + ",".join(self.run(stored)) + "]}"


def panel(*widgets):
    """Decorator: the function runs with the widgets' values now and again whenever one changes."""
    if not widgets:
        raise TypeError("ui.panel needs at least one widget: @ui.panel(ui.slider(...))")
    names = []
    for w in widgets:
        if not isinstance(w, _Widget):
            raise TypeError("ui.panel takes widgets (ui.slider, ui.select, ui.checkbox, ui.player), not a " + type(w).__name__)
        if w.name in names:
            raise ValueError("ui.panel: two widgets are named " + repr(w.name))
        names.append(w.name)

    def deco(fn):
        base = getattr(fn, "__name__", "panel")
        pid = base
        n = 1
        while pid in _panels:
            n += 1
            pid = base + "_" + str(n)
        p = _Panel(pid, widgets, fn)
        _panels[pid] = p
        seed = _seed_for(pid)
        want = {}
        for w in widgets:
            if w.name in seed:
                want[w.name] = seed[w.name]
        try:
            stored = p.stored(want)
        except Exception:
            stored = p.stored(None)
        specs = []
        for w in widgets:
            d = w.spec()
            d["value"] = stored[w.name]
            specs.append(d)
        _cp_host.panel('{"id":' + json.dumps(pid) + ',"widgets":' + json.dumps(specs) + ',"first":' + p.result(stored) + "}")
        return fn

    return deco


def _event(text_):
    """Called by the page's worker for a widget change. Answers through _cp_host.panelOut, never raises."""
    pid = ""
    try:
        req = json.loads(text_)
        pid = req.get("panel", "")
        p = _panels.get(pid)
        if p is None:
            raise ValueError("no such panel " + repr(pid))
        _cp_host.panelOut(p.result(p.stored(req.get("values"))))
    except Exception as e:
        _cp_host.panelOut(json.dumps({"panel": pid, "error": type(e).__name__ + ": " + str(e)}))
