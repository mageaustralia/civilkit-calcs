"""The routing error shared by the stand-in modules (numpy, matplotlib).

A stand-in implements a subset. Any other public name raises NotImplementedError naming it, so a script that needs more says
so instead of getting a different answer. Private and dunder names (`__path__`, `__file__`, `_foo`) get the ordinary
AttributeError, which import machinery and probes expect.
"""


def missing(module, name):
    """The exception a stand-in module's __getattr__ raises for `name`."""
    if name.startswith("_"):
        return AttributeError("module '" + module + "' has no attribute '" + name + "'")
    return NotImplementedError(module + "." + name + " is not available in the light Python stand-in")
