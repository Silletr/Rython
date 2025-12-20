![Python](https://img.shields.io/badge/Python-3776AB?style=flat&logo=python&logoColor=white)  
[![Rust](https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![LLVM](https://img.shields.io/badge/LLVM-15-blue?logo=llvm&logoColor=white)](https://llvm.org)

# Rython

Rython combines **Python's readability** with **Rust's performance** via a high-performance **LLVM-based compiler**.

----------------------------------------------------------------------------------------------

[![Compiler-Pipeline-with-2025-12-19-234220.png](https://i.postimg.cc/25b2KSMn/Compiler-Pipeline-with-2025-12-19-234220.png)](https://postimg.cc/Jy8NyM1h)

----------------------------------------------------------------------------------------------

## Key Features

- **AOT & JIT Compilation**: Build standalone native binaries or execute code on the fly using LLVM 15.
- **Automatic Memory Management**: Integrated **Boehm-Demers-Weiser** conservative garbage collector.
- **High Performance**: Native machine code generation for arithmetic and logic.
- **Python-like Syntax**: Clean and familiar syntax with static type hinting (`x: int = 5`).
- **Modern CLI**: Professional compiler driver (`ryc.py`) with rich visual feedback.

---

## Installation

Rython requires **LLVM 15** and **libgc** to be installed on your system (WSL/Linux recommended).

```bash
# Install dependencies
sudo apt update && sudo apt install llvm-15-dev libgc-dev build-essential clang python3-rich

# Clone repository
git clone https://github.com/Silletr/Rython.git
cd Rython/rython

# Build the compiler backend
cargo build
```

---

## Usage

### Compilation (AOT)
To compile a Rython file into a standalone native binary:

```bash
python3 ryc.py hello.ry -o meu_programa
./meu_programa
```

### JIT Execution
To run code immediately:

```bash
python3 jit_test_runner.py
```

---

## Example Code
```python
function main() -> int:
    # Variables
    x: int = 5
    y: int = 8
    z: int = x + y * 3
    
    # Native Printing (via Runtime)
    rython_print_str("Hello World! Rython is running on LLVM.")
    
    return 0
```

---

## Roadmap
- [x] LLVM IR JIT Engine
- [x] Ahead-of-Time (AOT) Binary Generation
- [x] Boehm Garbage Collector Integration
- [x] Professional CLI Driver
- [ ] Boolean logic and control flow (`if`/`else`)
- [ ] Complex data types (Lists, Dicts)
- [ ] Standard Library Expansion

---

## How to Contribute
- **Rust Devs**: Optimization, LLVM IR improvements, standard library.
- **Python Devs**: Tooling, tests, and documentation.
- **Designers**: Improving the CLI and brand identity.

Credit to **Emanuel71836** for the core architecture and development.

---
![Alt](https://repobeats.axiom.co/api/embed/4fceb7306320287505bc20e9c15a95c2b68cba99.svg "Repobeats analytics image")
