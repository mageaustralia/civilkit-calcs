"""The host side of a Calcpad worksheet's Python block: `export`, the sandbox guard and the runner.

The worker calls `install(raw)` once per interpreter, with the host object that offers `figure(json_text)`,
`plotly(json_text)`, `export(json_text)` and `linalg(json_text)`, then `run(source, name)` for each block. The host object is kept in a closure and the module that
carried it is removed, so a script cannot reach it by name.
"""
import builtins
import sys
import json

_DENY = ("js", "jsffi", "_cp_raw", "calcpad_py", "_thread", "ffi")
_state = {}


def _num(v, what):
    if isinstance(v, bool):
        return 1 if v else 0
    if isinstance(v, int):
        return v
    if isinstance(v, float):
        if v != v or v in (float("inf"), float("-inf")):
            raise ValueError("export: " + what + " is " + str(v) + ", which Calcpad cannot hold")
        return v
    raise TypeError("export: " + what + " is a " + type(v).__name__ + ", only numbers, lists of numbers and numeric arrays can be exported")


def _plain(v, what):
    """A number, a flat list of numbers or a list of equal-length rows, as JSON-able Python."""
    if hasattr(v, "tolist"):
        v = v.tolist()
    if isinstance(v, (list, tuple)):
        if len(v) == 0:
            raise ValueError("export: " + what + " is empty")
        first = v[0]
        if hasattr(first, "tolist"):
            first = first.tolist()
        if isinstance(first, (list, tuple)):
            rows = []
            for r in v:
                if hasattr(r, "tolist"):
                    r = r.tolist()
                if not isinstance(r, (list, tuple)) or len(r) == 0:
                    raise ValueError("export: " + what + " has rows of different shapes")
                rows.append([_num(x, what) for x in r])
            for r in rows:
                if len(r) != len(rows[0]):
                    raise ValueError("export: " + what + " has rows of different lengths")
            return rows
        return [_num(x, what) for x in v]
    return _num(v, what)


def _send(args, kw, hidden):
    items = []
    if args:
        if len(args) != 2 or not isinstance(args[0], str):
            raise TypeError('export() takes keyword arguments, or a name and a value: export("name", value)')
        items.append((args[0], args[1]))
    for k in kw:
        items.append((k, kw[k]))
    if not items:
        raise TypeError("export() needs at least one name=value")
    for name, value in items:
        rec = {"name": name, "value": _plain(value, name)}
        if hidden:
            rec["hidden"] = True
        _state["export"](json.dumps(rec))


def export(*args, **kw):
    """Send values to the worksheet as Calcpad definitions: export(M_max=12.5, x=[1, 2, 3]) or export("M_max", 12.5).

    A value is a number, a list of numbers (a Calcpad vector) or a list of equal-length lists / a 2-D array (a matrix).
    """
    _send(args, kw, False)


def export_data(*args, **kw):
    """Like export, but the definitions are not shown in the report (large arrays, data for a script or chart)."""
    _send(args, kw, True)


def install(raw):
    """Wire the host object in, install the import guard, and put `export` among the builtins."""
    fig = raw.figure
    exp = raw.export
    lin = raw.linalg
    plo = raw.plotly
    _state["export"] = lambda t: exp(t)
    _state["figure"] = lambda t: fig(t)
    sys.modules.pop("_cp_raw", None)

    class Host:
        figure = staticmethod(lambda t: fig(str(t)))
        plotly = staticmethod(lambda t: plo(str(t)))
        linalg = staticmethod(lambda t: lin(str(t)))

    sys.modules["_cp_host"] = Host()
    orig = builtins.__import__

    def guarded(name, *a, **k):
        if name.split(".")[0] in _DENY:
            raise ImportError("no module named '" + name + "' (not available in a worksheet script)")
        return orig(name, *a, **k)

    builtins.__import__ = guarded
    builtins.export = export
    builtins.export_data = export_data

    def no_input(prompt=""):
        raise EOFError("input() has no keyboard in a worksheet: write the value into the script")

    builtins.input = no_input


def run(source):
    """Run one block's source in the session namespace (the same dict for every block of a worksheet)."""
    ns = _state.get("ns")
    if ns is None:
        ns = {"__name__": "__main__"}
        _state["ns"] = ns
    exec(compile(source, "<python>", "exec"), ns)
