"""A thin plotly.graph_objects for a worksheet's Python block (see graph_objects.py).

Figures are sent to the page as JSON and drawn there by Plotly.js. Importing a name that is not provided raises
NotImplementedError naming it.
"""
import _route


def __getattr__(name):
    raise _route.missing("plotly", name)
