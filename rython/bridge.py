import importlib
import os
import sys

try:
    import rython_jit
except ImportError:
    # Look in 'target' at the current level (since we moved everything to the root)
    lib_path_rel = os.path.join(os.path.dirname(__file__), "target", "release")
    lib_path_dbg = os.path.join(os.path.dirname(__file__), "target", "debug")
    
    if os.path.exists(lib_path_rel):
        sys.path.append(lib_path_rel)
    elif os.path.exists(lib_path_dbg):
        sys.path.append(lib_path_dbg)
        
    try:
        rython_jit = importlib.import_module("rython_jit")
    except ImportError:
        # Fallback for systems that prefix shared libraries with 'lib' (like Unix/WSL)
        for folder in [lib_path_rel, lib_path_dbg, os.path.dirname(__file__)]:
            if not folder or not os.path.exists(folder):
                continue
            for ext in [".so", ".pyd"]:
                filename = os.path.join(folder, f"librython_jit{ext}")
                if os.path.exists(filename):
                    # We can't easily 'import' a file with a prefix that doesn't match the module name
                    # without renaming. Let's try to create a symlink if it doesn't exist
                    target_name = os.path.join(folder, f"rython_jit{ext}")
                    if not os.path.exists(target_name):
                        try:
                            os.symlink(filename, target_name)
                        except:
                            pass # Might fail on Windows or if Permissions are tight
        
        # Try importing again after potential symlink
        sys.path.append(os.path.dirname(__file__))
        rython_jit = importlib.import_module("rython_jit")


def compile_native(code: str) -> str:
    return rython_jit.compile_to_native(code)


def jit_run(code: str) -> int:
    return rython_jit.jit_run(code)
