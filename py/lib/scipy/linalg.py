"""scipy.linalg basics on the light numpy: solve, inv, det, cholesky, cho_factor, cho_solve, solve_triangular, eigh, eigvalsh, svd.

The numbers come from the Calcpad engine (see numpy/linalg.py). Options that change the algorithm but not the answer
(`assume_a`, `check_finite`, `overwrite_a`) are accepted; options that change the answer raise NotImplementedError.
"""
import _route

from numpy import _array as _a
from numpy._array import asarray, _ni
from numpy import linalg as _la
from numpy.linalg import LinAlgError, inv, det, norm, svd, pinv, eigvalsh


def _ignored(**kw):
    for k in kw:
        if kw[k] not in (None, False, True) and k not in ("overwrite_a", "overwrite_b", "check_finite"):
            raise _ni("scipy.linalg option " + k)


def solve(a, b, lower=False, overwrite_a=False, overwrite_b=False, check_finite=True, assume_a="gen", transposed=False):
    if assume_a not in ("gen", "sym", "her", "pos", "generic", "symmetric", "hermitian", "positive definite"):
        raise _ni("scipy.linalg.solve(assume_a='" + str(assume_a) + "')")
    a = asarray(a)
    if transposed:
        a = a.T
    return _la.solve(a, b)


def cholesky(a, lower=False, overwrite_a=False, check_finite=True):
    """Upper triangular U with a = U^T U by default (lower=True: L with a = L L^T). Reads the triangle that is returned."""
    return _la._call("cholesky", a=_la._pack(_la._square(a)), lower=bool(lower))[0]


def cho_factor(a, lower=False, overwrite_a=False, check_finite=True):
    return cholesky(a, lower), bool(lower)


def cho_solve(c_and_lower, b, overwrite_b=False, check_finite=True):
    c, lower = c_and_lower
    return _la._call("cho_solve", a=_la._pack(_la._square(c)), b=_la._pack(_la._rhs(b)), lower=bool(lower))[0]


def solve_triangular(a, b, trans=0, lower=False, unit_diagonal=False, overwrite_b=False, check_finite=True):
    """Solves a x = b reading only the stated triangle of a."""
    if unit_diagonal:
        raise _ni("solve_triangular(unit_diagonal=True)")
    if trans in (0, "N"):
        t = False
    elif trans in (1, 2, "T", "C"):
        t = True
    else:
        raise ValueError("trans must be 0, 1, 2, 'N', 'T' or 'C'")
    return _la._call("tri_solve", a=_la._pack(_la._square(a)), b=_la._pack(_la._rhs(b)), lower=bool(lower), trans=t)[0]


def eigh(a, lower=True, eigvals_only=False, **kw):
    _ignored(**kw)
    if eigvals_only:
        return eigvalsh(a, "L" if lower else "U")
    return _la.eigh(a, "L" if lower else "U")


def __getattr__(name):
    raise _route.missing("scipy.linalg", name)
