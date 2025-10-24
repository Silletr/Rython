import importlib
import os
import sys

try:
    import rython_jit
except ImportError:
    lib_path = os.path.join(os.path.dirname(__file__), "__rust__", "target", "release")
    sys.path.append(lib_path)
    rython_jit = importlib.import_module("rython_jit")

def compile_native(code: str) -> str:
    return rython_jit.compile_to_native(code)

