# Rython CLI Commands Reference

This document provides a comprehensive list of commands and options available in the Rython `ryc` compiler driver.

## Usage Overview

The `ryc.py` script serves as the primary interface for the Rython compiler. It supports both project-based compilation (using `Charge.toml`) and single-file compilation.

```bash
python3 ryc.py <command> [options]
```

---

## Commands

### 1. `init`
Initializes a new Rython project in the current directory.
- **Action**: Creates a default `Charge.toml` and a template `main.ry`.
- **Usage**:
  ```bash
  python3 ryc.py init
  ```

### 2. `build`
Compiles a Rython project or a specific file.
- **Action**: Generates a native binary executable. If a `Charge.toml` is present, it uses the settings defined there.
- **Usage**:
  ```bash
  # Compile the current project
  python3 ryc.py build

  # Compile a specific file (overrides Charge.toml)
  python3 ryc.py build hello.ry -o my_app
  ```

### 3. `run`
Compiles and immediately executes the project or file.
- **Action**: Performs a build and then launches the resulting binary.
- **Usage**:
  ```bash
  # Run the current project
  python3 ryc.py run

  # Run a specific file
  python3 ryc.py run test.ry
  ```

---

## Options & Flags

| Flag | Long Name | Description |
| :--- | :--- | :--- |
| `-h` | `--help` | Displays the stylized help screen with command summaries. |
| `-o` | `--output` | Specifies the name of the output binary (default: `a.out`). |
| `-s` | `--save-obj` | Persists the intermediate `.o` object file (usually deleted automatically). |

---

## Performance Optimizations

### Scripting Mode (Minimalist Syntax)
Rython now supports **top-level statements**. You can write your logic directly in the `.ry` file without wrapping it in a `function main()`. The compiler automatically generates an implicit entry point for you.

**Example:**
```python
x: int = 10
print_str("Starting...")
print_int(x)
```

### Native Built-ins
The following functions are available by default without any imports:
- `print_str(s)` / `print_int(i)` / `print_float(f)`
- `add(a, b)` / `minus(a, b)` / `multiply(a, b)` / `divide(a, b)`
- `fibonacci(n)`

### RAM-Based Compilation
By default, `ryc.py` generates temporary build artifacts in RAM (`/dev/shm`) to minimize disk I/O and increase speed.
- If `--save-obj` is used, the object file is automatically moved from RAM to the project directory upon completion.

---

## Project Configuration (`Charge.toml`)

Projects are managed via a `Charge.toml` file, which follows a Cargo-like structure:

```toml
[package]
name = "my_project"
version = "0.1.0"
entry = "hello.ry"

[build]
linker = "clang"
```
