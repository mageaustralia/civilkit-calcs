"""The array class of the light numpy: a small numpy-faithful `ndarray` for MicroPython.

Storage is one flat Python list (of floats, ints or bools; floats are the objects themselves, so reading one allocates nothing),
plus a shape, strides and an offset, so a slice is a view of the same storage (writes through it show in the original, as in
numpy). dtypes: float64, int64, bool. A list rather than array('d'): the MicroPython runtime never frees memory, and
reading an array('d') element boxes a new float each time, which costs about 3.5 times the allocation and twice the time.

The contract: what is implemented gives numpy's answer; everything else raises NotImplementedError saying it is not in the
light Python stand-in. Never a silently wrong answer. Divergences that remain: summation order, int64 overflow, and
`np.float64` scalars are Python floats.
"""
import math

_inf = float("inf")
_nan = float("nan")
_sum = sum
_max = max
_min = min
_abs = abs
_round = round
_any = any
_all = all

_RANK = {"b": 0, "i": 1, "f": 2}
_DT = {"b": "bool", "i": "int64", "f": "float64"}


def _ni(what):
    """The error for something outside the subset: names the operation and the way out."""
    return NotImplementedError(what + " is not in this light numpy (a.tolist() gives plain lists)")


def _rj(s, w, c=" "):
    """str.rjust (MicroPython's str has no rjust / ljust)."""
    return c * (w - len(s)) + s if len(s) < w else s


def _lj(s, w, c=" "):
    return s + c * (w - len(s)) if len(s) < w else s


def _skind(x):
    """'b', 'i' or 'f' for a Python scalar, None for anything else."""
    if isinstance(x, bool):
        return "b"
    if isinstance(x, int):
        return "i"
    if isinstance(x, float):
        return "f"
    return None


def _shs(shape):
    if len(shape) == 1:
        return "(%d,)" % shape[0]
    return "(" + ", ".join([str(n) for n in shape]) + ")"


def _prod(shape):
    n = 1
    for k in shape:
        n *= k
    return n


def _cstrides(shape):
    st = []
    s = 1
    for n in reversed(shape):
        st.append(s)
        s *= n
    st.reverse()
    return tuple(st)


def _offsets(shape, strides, off):
    """Flat storage offsets of every element, in C order."""
    idx = [off]
    for n, s in zip(shape, strides):
        steps = [k * s for k in range(n)]
        idx = [i + t for i in idx for t in steps]
    return idx


_PT = {"b": bool, "i": int, "f": float}


def _store(vals, kind):
    """The storage list for vals: vals itself when it is a list that already holds exactly the dtype's Python type, else a
    converted copy (so a float array never holds an int, and a bool array never holds 0 or 1)."""
    t = _PT[kind]
    if isinstance(vals, list):
        for v in vals:
            if type(v) is not t:
                return [t(x) for x in vals]
        return vals
    return [t(x) for x in vals]


def _wrap(vals, shape, kind):
    return ndarray(_store(vals, kind), tuple(shape), kind)


def _cast(kind, v):
    if kind == "f":
        return float(v)
    if kind == "i":
        return int(v)
    return bool(v)


def _slice(sl, n):
    """(start, stop, step, length) of a slice over n items, as slice.indices."""
    step = 1 if sl.step is None else sl.step
    if step == 0:
        raise ValueError("slice step cannot be zero")
    if step > 0:
        lo, hi = 0, n
        start = lo if sl.start is None else sl.start
        stop = hi if sl.stop is None else sl.stop
    else:
        lo, hi = -1, n - 1
        start = hi if sl.start is None else sl.start
        stop = lo if sl.stop is None else sl.stop
    if sl.start is not None:
        if start < 0:
            start += n
        start = _min(_max(start, lo), hi)
    if sl.stop is not None:
        if stop < 0:
            stop += n
        stop = _min(_max(stop, lo), hi)
    if step > 0:
        ln = _max(0, (stop - start + step - 1) // step)
    else:
        ln = _max(0, (start - stop - step - 1) // (-step))
    return start, stop, step, ln


# ---------------------------------------------------------------------------------------------- scalar arithmetic as numpy does it
def _tdiv(x, y):
    try:
        return x / y
    except ZeroDivisionError:
        if x != x or x == 0:
            return _nan
        neg = (x < 0) != (math.copysign(1.0, y) < 0)
        return -_inf if neg else _inf


def _tfloordiv(x, y):
    try:
        return x // y
    except ZeroDivisionError:
        if isinstance(x, int) and isinstance(y, int):
            return 0
        return _tdiv(x, y) if x != 0 and x == x else _nan


def _tmod(x, y):
    try:
        return x % y
    except ZeroDivisionError:
        return 0 if isinstance(x, int) and isinstance(y, int) else _nan


def _tpow(x, y):
    try:
        r = x ** y
    except ZeroDivisionError:
        return _inf
    except OverflowError:
        odd = isinstance(y, int) and y % 2 == 1
        return -_inf if (x < 0 and odd) else _inf
    if isinstance(r, complex):
        return _nan
    return r


_F = {
    "+": lambda p, q: p + q,
    "-": lambda p, q: p - q,
    "*": lambda p, q: p * q,
    "/": _tdiv,
    "//": _tfloordiv,
    "%": _tmod,
    "**": _tpow,
    "<": lambda p, q: p < q,
    "<=": lambda p, q: p <= q,
    ">": lambda p, q: p > q,
    ">=": lambda p, q: p >= q,
    "==": lambda p, q: p == q,
    "!=": lambda p, q: p != q,
    "&": lambda p, q: p & q,
    "|": lambda p, q: p | q,
    "^": lambda p, q: p ^ q,
}
_CMP = ("<", "<=", ">", ">=", "==", "!=")
_BIT = ("&", "|", "^")
_NAME = {"+": "add", "-": "subtract", "*": "multiply", "/": "divide", "//": "floor_divide", "%": "remainder", "**": "power",
         "&": "bitwise_and", "|": "bitwise_or", "^": "bitwise_xor"}


def _ew(op, x, y, xs, ys):
    """Apply op element-wise. x and y are sequences when xs / ys is True, else scalars. Returns a list."""
    if xs and ys:
        n = len(x)
    if op == "+":
        if xs and ys:
            return [x[i] + y[i] for i in range(n)]
        if xs:
            return [p + y for p in x]
        return [x + q for q in y]
    if op == "-":
        if xs and ys:
            return [x[i] - y[i] for i in range(n)]
        if xs:
            return [p - y for p in x]
        return [x - q for q in y]
    if op == "*":
        if xs and ys:
            return [x[i] * y[i] for i in range(n)]
        if xs:
            return [p * y for p in x]
        return [x * q for q in y]
    if op == "/":
        try:
            if xs and ys:
                return [x[i] / y[i] for i in range(n)]
            if xs:
                return [p / y for p in x]
            return [x / q for q in y]
        except ZeroDivisionError:
            pass
    f = _F[op]
    if xs and ys:
        return [f(x[i], y[i]) for i in range(n)]
    if xs:
        return [f(p, y) for p in x]
    return [f(x, q) for q in y]


def _rkind(op, ka, kb):
    if op in _CMP:
        return "b"
    if op in _BIT:
        if ka == "f" or kb == "f":
            raise TypeError("ufunc 'bitwise_" + {"&": "and", "|": "or", "^": "xor"}[op] + "' not supported for the input types")
        return "b" if (ka == "b" and kb == "b") else "i"
    if op == "/":
        return "f"
    if ka == "b" and kb == "b":
        raise _ni("boolean arithmetic (" + op + ") on a result array")
    return ka if _RANK[ka] >= _RANK[kb] else kb


def _bshape(sa, sb):
    n = _max(len(sa), len(sb))
    a = (1,) * (n - len(sa)) + tuple(sa)
    b = (1,) * (n - len(sb)) + tuple(sb)
    out = []
    for x, y in zip(a, b):
        if x == y or y == 1:
            out.append(x)
        elif x == 1:
            out.append(y)
        else:
            raise ValueError("operands could not be broadcast together with shapes " + _shs(sa) + " " + _shs(sb))
    return tuple(out)


def _bvals(arr, shape):
    """The elements of arr broadcast to `shape`, as a sequence in C order."""
    if arr._sh == shape:
        return arr._vals()
    n = len(shape)
    pad = n - len(arr._sh)
    strides = []
    for i in range(n):
        if i < pad:
            strides.append(0)
        else:
            j = i - pad
            strides.append(0 if arr._sh[j] == 1 else arr._st[j])
    d = arr._d
    return [d[i] for i in _offsets(shape, strides, arr._o)]


# ---------------------------------------------------------------------------------------------------------------- construction
def _flat(obj, depth, shape, out):
    if isinstance(obj, ndarray):
        obj = obj.tolist()
    if isinstance(obj, (list, tuple)):
        if depth >= len(shape) or len(obj) != shape[depth]:
            raise ValueError("setting an array element with a sequence. The requested array has an inhomogeneous shape")
        for x in obj:
            _flat(x, depth + 1, shape, out)
    else:
        if depth != len(shape):
            raise ValueError("setting an array element with a sequence. The requested array has an inhomogeneous shape")
        out.append(obj)


def _kind_of(dtype):
    if dtype is None:
        return None
    if isinstance(dtype, dtype_):
        return dtype._k
    if dtype is float or dtype in ("float", "float64", "f8", "d", "double", "<f8"):
        return "f"
    if dtype is bool or dtype in ("bool", "bool_", "?"):
        return "b"
    if dtype is int or dtype in ("int", "int64", "i8", "l", "<i8"):
        return "i"
    raise _ni("dtype " + repr(dtype))


class dtype_:
    """numpy's dtype, for the three kinds the array has."""

    def __init__(self, kind):
        if kind not in _DT:
            kind = _kind_of(kind)
        self._k = kind
        self.name = _DT[kind]
        self.kind = kind
        self.itemsize = 1 if kind == "b" else 8

    def __repr__(self):
        return "dtype('" + self.name + "')"

    def __str__(self):
        return self.name

    def __eq__(self, other):
        try:
            return _kind_of(other) == self._k
        except Exception:
            return False

    def __ne__(self, other):
        return not self.__eq__(other)


def asarray(obj, dtype=None):
    """np.asarray / np.array without the copy flag: an ndarray is returned as is (or converted when dtype differs)."""
    kind = _kind_of(dtype)
    if isinstance(obj, ndarray):
        if kind is None or kind == obj._k:
            return obj
        return obj.astype(kind)
    if not isinstance(obj, (list, tuple)):
        k = _skind(obj)
        if k is None:
            if isinstance(obj, (str, bytes, dict, complex)) or obj is None:
                raise _ni("an array of " + type(obj).__name__)
            if hasattr(obj, "__iter__"):
                obj = list(obj)
            else:
                raise _ni("an array from " + type(obj).__name__)
        else:
            return ndarray(_store([obj if kind is None else _cast(kind, obj)], kind or k), (), kind or k)
    shape = []
    o = obj
    while True:
        if isinstance(o, ndarray):
            o = o.tolist()
        if not isinstance(o, (list, tuple)):
            break
        shape.append(len(o))
        if len(o) == 0:
            break
        o = o[0]
    out = []
    _flat(obj, 0, shape, out)
    k = 0
    for x in out:
        t = _skind(x)
        if t is None:
            raise _ni("an array of " + type(x).__name__)
        if _RANK[t] > k:
            k = _RANK[t]
            if k == 2:
                break
    inferred = ("b", "i", "f")[k] if out else "f"
    if kind is None:
        kind = inferred
    elif kind != inferred:
        out = [_cast(kind, x) for x in out]
    return ndarray(_store(out, kind), tuple(shape), kind)


def from_list(vals, kind="f"):
    """A 1-D array straight from a list of numbers (no inspection of the items)."""
    return ndarray(_store(vals, kind), (len(vals),), kind)


def from_rows(rows, kind="f", ncols=None):
    """A 2-D array from a list of equal-length rows (ncols is needed only when there are no rows)."""
    if not rows:
        return ndarray(_store([], kind), (0, ncols or 0), kind)
    return ndarray(_store([v for r in rows for v in r], kind), (len(rows), len(rows[0])), kind)


def array_(obj, dtype=None, copy=True, **kw):
    """np.array: always a new array (a copy of an ndarray)."""
    if kw:
        raise _ni("np.array(" + ", ".join(sorted(kw)) + "=...)")
    r = asarray(obj, dtype)
    if r is obj:
        return r.copy()
    return r


def _shape_arg(shape):
    if isinstance(shape, int):
        return (shape,)
    return tuple(shape)


# -------------------------------------------------------------------------------------------------------------------- printing
def _fixed(x):
    s = ""
    for d in range(9):
        s = ("%." + str(d) + "f") % x
        if float(s) == x:
            break
    if "." in s:
        s = s.rstrip("0")
    else:
        s += "."
    return s


def _expo(x):
    s = ""
    for d in range(9):
        s = ("%." + str(d) + "e") % x
        if float(s) == x:
            break
    m, e = s.split("e")
    if "." in m:
        m = m.rstrip("0")
    else:
        m += "."
    return m + "e" + e


def _fkey(x):
    """Dictionary key for a float: nan equals itself, and -0.0 is not 0.0."""
    if x != x:
        return "nan"
    if x == 0:
        return "-0" if math.copysign(1.0, x) < 0 else "0"
    return x


def _fmt_floats(vals):
    """numpy's float formatting (precision 8, 'maxprec' mode) for a set of values: a function value -> text of a common width."""
    fin = [x for x in vals if x == x and x != _inf and x != -_inf]
    nz = [_abs(x) for x in fin if x != 0]
    use_exp = False
    if nz:
        mx = _max(nz)
        mn = _min(nz)
        use_exp = mx >= 1e8 or mn < 0.0001 or mx / mn > 1000.0
    parts = {}
    for x in fin:
        k = _fkey(x)
        if k in parts:
            continue
        if use_exp:
            m, e = _expo(x).split("e")
            i, f = m.split(".")
            parts[k] = (i, f, e[0], e[1:])
        else:
            i, f = _fixed(x).split(".")
            parts[k] = (i, f, "", "")
    pl = _max([len(parts[k][0]) for k in parts]) if parts else 0
    pr = _max([len(parts[k][1]) for k in parts]) if parts else 0
    es = _max([len(parts[k][3]) for k in parts]) if (parts and use_exp) else 0
    width_extra = (es + 2 + pr) if use_exp else pr
    neginf = _any([x == -_inf for x in vals])
    off = width_extra + 1
    if len(fin) != len(vals):
        pl = _max(pl, 3 - off, 3 + (1 if neginf else 0) - off)
    total = pl + off

    def fmt(x):
        if x != x:
            return _rj("nan", total)
        if x == _inf:
            return _rj("inf", total)
        if x == -_inf:
            return _rj("-inf", total)
        i, f, sg, ex = parts[_fkey(x)]
        if use_exp:
            return _rj(i, pl) + "." + _lj(f, pr, "0") + "e" + sg + _rj(ex, es, "0")
        return _rj(i, pl) + "." + _lj(f, pr)
    return fmt


def _format_values(vals, kind, nd=1):
    if kind == "f":
        return _fmt_floats(vals)
    if kind == "i":
        w = _max([len(str(v)) for v in vals])
        return lambda v: _rj(str(v), w)
    t = " True" if nd else "True"
    return lambda v: t if v else "False"


def _array2string(a, sep, prefix_len, suffix_len):
    """numpy's array2string for the default print options (precision 8, linewidth 75, threshold 1000, edgeitems 3)."""
    nd = len(a._sh)
    summary = a.size > 1000
    edge = 3
    d = a._d

    def walk(axis, off, visit):
        n = a._sh[axis]
        st = a._st[axis]
        if summary and n > 2 * edge:
            ids = list(range(edge)) + [None] + list(range(n - edge, n))
        else:
            ids = list(range(n))
        for i in ids:
            if i is None:
                visit(None, axis, off)
                continue
            o = off + i * st
            if axis == nd - 1:
                visit(o, axis, o)
            else:
                walk(axis + 1, o, visit)
    seen = []
    if nd == 0:
        shown = [d[a._o]]
    else:
        def collect(o, axis, _o):
            if o is not None:
                seen.append(d[o])
        walk(0, a._o, collect)
        shown = seen
    fmt = _format_values(shown, a._k, nd)
    width = 75 - suffix_len
    indent0 = " " * (1 + prefix_len)

    def rec(axis, off, hang, curr):
        if axis == nd:
            return fmt(d[off])
        nhang = hang + " "
        nwidth = curr - 1
        n = a._sh[axis]
        st = a._st[axis]
        show = summary and n > 2 * edge
        ids = (list(range(edge)) + [None] + list(range(n - edge, n))) if show else list(range(n))
        left = nd - axis
        if left == 1:
            elem_w = curr - _max(len(sep.rstrip()), 1)
            line = hang
            s = ""
            for k, i in enumerate(ids):
                word = "..." if i is None else rec(axis + 1, off + i * st, nhang, nwidth)
                if len(line) + len(word) > elem_w and len(line) > len(hang):
                    s += line.rstrip() + "\n"
                    line = hang
                line += word
                if k < len(ids) - 1:
                    line += sep
            s += line
        else:
            s = ""
            line_sep = sep.rstrip() + "\n" * (left - 1)
            for k, i in enumerate(ids):
                nested = "..." if i is None else rec(axis + 1, off + i * st, nhang, nwidth)
                s += hang + nested + (line_sep if k < len(ids) - 1 else "")
        return "[" + s[len(hang):] + "]"
    if nd == 0:
        return fmt(d[a._o])
    return rec(0, a._o, indent0, width)


# ------------------------------------------------------------------------------------------------------------------- the class
_UNSUPPORTED_METHODS = ("sort argsort searchsorted nonzero compress choose conj conjugate cumprod diagonal flat swapaxes take "
                        "trace view put repeat resize squeeze tobytes tofile partition argpartition byteswap getfield setfield "
                        "ptp dump dumps itemset newbyteorder setflags tostring real imag").split()


class ndarray:
    """numpy-faithful array over flat storage (see the module docstring). Build one with numpy.array, numpy.zeros, ..."""

    def __init__(self, d, shape, kind, off=0, strides=None):
        self._d = d
        self._sh = shape
        self._k = kind
        self._o = off
        self._st = _cstrides(shape) if strides is None else strides

    # ---- attributes
    @property
    def shape(self):
        return self._sh

    @property
    def ndim(self):
        return len(self._sh)

    @property
    def size(self):
        return _prod(self._sh)

    @property
    def dtype(self):
        return dtype_(self._k)

    @property
    def itemsize(self):
        return 1 if self._k == "b" else 8

    @property
    def nbytes(self):
        return self.size * self.itemsize

    @property
    def T(self):
        return ndarray(self._d, tuple(reversed(self._sh)), self._k, self._o, tuple(reversed(self._st)))

    def __getattr__(self, name):
        if name in _UNSUPPORTED_METHODS:
            raise _ni("ndarray." + name)
        raise AttributeError("'ndarray' object has no attribute '" + name + "'")

    # ---- storage access
    def _vals(self):
        """Elements in C order as a sequence (the storage itself when it is exactly this array: do not mutate)."""
        n = self.size
        d = self._d
        if self._st == _cstrides(self._sh):
            if self._o == 0 and len(d) == n:
                return d
            return d[self._o:self._o + n]
        return [d[i] for i in _offsets(self._sh, self._st, self._o)]

    def tolist(self):
        """Plain Python lists (a number for a 0-d array)."""
        v = self._vals()
        if not self._sh:
            return v[0]
        return _nest(v, self._sh)

    def copy(self):
        return ndarray(list(self._vals()), self._sh, self._k)

    def astype(self, dtype, **kw):
        if kw:
            raise _ni("astype(" + ", ".join(sorted(kw)) + "=...)")
        k = _kind_of(dtype)
        if k == self._k:
            return self.copy()
        return ndarray(_store([_cast(k, x) for x in self._vals()], k), self._sh, k)

    def item(self, *args):
        if args or self.size != 1:
            if self.size != 1:
                raise ValueError("can only convert an array of size 1 to a Python scalar")
            raise _ni("ndarray.item(index)")
        return self._vals()[0]

    def reshape(self, *shape, **kw):
        if kw:
            raise _ni("reshape(" + ", ".join(sorted(kw)) + "=...)")
        if len(shape) == 1 and isinstance(shape[0], (tuple, list)):
            shape = tuple(shape[0])
        shape = list(shape)
        n = self.size
        if shape.count(-1) > 1:
            raise ValueError("can only specify one unknown dimension")
        if -1 in shape:
            rest = _prod([k for k in shape if k != -1])
            if rest == 0 or n % rest:
                raise ValueError("cannot reshape array of size %d into shape %s" % (n, _shs(tuple(shape))))
            shape[shape.index(-1)] = n // rest
        shape = tuple(shape)
        if _prod(shape) != n:
            raise ValueError("cannot reshape array of size %d into shape %s" % (n, _shs(shape)))
        if self._st == _cstrides(self._sh):
            return ndarray(self._d, shape, self._k, self._o)
        return ndarray(list(self._vals()), shape, self._k)

    def ravel(self):
        return self.reshape(self.size)

    def flatten(self):
        return ndarray(list(self._vals()), (self.size,), self._k)

    def transpose(self, *axes):
        if axes:
            raise _ni("transpose(axes)")
        return self.T

    def fill(self, value):
        self[...] = value

    # ---- text
    def __repr__(self):
        if self.size == 0:
            if len(self._sh) == 1:
                return "array([], dtype=" + _DT[self._k] + ")"
            return "array([], shape=" + _shs(self._sh) + ", dtype=" + _DT[self._k] + ")"
        if self.size > 1000:
            return "array(" + _array2string(self, ", ", 6, 1) + ", shape=" + _shs(self._sh) + ")"
        return "array(" + _array2string(self, ", ", 6, 1) + ")"

    def __str__(self):
        if self.size == 0:
            return "[]"
        return _array2string(self, " ", 0, 0)

    # ---- sequence protocol
    def __len__(self):
        if not self._sh:
            raise TypeError("len() of unsized object")
        return self._sh[0]

    def __iter__(self):
        if not self._sh:
            raise TypeError("iteration over a 0-d array")
        if len(self._sh) == 1:
            return iter(self._vals())
        return self._rows()

    def _rows(self):
        for i in range(self._sh[0]):
            yield self[i]

    def __contains__(self, x):
        return bool((self == x).any())

    def __bool__(self):
        if self.size == 1:
            return bool(self._vals()[0])
        raise ValueError("The truth value of an array with more than one element is ambiguous. Use a.any() or a.all()")

    # No __float__ / __int__: MicroPython's float operators call them on the right operand and give up when they
    # raise, which breaks `2.5 * array`. Without them the operators fall through to the array's __rmul__ and friends.
    # (float(a) of a 0-d array is therefore not supported; reductions return plain Python numbers.)

    __hash__ = None

    # ---- indexing
    def _item_offset(self, key):
        """The storage offset of a tuple of ints that indexes every axis (m[i, j]), or None for any other key."""
        sh = self._sh
        if type(key) is not tuple or len(key) != len(sh) or not sh:
            return None
        off = self._o
        for ax in range(len(sh)):
            it = key[ax]
            if type(it) is not int:
                return None
            n = sh[ax]
            i = it + n if it < 0 else it
            if i < 0 or i >= n:
                raise IndexError("index %d is out of bounds for axis %d with size %d" % (it, ax, n))
            off += i * self._st[ax]
        return off

    def _basic(self, key):
        """(offset, shape, strides) of a basic index (ints, slices, Ellipsis), or None when the key is a mask or an index array."""
        items = key if type(key) is tuple else (key,)
        for it in items:
            if it is None:
                raise _ni("newaxis (None) indexing on a result array")
            if isinstance(it, bool) or not (isinstance(it, (int, slice)) or it is Ellipsis):
                if type(key) is tuple:
                    raise _ni("indexing with an array or a boolean inside a tuple on a result array")
                return None
        nd = len(self._sh)
        ne = 0
        ell = False
        for it in items:
            if it is Ellipsis:
                if ell:
                    raise IndexError("an index can only have a single ellipsis ('...')")
                ell = True
            else:
                ne += 1
        if ne > nd:
            raise IndexError("too many indices for array: array is %d-dimensional, but %d were indexed" % (nd, ne))
        full = []
        for it in items:
            if it is Ellipsis:
                full.extend([None] * (nd - ne))
            else:
                full.append(it)
        while len(full) < nd:
            full.append(None)
        off = self._o
        shape = []
        strides = []
        for ax in range(nd):
            it = full[ax]
            n = self._sh[ax]
            s = self._st[ax]
            if it is None or isinstance(it, slice):
                start, stop, step, ln = (0, n, 1, n) if it is None else _slice(it, n)
                if ln > 0:
                    off += start * s
                shape.append(ln)
                strides.append(s * step)
            else:
                i = it + n if it < 0 else it
                if i < 0 or i >= n:
                    raise IndexError("index %d is out of bounds for axis %d with size %d" % (it, ax, n))
                off += i * s
        return off, tuple(shape), tuple(strides)

    def _index_array(self, key):
        """A mask or integer index (list or ndarray) as an ndarray."""
        k = asarray(key)
        if k._k == "f":
            raise IndexError("arrays used as indices must be of integer (or boolean) type")
        return k

    def __getitem__(self, key):
        if type(key) is int and self._sh:
            # the common cases (a[i], m[i] for a row) without the general machinery
            sh = self._sh
            n = sh[0]
            i = key + n if key < 0 else key
            if i < 0 or i >= n:
                raise IndexError("index %d is out of bounds for axis 0 with size %d" % (key, n))
            if len(sh) == 1:
                return self._d[self._o + i * self._st[0]]
            return ndarray(self._d, sh[1:], self._k, self._o + i * self._st[0], self._st[1:])
        off = self._item_offset(key)
        if off is not None:
            return self._d[off]
        b = self._basic(key)
        if b is not None:
            off, shape, st = b
            if not shape:
                return self._d[off]
            return ndarray(self._d, shape, self._k, off, st)
        k = self._index_array(key)
        if k._k == "b":
            return self._mask_get(k)
        if len(k._sh) != 1:
            raise _ni("indexing with a multi-dimensional index array on a result array")
        n = self._sh[0] if self._sh else 0
        rest = self._sh[1:]
        out = []
        for i in k._vals():
            j = i + n if i < 0 else i
            if j < 0 or j >= n:
                raise IndexError("index %d is out of bounds for axis 0 with size %d" % (i, n))
            out.extend(ndarray(self._d, rest, self._k, self._o + j * self._st[0], self._st[1:])._vals())
        return ndarray(_store(out, self._k), (k._sh[0],) + rest, self._k)

    def _mask_get(self, m):
        if m._sh == self._sh:
            out = [v for v, t in zip(self._vals(), m._vals()) if t]
            return ndarray(_store(out, self._k), (len(out),), self._k)
        if len(m._sh) == 1 and len(self._sh) > 1 and m._sh[0] == self._sh[0]:
            out = []
            rows = 0
            for i, t in enumerate(m._vals()):
                if t:
                    rows += 1
                    out.extend(ndarray(self._d, self._sh[1:], self._k, self._o + i * self._st[0], self._st[1:])._vals())
            return ndarray(_store(out, self._k), (rows,) + self._sh[1:], self._k)
        raise IndexError("boolean index did not match indexed array: array is %s but the boolean index is %s" % (_shs(self._sh), _shs(m._sh)))

    def __setitem__(self, key, value):
        if type(key) is int and len(self._sh) == 1 and _skind(value) is not None:
            n = self._sh[0]
            i = key + n if key < 0 else key
            if i < 0 or i >= n:
                raise IndexError("index %d is out of bounds for axis 0 with size %d" % (key, n))
            self._d[self._o + i * self._st[0]] = _cast(self._k, value)
            return
        off = self._item_offset(key)
        if off is not None and _skind(value) is not None:
            self._d[off] = _cast(self._k, value)
            return
        b = self._basic(key)
        if b is not None:
            off, shape, st = b
            if not shape:
                if _skind(value) is None:
                    if isinstance(value, ndarray) and value.size == 1:
                        value = value._vals()[0]
                    else:
                        raise ValueError("setting an array element with a sequence.")
                self._d[off] = _cast(self._k, value)
                return
            target = ndarray(self._d, shape, self._k, off, st)
            offs = _offsets(shape, st, off)
            if _skind(value) is not None:
                v = _cast(self._k, value)
                d = self._d
                for i in offs:
                    d[i] = v
                return
            src = asarray(value)
            if _bshape(src._sh, shape) != shape:
                raise ValueError("could not broadcast input array from shape %s into shape %s" % (_shs(src._sh), _shs(shape)))
            vals = list(_bvals(src, shape))
            d = self._d
            k = self._k
            for i, v in zip(offs, vals):
                d[i] = _cast(k, v)
            return
        m = self._index_array(key)
        if m._k != "b":
            raise _ni("assignment through an index array on a result array")
        if m._sh != self._sh:
            raise _ni("boolean-mask assignment with a mask of another shape on a result array")
        offs = [o for o, t in zip(_offsets(self._sh, self._st, self._o), m._vals()) if t]
        d = self._d
        if _skind(value) is not None:
            v = _cast(self._k, value)
            for i in offs:
                d[i] = v
            return
        src = asarray(value)
        vals = list(src._vals())
        if len(vals) == 1:
            vals = vals * len(offs)
        if len(vals) != len(offs) or len(src._sh) > 1:
            raise ValueError("NumPy boolean array indexing assignment cannot assign %d input values to the %d output values where the mask is true" % (len(vals), len(offs)))
        for i, v in zip(offs, vals):
            d[i] = _cast(self._k, v)

    # ---- operators
    def _bin(self, other, op, swap=False):
        if isinstance(other, ndarray):
            o = other
        elif isinstance(other, (list, tuple)):
            o = asarray(other)
        else:
            kb = _skind(other)
            if kb is None:
                if op in _CMP:
                    raise _ni("comparison (" + op + ") of a result array with a " + type(other).__name__)
                if isinstance(other, complex):
                    raise _ni("complex arithmetic on a result array")
                return NotImplemented
            kind = _rkind(op, self._k, kb)
            if op == "**" and kind == "i" and (other < 0 if not swap else _any([v < 0 for v in self._vals()])):
                raise ValueError("Integers to negative integer powers are not allowed.")
            v = self._vals()
            vals = _ew(op, other, v, False, True) if swap else _ew(op, v, other, True, False)
            return _wrap(vals, self._sh, kind)
        kind = _rkind(op, self._k, o._k)
        a, b = (o, self) if swap else (self, o)
        shape = a._sh if a._sh == b._sh else _bshape(a._sh, b._sh)
        x = _bvals(a, shape)
        y = _bvals(b, shape)
        if op == "**" and kind == "i" and _any([v < 0 for v in y]):
            raise ValueError("Integers to negative integer powers are not allowed.")
        return _wrap(_ew(op, x, y, True, True), shape, kind)

    def __add__(self, o):
        return self._bin(o, "+")

    def __radd__(self, o):
        return self._bin(o, "+", True)

    def __sub__(self, o):
        return self._bin(o, "-")

    def __rsub__(self, o):
        return self._bin(o, "-", True)

    def __mul__(self, o):
        return self._bin(o, "*")

    def __rmul__(self, o):
        return self._bin(o, "*", True)

    def __truediv__(self, o):
        return self._bin(o, "/")

    def __rtruediv__(self, o):
        return self._bin(o, "/", True)

    def __floordiv__(self, o):
        return self._bin(o, "//")

    def __rfloordiv__(self, o):
        return self._bin(o, "//", True)

    def __mod__(self, o):
        return self._bin(o, "%")

    def __rmod__(self, o):
        return self._bin(o, "%", True)

    def __pow__(self, o):
        return self._bin(o, "**")

    def __rpow__(self, o):
        return self._bin(o, "**", True)

    def __and__(self, o):
        return self._bin(o, "&")

    def __rand__(self, o):
        return self._bin(o, "&", True)

    def __or__(self, o):
        return self._bin(o, "|")

    def __ror__(self, o):
        return self._bin(o, "|", True)

    def __xor__(self, o):
        return self._bin(o, "^")

    def __rxor__(self, o):
        return self._bin(o, "^", True)

    def __lt__(self, o):
        return self._bin(o, "<")

    def __le__(self, o):
        return self._bin(o, "<=")

    def __gt__(self, o):
        return self._bin(o, ">")

    def __ge__(self, o):
        return self._bin(o, ">=")

    def __eq__(self, o):
        return self._bin(o, "==")

    def __ne__(self, o):
        return self._bin(o, "!=")

    def __matmul__(self, o):
        return dot(self, o)

    def __rmatmul__(self, o):
        return dot(o, self)

    def __neg__(self):
        if self._k == "b":
            raise TypeError("The numpy boolean negative, the `-` operator, is not supported, use the `~` operator or the logical_not function instead.")
        return ndarray(_store([-x for x in self._vals()], self._k), self._sh, self._k)

    def __pos__(self):
        if self._k == "b":
            raise TypeError("ufunc 'positive' did not contain a loop with signature matching types <class 'numpy.dtypes.BoolDType'> -> None")
        return self.copy()

    def __abs__(self):
        return ndarray(_store([_abs(x) for x in self._vals()], self._k), self._sh, self._k)

    def __invert__(self):
        if self._k == "f":
            raise TypeError("ufunc 'invert' not supported for the input types, and the inputs could not be safely coerced to any supported types according to the casting rule ''safe''")
        if self._k == "b":
            return ndarray([not x for x in self._vals()], self._sh, "b")
        return ndarray(_store([~x for x in self._vals()], "i"), self._sh, "i")

    # ---- reductions as methods
    def sum(self, axis=None, **kw):
        return sum_(self, axis, **kw)

    def prod(self, axis=None, **kw):
        return prod_(self, axis, **kw)

    def mean(self, axis=None, **kw):
        return mean(self, axis, **kw)

    def std(self, axis=None, **kw):
        return std(self, axis, **kw)

    def var(self, axis=None, **kw):
        return var(self, axis, **kw)

    def max(self, axis=None, **kw):
        return max_(self, axis, **kw)

    def min(self, axis=None, **kw):
        return min_(self, axis, **kw)

    def argmax(self, axis=None, **kw):
        return argmax(self, axis, **kw)

    def argmin(self, axis=None, **kw):
        return argmin(self, axis, **kw)

    def any(self, axis=None, **kw):
        return any_(self, axis, **kw)

    def all(self, axis=None, **kw):
        return all_(self, axis, **kw)

    def cumsum(self, axis=None):
        return cumsum(self, axis)

    def round(self, decimals=0):
        return round_(self, decimals)

    def clip(self, a_min=None, a_max=None):
        return clip(self, a_min, a_max)

    def dot(self, other):
        return dot(self, other)


def _nest(vals, shape):
    if len(shape) == 1:
        return list(vals)
    step = _prod(shape[1:])
    return [_nest(vals[i * step:(i + 1) * step], shape[1:]) for i in range(shape[0])]


# ----------------------------------------------------------------------------------------------------------------- reductions
def _axis(axis, nd):
    if axis is None:
        return None
    if not isinstance(axis, int) or isinstance(axis, bool):
        raise _ni("a tuple of axes")
    if axis < -nd or axis >= nd:
        raise ValueError("axis %d is out of bounds for array of dimension %d" % (axis, nd))
    return axis + nd if axis < 0 else axis


def _reduce(a, axis, f, kind=None, keepdims=False, **kw):
    for k in kw:
        if kw[k] is not None:
            raise _ni("the '" + k + "' argument of a reduction")
    a = asarray(a)
    kind = kind or a._k
    ax = _axis(axis, len(a._sh))
    if ax is None:
        r = f(list(a._vals()))
        if keepdims:
            return _wrap([r], (1,) * len(a._sh), kind)
        return r
    n = a._sh[ax]
    s = a._st[ax]
    oshape = a._sh[:ax] + a._sh[ax + 1:]
    ost = a._st[:ax] + a._st[ax + 1:]
    steps = [k * s for k in range(n)]
    d = a._d
    out = [f([d[b + t] for t in steps]) for b in _offsets(oshape, ost, a._o)]
    if not oshape:
        if keepdims:
            return _wrap(out, (1,), kind)
        return out[0]
    res = _wrap(out, oshape, kind)
    if keepdims:
        res = res.reshape(a._sh[:ax] + (1,) + a._sh[ax + 1:])
    return res


def sum_(a, axis=None, dtype=None, out=None, keepdims=False, **kw):
    a = asarray(a)
    if dtype is not None:
        raise _ni("sum(dtype=...)")
    k = "f" if a._k == "f" else "i"
    conv = float if k == "f" else int
    return _reduce(a, axis, lambda v: conv(_sum(v)), k, keepdims, out=out, **kw)


def prod_(a, axis=None, dtype=None, out=None, keepdims=False, **kw):
    a = asarray(a)
    k = "f" if a._k == "f" else "i"

    def f(v):
        r = 1.0 if k == "f" else 1
        for x in v:
            r = r * x
        return r
    return _reduce(a, axis, f, k, keepdims, out=out, **kw)


def mean(a, axis=None, dtype=None, out=None, keepdims=False, **kw):
    if dtype is not None:
        raise _ni("mean(dtype=...)")
    return _reduce(a, axis, lambda v: _sum(v) / len(v) if v else _nan, "f", keepdims, out=out, **kw)


def _varf(ddof):
    def f(v):
        n = len(v)
        if n - ddof <= 0:
            return _nan
        m = _sum(v) / n
        return _sum([(x - m) ** 2 for x in v]) / (n - ddof)
    return f


def var(a, axis=None, dtype=None, out=None, ddof=0, keepdims=False, **kw):
    if dtype is not None:
        raise _ni("var(dtype=...)")
    return _reduce(a, axis, _varf(ddof), "f", keepdims, out=out, **kw)


def std(a, axis=None, dtype=None, out=None, ddof=0, keepdims=False, **kw):
    if dtype is not None:
        raise _ni("std(dtype=...)")
    vf = _varf(ddof)
    return _reduce(a, axis, lambda v: math.sqrt(vf(v)) if len(v) - ddof > 0 else _nan, "f", keepdims, out=out, **kw)


def _extreme(name, pick):
    def f(v):
        if not v:
            raise ValueError("zero-size array to reduction operation " + name + " which has no identity")
        for x in v:
            if x != x:
                return x
        return pick(v)
    return f


def max_(a, axis=None, out=None, keepdims=False, **kw):
    return _reduce(a, axis, _extreme("maximum", _max), None, keepdims, out=out, **kw)


def min_(a, axis=None, out=None, keepdims=False, **kw):
    return _reduce(a, axis, _extreme("minimum", _min), None, keepdims, out=out, **kw)


def _argf(name, better):
    def f(v):
        if not v:
            raise ValueError("attempt to get " + name + " of an empty sequence")
        best = 0
        for i in range(len(v)):
            x = v[i]
            if x != x:
                return i
            if better(x, v[best]):
                best = i
        return best
    return f


def argmax(a, axis=None, out=None, keepdims=False, **kw):
    return _reduce(a, axis, _argf("argmax", lambda x, y: x > y), "i", keepdims, out=out, **kw)


def argmin(a, axis=None, out=None, keepdims=False, **kw):
    return _reduce(a, axis, _argf("argmin", lambda x, y: x < y), "i", keepdims, out=out, **kw)


def any_(a, axis=None, out=None, keepdims=False, **kw):
    return _reduce(a, axis, lambda v: _any([x != 0 for x in v]), "b", keepdims, out=out, **kw)


def all_(a, axis=None, out=None, keepdims=False, **kw):
    return _reduce(a, axis, lambda v: _all([x != 0 for x in v]), "b", keepdims, out=out, **kw)


def count_nonzero(a, axis=None, keepdims=False):
    return _reduce(a, axis, lambda v: _sum([1 for x in v if x != 0]), "i", keepdims)


def median(a, axis=None, **kw):
    def f(v):
        if not v:
            return _nan
        for x in v:
            if x != x:
                return _nan
        s = sorted(v)
        n = len(s)
        if n % 2:
            return float(s[n // 2])
        return (s[n // 2 - 1] + s[n // 2]) / 2
    return _reduce(a, axis, f, "f", False, **kw)


def average(a, axis=None, weights=None, **kw):
    if weights is not None or kw.get("returned"):
        raise _ni("average(weights=...)")
    return mean(a, axis)


def cumsum(a, axis=None):
    a = asarray(a)
    k = "f" if a._k == "f" else "i"
    if axis is None or (len(a._sh) == 1 and axis in (0, -1)):
        out = []
        t = 0.0 if k == "f" else 0
        for x in a._vals():
            t = t + x
            out.append(t)
        return _wrap(out, (a.size,), k)
    raise _ni("cumsum(axis=...) of a multi-dimensional array")


_ENGINE_DOT_MIN = 100000   # multiply-adds from which a float product goes to the engine (about 46 by 46 times 46 by 46)


def _engine_matmul(a, b):
    """The product from the Calcpad engine when it is loaded here (a block that mentions linalg or a product loads it),
    else None and the caller multiplies in Python. Same values to rounding: both add the products in index order."""
    try:
        from numpy import linalg as la
        return la._call("matmul", a=la._pack(a), b=la._pack(b))[0]
    except Exception:
        return None


def dot(a, b):
    a = asarray(a)
    b = asarray(b)
    if not a._sh or not b._sh:
        return a * b
    if len(a._sh) > 2 or len(b._sh) > 2:
        raise _ni("dot of arrays with more than 2 dimensions")
    k = "f" if "f" in (a._k, b._k) else "i"
    ka = a._sh[-1]
    kb = b._sh[0]
    if ka != kb:
        raise ValueError("shapes %s and %s not aligned: %d (dim %d) != %d (dim 0)" % (_shs(a._sh), _shs(b._sh), ka, len(a._sh) - 1, kb))
    if k == "f" and len(a._sh) == 2 and a._sh[0] * ka * (b._sh[1] if len(b._sh) == 2 else 1) >= _ENGINE_DOT_MIN:
        r = _engine_matmul(a, b)
        if r is not None:
            return r
    rows = [list(r) for r in (a if len(a._sh) == 2 else [a])]
    cols = [list(c) for c in (b.T if len(b._sh) == 2 else [b])]
    out = [_sum([x * y for x, y in zip(r, c)]) for r in rows for c in cols]
    if len(a._sh) == 1 and len(b._sh) == 1:
        return _cast(k, out[0])
    shape = (() if len(a._sh) == 1 else (a._sh[0],)) + (() if len(b._sh) == 1 else (b._sh[1],))
    return _wrap([_cast(k, x) for x in out], shape, k)


# -------------------------------------------------------------------------------------------------------------- element-wise functions
def _unary(f, kind="f"):
    def g(x):
        if isinstance(x, (int, float)):
            return f(x)
        a = asarray(x)
        return ndarray(_store([f(v) for v in a._vals()], kind if kind else a._k), a._sh, kind if kind else a._k)
    return g


def _dom(f, lo_value=None):
    """math function with numpy's results outside the domain: nan (or -inf at a pole), inf on overflow."""
    def g(x):
        try:
            return f(x)
        except ValueError:
            if lo_value is not None and x == 0:
                return lo_value
            return _nan
        except OverflowError:
            return _inf
    return g


def _floor(x):
    if x != x or x == _inf or x == -_inf:
        return float(x)
    return float(math.floor(x))


def _ceil(x):
    if x != x or x == _inf or x == -_inf:
        return float(x)
    return float(math.ceil(x))


def _exp(x):
    return _inf if x > 709.782712893384 else math.exp(x)


def _sinh(x):
    if x > 710.4758600739439:
        return _inf
    if x < -710.4758600739439:
        return -_inf
    return math.sinh(x)


def _cosh(x):
    return _inf if _abs(x) > 710.4758600739439 else math.cosh(x)


sqrt = _unary(_dom(math.sqrt))
exp = _unary(_dom(_exp))
log = _unary(_dom(math.log, -_inf))
log10 = _unary(_dom(math.log10, -_inf))
log2 = _unary(_dom(math.log2, -_inf)) if hasattr(math, "log2") else None
sin = _unary(_dom(math.sin))
cos = _unary(_dom(math.cos))
tan = _unary(_dom(math.tan))
arcsin = _unary(_dom(math.asin))
arccos = _unary(_dom(math.acos))
arctan = _unary(_dom(math.atan))
sinh = _unary(_dom(_sinh))
cosh = _unary(_dom(_cosh))
tanh = _unary(_dom(math.tanh))
floor = _unary(_floor)
ceil = _unary(_ceil)
square = _unary(lambda v: v * v, None)


def absolute(x):
    if isinstance(x, (int, float)):
        return _abs(x)
    return abs_(asarray(x))


def abs_(x):
    if isinstance(x, (int, float)):
        return _abs(x)
    a = asarray(x)
    return a.__abs__()


def _boolf(f):
    def g(x):
        if isinstance(x, (int, float)):
            return f(x)
        a = asarray(x)
        return ndarray([f(v) for v in a._vals()], a._sh, "b")
    return g


isnan = _boolf(lambda v: v != v)
isinf = _boolf(lambda v: v == _inf or v == -_inf)
isfinite = _boolf(lambda v: v == v and v != _inf and v != -_inf)


def round_(a, decimals=0, out=None):
    a = asarray(a)
    if a._k != "f":
        if decimals >= 0:
            return a.copy()
        raise _ni("round of an integer array to negative decimals")
    f = 10.0 ** _abs(decimals)

    def r(x):
        if x != x or x == _inf or x == -_inf:
            return x
        if decimals == 0:
            return float(_round(x))
        if decimals > 0:
            return float(_round(x * f)) / f
        return float(_round(x / f)) * f
    return ndarray(_store([r(x) for x in a._vals()], "f"), a._sh, "f")


def _pair(x, y):
    a = asarray(x)
    b = asarray(y)
    shape = a._sh if a._sh == b._sh else _bshape(a._sh, b._sh)
    return a, b, shape, _bvals(a, shape), _bvals(b, shape)


def _scalar_or(vals, shape, kind):
    if not shape:
        return vals[0]
    return _wrap(vals, shape, kind)


def maximum(x, y):
    a, b, shape, p, q = _pair(x, y)
    k = a._k if _RANK[a._k] >= _RANK[b._k] else b._k
    return _scalar_or([u if u != u else (v if v != v else (u if u >= v else v)) for u, v in zip(p, q)], shape, k)


def minimum(x, y):
    a, b, shape, p, q = _pair(x, y)
    k = a._k if _RANK[a._k] >= _RANK[b._k] else b._k
    return _scalar_or([u if u != u else (v if v != v else (u if u <= v else v)) for u, v in zip(p, q)], shape, k)


def clip(a, a_min=None, a_max=None):
    r = asarray(a)
    if a_min is None and a_max is None:
        return r.copy()
    if a_min is not None:
        r = maximum(r, a_min)
    if a_max is not None:
        r = minimum(r, a_max)
    return r


def where(cond, x=None, y=None):
    if x is None and y is None:
        raise _ni("numpy.where(condition) with one argument")
    c = asarray(cond)
    a = asarray(x)
    b = asarray(y)
    shape = _bshape(_bshape(c._sh, a._sh), b._sh)
    cv = _bvals(c, shape)
    av = _bvals(a, shape)
    bv = _bvals(b, shape)
    k = a._k if _RANK[a._k] >= _RANK[b._k] else b._k
    return _scalar_or([p if t else q for t, p, q in zip(cv, av, bv)], shape, k)


def isclose(a, b, rtol=1e-05, atol=1e-08, equal_nan=False):
    x, y, shape, p, q = _pair(a, b)

    def close(u, v):
        if u != u or v != v:
            return bool(equal_nan and u != u and v != v)
        if u == v:
            return True
        if u in (_inf, -_inf) or v in (_inf, -_inf):
            return False
        return _abs(u - v) <= atol + rtol * _abs(v)
    return _scalar_or([close(u, v) for u, v in zip(p, q)], shape, "b")


def allclose(a, b, rtol=1e-05, atol=1e-08, equal_nan=False):
    r = isclose(a, b, rtol, atol, equal_nan)
    return bool(r) if not isinstance(r, ndarray) else bool(r.all())


def concatenate(arrays, axis=0):
    arrs = [asarray(a) for a in arrays]
    if not arrs:
        raise ValueError("need at least one array to concatenate")
    nd = len(arrs[0]._sh)
    if nd == 0:
        raise ValueError("zero-dimensional arrays cannot be concatenated")
    ax = _axis(axis, nd)
    for a in arrs:
        if len(a._sh) != nd:
            raise ValueError("all the input array dimensions except for the concatenation axis must match exactly")
        for i in range(nd):
            if i != ax and a._sh[i] != arrs[0]._sh[i]:
                raise ValueError("all the input array dimensions except for the concatenation axis must match exactly")
    k = "b"
    for a in arrs:
        if _RANK[a._k] > _RANK[k]:
            k = a._k
    if ax == 0:
        vals = []
        for a in arrs:
            vals.extend(a._vals())
        shape = (_sum([a._sh[0] for a in arrs]),) + arrs[0]._sh[1:]
        return _wrap([_cast(k, v) for v in vals], shape, k)
    if nd == 2 and ax == 1:
        rows = []
        for i in range(arrs[0]._sh[0]):
            for a in arrs:
                rows.extend(list(a[i]))
        return _wrap([_cast(k, v) for v in rows], (arrs[0]._sh[0], _sum([a._sh[1] for a in arrs])), k)
    raise _ni("concatenate(axis=%d) of %d-dimensional arrays" % (axis, nd))
