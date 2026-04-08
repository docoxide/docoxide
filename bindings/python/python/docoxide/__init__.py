from . import docoxide
from .docoxide import *


__doc__ = docoxide.__doc__
if hasattr(docoxide, "__all__"):
    __all__ = docoxide.__all__
