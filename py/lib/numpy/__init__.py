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
                          _kind_of, _wrap, _ni, _cast, _shape_arg, _skind, _nan, _inf)

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


# Names numpy has that this subset does not. Every public name this subset lacks raises NotImplementedError (see _route.py), listed here or not: the
# real numpy decides, which also reports a genuine typo. The list is documentation of what is known to exist; private and dunder names stay AttributeError.
_NOT_IMPLEMENTED = ("""
add subtract multiply divide power mod remainder fmod negative positive sign signbit copysign hypot arctan2 arcsinh arccosh
arctanh deg2rad rad2deg degrees radians exp2 expm1 log1p cbrt reciprocal rint fix trunc modf frexp ldexp gcd lcm
logical_and logical_or logical_not logical_xor bitwise_and bitwise_or bitwise_xor invert left_shift right_shift greater
greater_equal less less_equal equal not_equal isscalar isreal iscomplex nan_to_num nansum nanmean nanmax nanmin nanstd
nanvar nanmedian nanargmax nanargmin nanprod nancumsum percentile quantile nanpercentile cumprod diff ediff1d gradient trapz
trapezoid cross inner outer tensordot einsum kron vdot fft random polynomial polyfit polyval poly1d roots interp
convolve correlate cov corrcoef histogram histogram2d histogramdd bincount digitize unique intersect1d union1d setdiff1d in1d
isin sort argsort lexsort searchsorted partition argpartition nonzero flatnonzero argwhere extract compress take choose select
piecewise place put putmask delete insert append resize pad tile repeat roll rot90 flip fliplr flipud swapaxes moveaxis
rollaxis transpose reshape ravel squeeze expand_dims atleast_1d atleast_2d atleast_3d broadcast_to broadcast_arrays
meshgrid mgrid ogrid indices ix_ diag diagonal diagflat tri tril triu eye identity empty empty_like full_like fromfunction
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
