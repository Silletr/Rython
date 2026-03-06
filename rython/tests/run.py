#!/usr/bin/env python3
import sys
from pathlib import Path
import rython

def execute_file(filepath: Path):
    code = filepath.read_text(encoding="utf-8")
    # Apenas executa e retorna o resultado
    return rython.compile_to_native(code)

if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit(1)
    
    path = Path(sys.argv[1])
    if not path.is_file():
        sys.exit(1)
    
    # Imprime apenas o resultado final
    print(execute_file(path))