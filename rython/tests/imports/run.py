#!/usr/bin/env python3
import sys
from pathlib import Path
import rython

def execute_file(filepath: Path):
    return rython.compile_file_py(str(filepath))

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("usage: run_test.py <file.ry>")
        sys.exit(1)
    
    path = Path(sys.argv[1])
    if not path.is_file():
        print(f"file not found: {path}")
        sys.exit(1)
    
    print(execute_file(path))