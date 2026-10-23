"""numpy.linalg for the light numpy: dense linear algebra computed by the Calcpad engine.

Every function here sends its matrices to the same engine that evaluates Calcpad lines (`lsolve`, `inverse`, `det`,
`cholesky`, `eigenvals`, `svd`), through one synchronous call, so a Python block and a Calcpad line give the same numbers.
Matrices travel as packed 64-bit floats (hexadecimal text), which is exact and fast; decimal text took seconds for a
240 by 240 matrix. What is not here raises NotImplementedError naming the function; it never returns a different answer.
"""
import json
import math
from array import array

import _cp_host
import _route

from numpy import _array as _a
from numpy._array import asarray, _ni

_inf = float("inf")


class LinAlgError(ValueError):
    """The engine refused the matrix (singular, not positive definite, wrong shape)."""


def _pack(a):
    return {"shape": list(a._sh), "hex": bytes(array("d", a._vals())).hex()}


def _unpack(t):
    shape = t["shape"]
    vals = list(array("d", bytes.fromhex(t["hex"])))
    if not shape:
        return vals[0]
    return _a._wrap(vals, tuple(shape), "f")


def _call(op, **req):
    """One engine call. The request text is put together by hand: json.dumps of a long string is very slow in MicroPython."""
    host = getattr(_cp_host, "linalg", None)
    if host is None:
        raise NotImplementedError("numpy.linalg is not available here")
    parts = ['{"op":', json.dumps(op)]
    for k in req:
        v = req[k]
        if isinstance(v, dict):
            parts.append(',"%s":{"shape":%s,"hex":"' % (k, json.dumps(v["shape"])))
            parts.append(v["hex"])
            parts.append('"}')
        else:
            parts.append(',"%s":%s' % (k, json.dumps(v)))
    parts.append("}")
    try:
        ans = json.loads(host("".join(parts)))
    except ValueError:
        raise LinAlgError("the linear algebra engine gave an unreadable answer")
    if not ans.get("ok"):
        raise LinAlgError(str(ans.get("error", "linear algebra failed")))
    return [_unpack(t) for t in ans["out"]]


def _matrix(a):
    """a as a 2-D ndarray (booleans and ints are fine; the engine refuses NaN and inf)."""
    a = asarray(a)
    if len(a._sh) < 2:
        raise LinAlgError("%d-dimensional array given. Array must be at least two-dimensional" % len(a._sh))
    if len(a._sh) > 2:
        raise _ni("numpy.linalg of stacked matrices (" + str(len(a._sh)) + "-dimensional input)")
    return a


def _square(a):
    a = _matrix(a)
    if a._sh[0] != a._sh[1]:
        raise LinAlgError("Last 2 dimensions of the array must be square")
    return a


def _rhs(b):
    b = asarray(b)
    if len(b._sh) not in (1, 2):
        raise _ni("numpy.linalg.solve with a " + str(len(b._sh)) + "-dimensional right-hand side")
    return b


def solve(a, b):
    """Solution x of a x = b (b a vector or a matrix of columns)."""
    a = _square(a)
    b = _rhs(b)
    if b._sh[0] != a._sh[0]:
        raise ValueError("solve: Input operand 1 has a mismatch in its core dimension 0, with gufunc signature (m,m),(m,n)->(m,n) (size %d is different from %d)" % (b._sh[0], a._sh[0]))
    return _call("solve", a=_pack(a), b=_pack(b))[0]


def inv(a):
    return _call("inv", a=_pack(_square(a)))[0]


def det(a):
    return _call("det", a=_pack(_square(a)))[0]


def cholesky(a):
    """The lower triangular L with a = L L^T (the lower triangle of a is read, as numpy does)."""
    return _call("cholesky", a=_pack(_square(a)), lower=True)[0]


def _uplo(u):
    u = str(u).upper()
    if u not in ("L", "U"):
        raise ValueError("UPLO argument must be 'L' or 'U'")
    return u == "L"


def eigh(a, UPLO="L"):
    """Eigenvalues (ascending) and eigenvectors (columns) of a symmetric matrix; one triangle of a is read."""
    w, v = _call("eigh", a=_pack(_square(a)), lower=_uplo(UPLO), vectors=True)
    return w, v


def eigvalsh(a, UPLO="L"):
    return _call("eigh", a=_pack(_square(a)), lower=_uplo(UPLO), vectors=False)[0]


def _check_symmetric(a, name):
    rows = a.tolist()
    n = len(rows)
    scale = 0.0
    for r in rows:
        for x in r:
            scale = max(scale, abs(x))
    for i in range(n):
        for j in range(i):
            if abs(rows[i][j] - rows[j][i]) > 1e-12 * scale:
                raise _ni("numpy.linalg." + name + " of a non-symmetric matrix (the engine's eigen solver is symmetric only; eigh and eigvalsh read one triangle)")


def eig(a):
    """Symmetric matrices only. The eigenvalues come in ascending order (numpy's order is unspecified)."""
    a = _square(a)
    _check_symmetric(a, "eig")
    return eigh(a)


def eigvals(a):
    a = _square(a)
    _check_symmetric(a, "eigvals")
    return eigvalsh(a)


def svd(a, full_matrices=True, compute_uv=True, hermitian=False):
    a = _matrix(a)
    if compute_uv and full_matrices and a._sh[0] != a._sh[1]:
        raise _ni("svd(full_matrices=True) of a non-square matrix (pass full_matrices=False)")
    u, s, vt = _call("svd", a=_pack(a))
    if not compute_uv:
        return s
    return u, s, vt


def matrix_rank(A, tol=None, hermitian=False):
    if tol is not None:
        raise _ni("matrix_rank(tol=...)")
    A = asarray(A)
    if len(A._sh) < 2:
        return int(any([x != 0 for x in A._vals()]))
    return int(_call("rank", a=_pack(_matrix(A)))[0])


def cond(x, p=None):
    if p not in (None, 2):
        raise _ni("cond(p=" + str(p) + ")")
    return _call("cond", a=_pack(_square(x)))[0]


def pinv(a, rcond=1e-15, hermitian=False):
    """Moore-Penrose inverse from the singular value decomposition."""
    a = _matrix(a)
    u, s, vt = _call("svd", a=_pack(a))
    ul, sl, vl = u.tolist(), s.tolist(), vt.tolist()
    m, n = a._sh
    cut = rcond * max(sl) if sl else 0.0
    out = [[0.0] * m for _ in range(n)]
    for t in range(len(sl)):
        if sl[t] > cut:
            w = 1.0 / sl[t]
            for i in range(n):
                vi = vl[t][i] * w
                for j in range(m):
                    out[i][j] += vi * ul[j][t]
    return _a.from_rows(out, "f", m)


def norm(x, ord=None, axis=None, keepdims=False):
    if axis is not None or keepdims:
        raise _ni("numpy.linalg.norm(axis=/keepdims=)")
    x = asarray(x)
    v = [float(t) for t in x._vals()]
    if ord is None or ord == "fro" or (ord == 2 and len(x._sh) == 1):
        if ord == "fro" and len(x._sh) != 2:
            raise ValueError("Invalid norm order for vectors.")
        return math.sqrt(sum([t * t for t in v]))
    if len(x._sh) == 1:
        if ord == 1:
            return sum([abs(t) for t in v])
        if ord == _inf:
            return max([abs(t) for t in v])
        if ord == -_inf:
            return min([abs(t) for t in v])
        if ord == 0:
            return float(sum([1 for t in v if t != 0]))
        return sum([abs(t) ** ord for t in v]) ** (1.0 / ord)
    if len(x._sh) == 2:
        rows = x.tolist()
        if ord == 1:
            return max([sum([abs(float(r[j])) for r in rows]) for j in range(x._sh[1])])
        if ord == _inf:
            return max([sum([abs(float(t)) for t in r]) for r in rows])
        if ord == 2:
            return _call("svd", a=_pack(x))[1].tolist()[0]
    raise _ni("numpy.linalg.norm(ord=" + str(ord) + ") of a " + str(len(x._sh)) + "-dimensional array")


def __getattr__(name):
    raise _route.missing("numpy.linalg", name)
