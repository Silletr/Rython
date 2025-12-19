#!/usr/bin/env python3
import os
import sys
import subprocess
import argparse
import bridge
from bridge import rython_jit

def compile_rython(input_file, output_executable):
    if not os.path.exists(input_file):
        print(f"Error: File {input_file} not found")
        sys.exit(1)

    print(f"--- Rython Compiler Driver ---")
    print(f"Input: {input_file}")
    
    with open(input_file, 'r') as f:
        code = f.read()

    # 1. Compile to Object File
    obj_file = input_file + ".o"
    print(f"Compiling Rython to object file: {obj_file}")
    try:
        rython_jit.compile_to_object(code, obj_file)
    except Exception as e:
        print(f"Compilation Error: {e}")
        sys.exit(1)

    # 2. Find the runtime library
    # We look for the absolute path of the directory containing rython_jit.__file__
    lib_dir = os.path.dirname(rython_jit.__file__)
    lib_name = "rython_jit"
    
    # 3. Choose Linker (Clang preferred, GCC as fallback)
    linker = "clang"
    if subprocess.call(["which", "clang"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL) != 0:
        if subprocess.call(["which", "gcc"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL) == 0:
            linker = "gcc"
        else:
            print("Error: No linker found (clang or gcc). Please install build-essential or clang.")
            print("Run: sudo apt update && sudo apt install build-essential")
            sys.exit(1)

    # 4. Get Python linking flags (required because rython_jit is a pyo3 module)
    try:
        py_libs = subprocess.check_output(["python3-config", "--libs", "--embed"]).decode().strip().split()
        py_ldflags = subprocess.check_output(["python3-config", "--ldflags", "--embed"]).decode().strip().split()
    except:
        # Fallback if --embed is not supported (older python)
        py_libs = subprocess.check_output(["python3-config", "--libs"]).decode().strip().split()
        py_ldflags = subprocess.check_output(["python3-config", "--ldflags"]).decode().strip().split()

    # 5. Link with Linker
    print(f"Linking with {linker} to produce binary: {output_executable}")
    
    cmd = [
        linker,
        obj_file,
        "-L" + lib_dir,
        "-l" + lib_name,
        "-lgc",
    ] + py_ldflags + py_libs + [
        "-Wl,-rpath," + lib_dir,
        "-o", output_executable
    ]
    
    try:
        subprocess.run(cmd, check=True)
        print(f"SUCCESS: Binary created at ./{output_executable}")
    except subprocess.CalledProcessError as e:
        print(f"Linking Failed: {e}")
        sys.exit(1)
    finally:
        # Cleanup temporary object file
        if os.path.exists(obj_file):
            os.remove(obj_file)

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Rython AOT Compiler")
    parser.add_argument("input", help="Input Rython (.ry) file")
    parser.add_argument("-o", "--output", default="a.out", help="Output executable name")
    
    args = parser.parse_args()
    compile_rython(args.input, args.output)
