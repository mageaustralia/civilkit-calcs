"""A very small scipy for the light numpy: only scipy.linalg (see scipy/linalg.py). Any other name raises NotImplementedError."""
import _route


def __getattr__(name):
    if name == "linalg":
        import scipy.linalg as m
        return m
    raise _route.missing("scipy", name)
