"""A numpy-faithful subset over one array class (`numpy._array.ndarray`), for MicroPython.

What is implemented gives numpy's answer; anything else raises NotImplementedError, never a different answer.
docs/python-blocks.md has the table.
"""
import math

import _route

from numpy import _array as _a
from numpy._array import (ndarray, asarray, mean, std, var, argmax, argmin, count_nonzero, median, average, cumsum, dot, sqrt, exp,
                          log, log10, log2, sin, cos, tan, arcsin, arccos, arctan, sinh, cosh, tanh, floor, ceil, square,
                          absolute, isnan, isinf, isfinite, maximum, minimum, clip, where, isclose, allclose, concatenate,
                          _kind_of, _wrap, _ni, _cast, _shape_arg, _skind, _nan, _inf, _pair, _scalar_or)

pi = math.pi
e = math.e
inf = _inf
nan = _nan
newaxis = None
float64 = float
int64 = int
bool_ = bool
dtype = _a.dtype_

array = _a.array_
sum = _a.sum_
prod = _a.prod_
max = amax = _a.max_
min = amin = _a.min_
any = _a.any_
all = _a.all_
abs = _a.abs_
round = around = _a.round_
matmul = dot


def zeros(shape, dtype=float):
    sh = _shape_arg(shape)
    k = _kind_of(dtype)
    n = 1
    for m in sh:
        n *= m
    return _wrap([_cast(k, 0)] * n, sh, k)


def ones(shape, dtype=float):
    sh = _shape_arg(shape)
    k = _kind_of(dtype)
    n = 1
    for m in sh:
        n *= m
    return _wrap([_cast(k, 1)] * n, sh, k)


def full(shape, fill_value, dtype=None):
    sh = _shape_arg(shape)
    k = _kind_of(dtype) or _skind(fill_value)
    if k is None:
        raise _ni("full with a " + type(fill_value).__name__)
    n = 1
    for m in sh:
        n *= m
    return _wrap([_cast(k, fill_value)] * n, sh, k)


def zeros_like(a, dtype=None):
    a = asarray(a)
    return zeros(a.shape, dtype or a.dtype)


def ones_like(a, dtype=None):
    a = asarray(a)
    return ones(a.shape, dtype or a.dtype)


def linspace(start, stop, num=50, endpoint=True, retstep=False, dtype=None, axis=0):
    if retstep or axis != 0:
        raise _ni("linspace(retstep=/axis=)")
    if num < 0:
        raise ValueError("Number of samples, %d, must be non-negative." % num)
    div = (num - 1) if endpoint else num
    start = float(start)
    stop = float(stop)
    if num == 0:
        out = []
    elif div > 0:
        step = (stop - start) / div
        out = [i * step + start for i in range(num)]
        if endpoint:
            out[-1] = stop
    else:
        out = [start] * num
    r = _wrap(out, (num,), "f")
    return r if dtype is None else r.astype(dtype)


def logspace(start, stop, num=50, endpoint=True, base=10.0, dtype=None):
    lin = linspace(start, stop, num, endpoint)
    r = _wrap([base ** v for v in lin._vals()], (num,), "f")
    return r if dtype is None else r.astype(dtype)


def arange(start, stop=None, step=1, dtype=None):
    if stop is None:
        start, stop = 0, start
    if step == 0:
        raise ZeroDivisionError("Maximum allowed size exceeded")
    isint = _skind(start) in ("i", "b") and _skind(stop) in ("i", "b") and _skind(step) in ("i", "b")
    n = int(math.ceil((stop - start) / step))
    if n < 0:
        n = 0
    if isint:
        out = [start + i * step for i in range(n)]
        k = "i"
    else:
        delta = (start + step) - start
        out = [start + i * delta for i in range(n)]
        k = "f"
    r = _wrap(out, (n,), k)
    return r if dtype is None else r.astype(dtype)


def loadtxt(fname, dtype=float, comments="#", delimiter=None, skiprows=0, **kw):
    rows = []
    with open(fname) as f:
        lines = f.read().split("\n")
    for i, line in enumerate(lines):
        if i < skiprows:
            continue
        line = line.split(comments)[0].strip()
        if not line:
            continue
        parts = line.split(delimiter) if delimiter else line.split()
        rows.append([dtype(p) for p in parts])
    if rows and len(rows[0]) == 1:
        return array([r[0] for r in rows], dtype)
    return array(rows, dtype)


def diff(a, n=1):
    v = asarray(a).tolist()
    if v and isinstance(v[0], list):
        raise _ni("diff of a 2-D array")
    for _ in range(n):
        v = [v[i + 1] - v[i] for i in range(len(v) - 1)]
    return array(v)


def trapezoid(y, x=None, dx=1.0):
    ys = asarray(y).tolist()
    xs = asarray(x).tolist() if x is not None else [i * dx for i in range(len(ys))]
    total = 0.0
    for i in range(len(ys) - 1):
        total += (xs[i + 1] - xs[i]) * (ys[i] + ys[i + 1]) / 2
    return total


trapz = trapezoid


def eye(n, m=None, k=0, dtype=float):
    m = n if m is None else m
    return array([[1 if j - i == k else 0 for j in range(m)] for i in range(n)], dtype)


def _hypot(u, v):
    u = _a._abs(u)
    v = _a._abs(v)
    if u == _inf or v == _inf:
        return _inf
    if u != u or v != v:
        return _nan
    big = u if u >= v else v
    if big == 0:
        return 0.0
    r = (v if u >= v else u) / big
    return big * math.sqrt(1.0 + r * r)


def hypot(x1, x2):
    """sqrt(x1**2 + x2**2) element by element, without overflow for large values."""
    a, b, shape, p, q = _pair(x1, x2)
    return _scalar_or([_hypot(u, v) for u, v in zip(p, q)], shape, "f")


def arctan2(x1, x2):
    """The angle of the point (x2, x1) in radians, element by element (math.atan2 with numpy's argument order)."""
    a, b, shape, p, q = _pair(x1, x2)
    return _scalar_or([math.atan2(u, v) for u, v in zip(p, q)], shape, "f")


def meshgrid(*xi, copy=True, sparse=False, indexing="xy"):
    """Coordinate arrays from 1-D coordinate vectors; indexing "xy" (the default, rows follow the second vector) or "ij"."""
    if indexing not in ("xy", "ij"):
        raise ValueError("Valid values for `indexing` are 'xy' and 'ij'.")
    arrs = [asarray(x).ravel() for x in xi]
    nd = len(arrs)
    if nd == 0:
        return ()
    ns = [a.size for a in arrs]
    swap = indexing == "xy" and nd > 1
    # The position of input k among the output axes.
    axis_of = list(range(nd))
    if swap:
        axis_of[0], axis_of[1] = 1, 0
    full = list(ns)
    if swap:
        full[0], full[1] = full[1], full[0]
    out = []
    for k, a in enumerate(arrs):
        vals = a._vals()
        if sparse:
            sh = [1] * nd
            sh[axis_of[k]] = ns[k]
            out.append(_wrap(list(vals), tuple(sh), a._k))
            continue
        # Row-major walk over the full shape: the value repeats `inner` times and the pattern `outer` times.
        ax = axis_of[k]
        inner = 1
        for d in full[ax + 1:]:
            inner *= d
        outer = 1
        for d in full[:ax]:
            outer *= d
        flat = []
        for _ in range(outer):
            for v in vals:
                flat.extend([v] * inner)
        out.append(_wrap(flat, tuple(full), a._k))
    return tuple(out)


def outer(a, b, out=None):
    """The outer product of two vectors (both are flattened first), as numpy.outer."""
    if out is not None:
        raise _ni("outer(out=)")
    x = asarray(a).ravel()
    y = asarray(b).ravel()
    k = "f" if "f" in (x._k, y._k) else "i"
    yv = y._vals()
    return _wrap([_cast(k, u * v) for u in x._vals() for v in yv], (x.size, y.size), k)


def interp(x, xp, fp, left=None, right=None, period=None):
    """Linear interpolation in the increasing points (xp, fp), as numpy.interp: outside the range the end values (or left/right)."""
    if period is not None:
        raise _ni("interp(period=)")
    xs = asarray(xp).ravel()._vals()
    fs = asarray(fp).ravel()._vals()
    if len(xs) != len(fs):
        raise ValueError("fp and xp are not of the same length.")
    if not xs:
        raise ValueError("array of sample points is empty")
    lo = float(fs[0]) if left is None else float(left)
    hi = float(fs[-1]) if right is None else float(right)

    def one(t):
        if t != t:
            return _nan
        if t < xs[0]:
            return lo
        if t > xs[-1]:
            return hi
        if t == xs[-1]:
            return float(fs[-1])
        # Binary search for xs[j] <= t < xs[j + 1].
        i, j = 0, len(xs) - 1
        while j - i > 1:
            m = (i + j) // 2
            if xs[m] <= t:
                i = m
            else:
                j = m
        x0, x1 = xs[i], xs[j]
        if x1 == x0:
            return float(fs[i])
        s = (t - x0) / (x1 - x0)
        return fs[i] + s * (fs[j] - fs[i])
    if isinstance(x, (int, float)):
        return one(x)
    a = asarray(x)
    return _wrap([one(t) for t in a._vals()], a._sh, "f")


def polyval(p, x):
    """The polynomial with coefficients p (highest power first) at x, by Horner's rule."""
    cs = asarray(p).ravel()._vals()

    def one(t):
        r = 0
        for c in cs:
            r = r * t + c
        return r
    if isinstance(x, (int, float)):
        return one(x)
    a = asarray(x)
    k = "f" if "f" in (a._k, asarray(p)._k) else "i"
    return _wrap([_cast(k, one(t)) for t in a._vals()], a._sh, k)


def polyfit(x, y, deg, rcond=None, full=False, w=None, cov=False):
    """Least-squares polynomial fit of degree deg, highest power first, as numpy.polyfit (Householder QR of the scaled
    Vandermonde matrix, so a fit of moderate degree keeps its accuracy)."""
    if full or w is not None or cov:
        raise _ni("polyfit(full=/w=/cov=)")
    xs = [float(v) for v in asarray(x).ravel()._vals()]
    ys = asarray(y)
    if len(ys._sh) != 1:
        raise _ni("polyfit of a 2-D y")
    ys = [float(v) for v in ys._vals()]
    deg = int(deg)
    if deg < 0:
        raise ValueError("expected deg >= 0")
    if len(xs) == 0:
        raise TypeError("expected non-empty vector for x")
    if len(xs) != len(ys):
        raise TypeError("expected x and y to have same length")
    m, n = len(xs), deg + 1
    A = [[t ** (deg - j) for j in range(n)] for t in xs]
    # Column scaling, as numpy does, then Householder QR on A and y together.
    scale = []
    for j in range(n):
        s = math.sqrt(_a._sum([A[i][j] * A[i][j] for i in range(m)]))
        s = s if s else 1.0
        scale.append(s)
        for i in range(m):
            A[i][j] /= s
    b = list(ys)
    for j in range(_a._min(n, m)):
        norm = math.sqrt(_a._sum([A[i][j] * A[i][j] for i in range(j, m)]))
        if norm == 0:
            continue
        alpha = -norm if A[j][j] >= 0 else norm
        v = [0.0] * m
        v[j] = A[j][j] - alpha
        for i in range(j + 1, m):
            v[i] = A[i][j]
        vv = _a._sum([v[i] * v[i] for i in range(j, m)])
        if vv == 0:
            continue
        for c in range(j, n):
            d = 2 * _a._sum([v[i] * A[i][c] for i in range(j, m)]) / vv
            for i in range(j, m):
                A[i][c] -= d * v[i]
        d = 2 * _a._sum([v[i] * b[i] for i in range(j, m)]) / vv
        for i in range(j, m):
            b[i] -= d * v[i]
    c = [0.0] * n
    for j in range(_a._min(n, m) - 1, -1, -1):
        t = b[j] - _a._sum([A[j][k] * c[k] for k in range(j + 1, n)])
        c[j] = t / A[j][j] if A[j][j] != 0 else 0.0
    return _wrap([c[j] / scale[j] for j in range(n)], (n,), "f")


# Names numpy has that this subset does not. Every public name this subset lacks raises NotImplementedError (see _route.py), listed here or not: the
# real numpy decides, which also reports a genuine typo. The list is documentation of what is known to exist; private and dunder names stay AttributeError.
_NOT_IMPLEMENTED = ("""
add subtract multiply divide power mod remainder fmod negative positive sign signbit copysign arcsinh arccosh
arctanh deg2rad rad2deg degrees radians exp2 expm1 log1p cbrt reciprocal rint fix trunc modf frexp ldexp gcd lcm
logical_and logical_or logical_not logical_xor bitwise_and bitwise_or bitwise_xor invert left_shift right_shift greater
greater_equal less less_equal equal not_equal isscalar isreal iscomplex nan_to_num nansum nanmean nanmax nanmin nanstd
nanvar nanmedian nanargmax nanargmin nanprod nancumsum percentile quantile nanpercentile cumprod diff ediff1d gradient trapz
trapezoid cross inner tensordot einsum kron vdot fft random polynomial poly1d roots
convolve correlate cov corrcoef histogram histogram2d histogramdd bincount digitize unique intersect1d union1d setdiff1d in1d
isin sort argsort lexsort searchsorted partition argpartition nonzero flatnonzero argwhere extract compress take choose select
piecewise place put putmask delete insert append resize pad tile repeat roll rot90 flip fliplr flipud swapaxes moveaxis
rollaxis transpose reshape ravel squeeze expand_dims atleast_1d atleast_2d atleast_3d broadcast_to broadcast_arrays
mgrid ogrid indices ix_ diag diagonal diagflat tri tril triu eye identity empty empty_like full_like fromfunction
fromiter frombuffer fromstring genfromtxt savetxt save load savez tofile array_equal array_equiv array_split split hsplit vsplit
dsplit stack vstack hstack dstack column_stack row_stack block copy shape size ndim result_type promote_types can_cast
min_scalar_type iinfo finfo float32 float16 int8 int16 int32 uint8 uint16 uint32 uint64 complex64 complex128 complex_ str_ object_
datetime64 timedelta64 matrix asmatrix ma ufunc vectorize frompyfunc apply_along_axis apply_over_axes ptp nextafter spacing
heaviside fmax fmin real imag conj conjugate angle unwrap sinc i0 kaiser hanning hamming blackman bartlett
cumulative_sum cumulative_prod vecdot matvec vecmat ndenumerate ndindex nditer flatiter
""").split()


def __getattr__(name):
    if name == "linalg":
        import numpy.linalg as _linalg
        return _linalg
    if name in _NOT_IMPLEMENTED:
        raise _route.missing("numpy", name)
    raise _route.missing("numpy", name)
