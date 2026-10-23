"""plotly.graph_objs: the older spelling of plotly.graph_objects, the same module."""
from plotly.graph_objects import *
from plotly.graph_objects import __all__
import plotly.graph_objects as _go


def __getattr__(name):
    raise _go._no("plotly.graph_objs", name)
