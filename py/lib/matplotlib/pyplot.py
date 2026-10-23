import _route

from matplotlib._chart import (figure, gcf, gca, subplots, subplot, show, close, savefig, tight_layout,
                              Figure, Axes)


def plot(*a, **kw):
    return gca().plot(*a, **kw)


def scatter(*a, **kw):
    return gca().scatter(*a, **kw)


def bar(*a, **kw):
    return gca().bar(*a, **kw)


def fill_between(*a, **kw):
    return gca().fill_between(*a, **kw)


def axhline(*a, **kw):
    return gca().axhline(*a, **kw)


def axvline(*a, **kw):
    return gca().axvline(*a, **kw)


def suptitle(t, **kw):
    gcf().suptitle(t, **kw)


def text(*a, **kw):
    return gca().text(*a, **kw)


def xlabel(s, **kw):
    gca().set_xlabel(s, **kw)


def ylabel(s, **kw):
    gca().set_ylabel(s, **kw)


def title(s, **kw):
    gca().set_title(s, **kw)


def xlim(*a, **kw):
    return gca().set_xlim(*a, **kw)


def ylim(*a, **kw):
    return gca().set_ylim(*a, **kw)


def xscale(v, **kw):
    gca().set_xscale(v)


def yscale(v, **kw):
    gca().set_yscale(v)


def grid(*a, **kw):
    gca().grid(*a, **kw)


def minorticks_on():
    gca().minorticks_on()


def legend(*a, **kw):
    gca().legend(*a, **kw)


def axis(arg=None, **kw):
    if arg == "equal":
        gca().set_aspect("equal")
        return None
    raise NotImplementedError("matplotlib: axis(" + repr(arg) + ")")


def __getattr__(name):
    raise _route.missing("matplotlib.pyplot", name)
