"""A recorder with matplotlib's shape (see matplotlib/_chart.py); the host draws the SVG."""
import _route


def use(*args, **kw):
    pass


def __getattr__(name):
    raise _route.missing("matplotlib", name)
