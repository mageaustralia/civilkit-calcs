"""The math module of a worksheet script: MicroPython's own math plus the functions of CPython's math it lacks
(hypot, dist, prod, fsum, isqrt, comb, perm, gcd, lcm), with CPython's answers. calcpad_py.install puts it in
sys.modules as "math", so `import math` and `from math import hypot` find it."""
from math import *
import math as _m


def hypot(*coords):
    """The Euclidean norm sqrt(sum(x*x)), scaled so that large or small values do not overflow or underflow."""
    vals = [abs(float(c)) for c in coords]
    if any(v == inf for v in vals):
        return inf
    if any(v != v for v in vals):
        return nan
    big = max(vals) if vals else 0.0
    if big == 0.0:
        return 0.0
    return big * _m.sqrt(fsum([(v / big) ** 2 for v in vals]))


def dist(p, q):
    """The Euclidean distance between two points given as sequences of coordinates of the same length."""
    p = list(p)
    q = list(q)
    if len(p) != len(q):
        raise ValueError("both points must have the same number of dimensions")
    return hypot(*[a - b for a, b in zip(p, q)])


def prod(iterable, *, start=1):
    """The product of the values, starting from start (1)."""
    r = start
    for v in iterable:
        r = r * v
    return r


def fsum(iterable):
    """An accurate floating-point sum (Shewchuk's exact partial sums, as CPython)."""
    partials = []
    special = 0.0
    for x in iterable:
        x = float(x)
        if x != x or x in (inf, -inf):
            special += x
            continue
        i = 0
        for y in partials:
            if abs(x) < abs(y):
                x, y = y, x
            hi = x + y
            lo = y - (hi - x)
            if lo:
                partials[i] = lo
                i += 1
            x = hi
        partials[i:] = [x]
    if special:
        return special
    hi = 0.0
    if partials:
        hi = partials.pop()
        lo = 0.0
        while partials:
            x = hi
            y = partials.pop()
            hi = x + y
            lo = y - (hi - x)
            if lo:
                break
        if partials and ((lo < 0 and partials[-1] < 0) or (lo > 0 and partials[-1] > 0)):
            y = lo * 2
            x = hi + y
            if y == x - hi:
                hi = x
    return hi


def isqrt(n):
    """The integer square root: the largest a with a*a <= n."""
    n = int(n)
    if n < 0:
        raise ValueError("isqrt() argument must be nonnegative")
    if n == 0:
        return 0
    x = 1 << ((n.bit_length() + 1) // 2) if hasattr(n, "bit_length") else int(_m.sqrt(n)) + 1
    while True:
        y = (x + n // x) // 2
        if y >= x:
            return x
        x = y


def gcd(*ints):
    """The greatest common divisor of the integers (0 when there are none)."""
    g = 0
    for a in ints:
        a = abs(int(a))
        while a:
            g, a = a, g % a
    return g


def lcm(*ints):
    """The least common multiple of the integers (1 when there are none)."""
    r = 1
    for a in ints:
        a = abs(int(a))
        if a == 0:
            return 0
        r = r // gcd(r, a) * a
    return r


def perm(n, k=None):
    """The number of ways to choose k of n items in order (n! / (n - k)!)."""
    n = int(n)
    k = n if k is None else int(k)
    if n < 0 or k < 0:
        raise ValueError("n and k must be non-negative integers")
    if k > n:
        return 0
    r = 1
    for i in range(n - k + 1, n + 1):
        r *= i
    return r


def comb(n, k):
    """The number of ways to choose k of n items without order."""
    n = int(n)
    k = int(k)
    if n < 0 or k < 0:
        raise ValueError("n and k must be non-negative integers")
    if k > n:
        return 0
    k = min(k, n - k)
    r = 1
    for i in range(1, k + 1):
        r = r * (n - k + i) // i
    return r
