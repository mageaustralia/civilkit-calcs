"""The figure recorder behind matplotlib.pyplot in the light Python stand-in.

It records pyplot and Axes calls as a plain specification (lines, scatter, bars, fills, reference lines, text and the axis decoration) and hands
the finished specification to the host (`_cp_host.figure`), which draws the SVG. Nothing is drawn here. A name the recorder
does not implement raises NotImplementedError("matplotlib: <name>").
"""
import json
import _cp_host



def _nope(name):
    return NotImplementedError("matplotlib: " + name)


def _list(v):
    """A sequence as a plain list of numbers (None stays None); a scalar becomes a one-element list."""
    if isinstance(v, (int, float)):
        return [v]
    return [x for x in v]


_LINE_KW = {"color": "color", "c": "color", "linestyle": "linestyle", "ls": "linestyle", "linewidth": "linewidth",
            "lw": "linewidth", "marker": "marker", "markersize": "markersize", "ms": "markersize",
            "markerfacecolor": "markerfacecolor", "mfc": "markerfacecolor", "markeredgecolor": "markeredgecolor",
            "mec": "markeredgecolor", "alpha": "alpha", "label": "label"}


class Line2D:
    def __init__(self, item):
        self._item = item


class Axes:
    def __init__(self, figure, index):
        self.figure = figure
        self._index = index
        self._items = []
        self._o = {}

    def __getattr__(self, name):
        if name.startswith("_"):
            raise AttributeError(name)
        raise _nope("Axes." + name)

    # ---- drawing ----
    def plot(self, *args, **kw):
        args = list(args)
        fmt = None
        lines = []
        while args:
            if len(args) >= 3 and isinstance(args[2], str):
                x, y, fmt = _list(args[0]), _list(args[1]), args[2]
                args = args[3:]
            elif len(args) >= 2 and not isinstance(args[1], str):
                x, y = _list(args[0]), _list(args[1])
                args = args[2:]
            elif len(args) >= 2:
                y, fmt = _list(args[0]), args[1]
                x = list(range(len(y)))
                args = args[2:]
            else:
                y = _list(args[0])
                x = list(range(len(y)))
                args = args[1:]
            item = {"type": "line", "x": x, "y": y}
            if fmt:
                item["fmt"] = fmt
            for k in kw:
                if k in _LINE_KW:
                    item[_LINE_KW[k]] = kw[k]
                elif k in ("zorder", "solid_capstyle", "dash_capstyle", "markeredgewidth", "mew", "drawstyle", "scalex",
                           "scaley", "data", "picker", "clip_on", "visible", "rasterized"):
                    pass
                else:
                    raise _nope("Axes.plot(" + k + "=)")
            self._items.append(item)
            lines.append(Line2D(item))
            fmt = None
        return lines

    def scatter(self, x, y, s=None, c=None, marker=None, color=None, label=None, **kw):
        item = {"type": "scatter", "x": _list(x), "y": _list(y)}
        if s is not None:
            item["s"] = s
        if marker is not None:
            item["marker"] = marker
        col = color if color is not None else c
        if isinstance(col, str):
            item["color"] = col
        if label is not None:
            item["label"] = label
        self._items.append(item)
        return Line2D(item)

    def text(self, x, y, s, fontsize=None, color=None, ha=None, va=None, horizontalalignment=None,
             verticalalignment=None, **kw):
        item = {"type": "text", "x": x, "y": y, "s": s}
        if fontsize is not None:
            item["fontsize"] = fontsize
        if color is not None:
            item["color"] = color
        ha = ha or horizontalalignment
        va = va or verticalalignment
        if ha:
            item["ha"] = ha
        if va:
            item["va"] = va
        self._items.append(item)
        return None

    def bar(self, x, height, width=0.8, bottom=0, color=None, label=None, align="center", **kw):
        xs = _list(x)
        hs = _list(height) if not isinstance(height, (int, float)) else [height] * len(xs)
        bs = _list(bottom) if not isinstance(bottom, (int, float)) else [bottom] * len(xs)
        ws = _list(width) if not isinstance(width, (int, float)) else [width] * len(xs)
        item = {"type": "bar", "x": xs, "height": hs, "width": ws, "bottom": bs, "align": align}
        if isinstance(color, str):
            item["color"] = color
        if label is not None:
            item["label"] = label
        self._items.append(item)
        return Line2D(item)

    def fill_between(self, x, y1, y2=0, color=None, alpha=None, label=None, **kw):
        xs = _list(x)
        a = _list(y1) if not isinstance(y1, (int, float)) else [y1] * len(xs)
        b = _list(y2) if not isinstance(y2, (int, float)) else [y2] * len(xs)
        item = {"type": "fill_between", "x": xs, "y1": a, "y2": b}
        if isinstance(color, str):
            item["color"] = color
        elif isinstance(kw.get("facecolor"), str):
            item["color"] = kw["facecolor"]
        if alpha is not None:
            item["alpha"] = alpha
        if label is not None:
            item["label"] = label
        self._items.append(item)
        return Line2D(item)

    def axhline(self, y=0, color=None, linestyle=None, ls=None, linewidth=None, lw=None, label=None, **kw):
        item = {"type": "hline", "y": y}
        self._ref_style(item, color, linestyle or ls, linewidth or lw, label)
        self._items.append(item)
        return Line2D(item)

    def axvline(self, x=0, color=None, linestyle=None, ls=None, linewidth=None, lw=None, label=None, **kw):
        item = {"type": "vline", "x": x}
        self._ref_style(item, color, linestyle or ls, linewidth or lw, label)
        self._items.append(item)
        return Line2D(item)

    def _ref_style(self, item, color, linestyle, linewidth, label):
        if color is not None:
            item["color"] = color
        if linestyle is not None:
            item["linestyle"] = linestyle
        if linewidth is not None:
            item["linewidth"] = linewidth
        if label is not None:
            item["label"] = label

    # ---- decoration ----
    def set_xlabel(self, label, **kw):
        self._o["xlabel"] = label

    def set_ylabel(self, label, **kw):
        self._o["ylabel"] = label

    def set_title(self, label, **kw):
        self._o["title"] = label

    def get_title(self):
        return self._o.get("title", "")

    def set_xlim(self, left=None, right=None, **kw):
        if right is None and left is not None and not isinstance(left, (int, float)):
            left, right = left[0], left[1]
        self._o["xlim"] = [left, right]
        return (left, right)

    def set_ylim(self, bottom=None, top=None, **kw):
        if top is None and bottom is not None and not isinstance(bottom, (int, float)):
            bottom, top = bottom[0], bottom[1]
        self._o["ylim"] = [bottom, top]
        return (bottom, top)

    def get_xlim(self):
        v = self._o.get("xlim")
        return tuple(v) if v else (0.0, 1.0)

    def get_ylim(self):
        v = self._o.get("ylim")
        return tuple(v) if v else (0.0, 1.0)

    def set_xscale(self, v, **kw):
        self._o["xscale"] = v

    def set_yscale(self, v, **kw):
        self._o["yscale"] = v

    def set_aspect(self, aspect, **kw):
        if aspect == "equal":
            self._o["aspect"] = "equal"

    def grid(self, visible=None, which="major", axis="both", **kw):
        if visible is False:
            self._o.pop("grid", None)
            return
        ls = kw.get("linestyle", kw.get("ls"))
        if which == "major" and axis == "both" and ls is None:
            self._o["grid"] = True
        else:
            g = {"which": which, "axis": axis}
            if ls is not None:
                g["linestyle"] = ls
            self._o["grid"] = g

    def minorticks_on(self):
        self._o["minorticks"] = True

    def legend(self, *args, loc=None, bbox_to_anchor=None, **kw):
        lg = {}
        if loc is not None:
            lg["loc"] = loc
        if bbox_to_anchor is not None:
            lg["bbox_to_anchor"] = list(bbox_to_anchor)
        if args and not isinstance(args[0], str) and isinstance(args[0], (list, tuple)):
            lg["labels"] = list(args[0])
        self._o["legend"] = lg if lg else True

    def get_legend(self):
        return self._o.get("legend")

    def get_figure(self):
        return self.figure

    def _spec(self):
        d = {"index": self._index}
        for k in self._o:
            d[k] = self._o[k]
        d["items"] = self._items
        return d


class _AxArray:
    """What plt.subplots returns for several axes: a sequence that also accepts [row, col] and has flatten().
    (Not a list subclass: MicroPython cannot call list.__getitem__ from a subclass, so axes[0] would fail.)"""

    def __init__(self, items):
        self._items = list(items)

    def __len__(self):
        return len(self._items)

    def __iter__(self):
        return iter(self._items)

    def __getitem__(self, i):
        if isinstance(i, tuple):
            return self._items[i[0]][i[1]]
        return self._items[i]

    def flatten(self):
        out = []
        for a in self._items:
            if isinstance(a, _AxArray):
                out.extend(a._items)
            else:
                out.append(a)
        return out

    ravel = flatten

    @property
    def flat(self):
        return self.flatten()


class Figure:
    def __init__(self, figsize=None, nrows=1, ncols=1, tight=False, dpi=None):
        self.figsize = figsize
        self.nrows = nrows
        self.ncols = ncols
        self.tight = tight
        self.dpi = dpi
        self.axes = []
        self.number = 0
        self._shown = False
        self._suptitle = None

    def add_subplot(self, *args, **kw):
        if len(args) == 1 and isinstance(args[0], int) and args[0] >= 111:
            r, c, i = args[0] // 100, (args[0] // 10) % 10, args[0] % 10
        elif len(args) == 3:
            r, c, i = args
        else:
            r, c, i = 1, 1, 1
        if not self.axes:
            self.nrows, self.ncols = r, c
        while len(self.axes) < i:
            self.axes.append(Axes(self, len(self.axes)))
        return self.axes[i - 1]

    def _make_axes(self):
        self.axes = [Axes(self, i) for i in range(self.nrows * self.ncols)]

    def get_axes(self):
        return self.axes

    def tight_layout(self, **kw):
        self.tight = True

    def savefig(self, fname=None, **kw):
        self._render()

    def suptitle(self, t, **kw):
        self._suptitle = t

    def set_size_inches(self, w, h=None, **kw):
        self.figsize = (w, h) if h is not None else tuple(w)

    def __getattr__(self, name):
        if name.startswith("_"):
            raise AttributeError(name)
        raise _nope("Figure." + name)

    def _spec(self):
        s = {"axes": [a._spec() for a in self.axes]}
        if self.figsize:
            s["figsize"] = list(self.figsize)
        if self.dpi:
            s["dpi"] = self.dpi
        if self.nrows != 1 or self.ncols != 1:
            s["nrows"] = self.nrows
            s["ncols"] = self.ncols
        if self.tight:
            s["tight_layout"] = self.tight
        if self._suptitle is not None:
            s["suptitle"] = self._suptitle
        return s

    def _render(self):
        _cp_host.figure(json.dumps(self._spec()))
        self._shown = True


_open = []
_state = {"current": None}


def figure(num=None, figsize=None, dpi=None, **kw):
    f = Figure(figsize=figsize, dpi=dpi)
    f.number = len(_open) + 1
    _open.append(f)
    _state["current"] = f
    return f


def gcf():
    if _state["current"] is None:
        figure()
    return _state["current"]


def gca():
    f = gcf()
    if not f.axes:
        f.nrows = f.ncols = 1
        f._make_axes()
    return f.axes[-1]


def subplots(nrows=1, ncols=1, figsize=None, dpi=None, **kw):
    f = figure(figsize=figsize, dpi=dpi)
    f.nrows, f.ncols = nrows, ncols
    f._make_axes()
    if nrows == 1 and ncols == 1:
        return f, f.axes[0]
    if nrows == 1 or ncols == 1:
        return f, _AxArray(f.axes)
    return f, _AxArray([_AxArray(f.axes[r * ncols:(r + 1) * ncols]) for r in range(nrows)])


def subplot(*args, **kw):
    return gcf().add_subplot(*args, **kw)


def show(*args, **kw):
    for f in list(_open):
        if f.axes and not f._shown:
            f._render()
    del _open[:]
    _state["current"] = None


def close(*args, **kw):
    del _open[:]
    _state["current"] = None


def savefig(fname=None, **kw):
    gcf().savefig(fname)


def tight_layout(**kw):
    gcf().tight_layout()
