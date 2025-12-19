# JIT Folder

This folder contains the **compiler**
This document explains what to modify, and if necessary, in the jit folder.

## 1 ryc.py

The **Rython Compiler (ryc)** is the Ahead-of-Time (AOT) driver. It transforms `.ry` source files into standalone executable binaries.

### How it works:
1. **Frontend**: Reads the Rython code and uses the Rust extension (`rython_jit`) to generate a standard LLVM Object File (`.o`).
2. **Linking**: Automatically invokes a system linker (`clang` or `gcc`) to combine the object file with:
   - **Boehm GC**: For automatic memory management.
   - **Python Runtime**: Since the bridge is built on PyO3.
   - **Rython Runtime**: The core C-compatible helpers (`rython_malloc`, `rython_print_str`).
3. **Binary**: Produces a final, high-performance executable.

## #2 bridge.py
Acts as the interface between the Python ecosystem and the Rust-based LLVM backend. It handles library discovery and prefix issues (`librython_jit.so` vs `rython_jit`).

## #3 jit_test_runner.py
A validation tool used to verify that both the JIT engine and the Garbage Collector are functioning correctly within the same environment.

## #4 hello.ry
A sample script demonstrating "normal" Rython syntax that can be compiled directly to a binary.

```python
function main() -> int:
    rython_print_str("Hello World!")
    return 0
```
----------------------------------------------

Credit to **Emanuel71836**