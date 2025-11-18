[![Python](https://img.shields.io/badge/python-3.11+-blue)](https://www.python.org)  
[![Rust](https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white)](https://www.rust-lang.org)

---

## Introduction

I think almost everyone has already heard about Rust and Python, right?  
So here's my idea: this repository is for a **programming language** based on **Rust** (compiler, JIT in future, all math operations) and **Python** (Python-like syntax, but a little different).  

Rython aims to combine Python's readability with Rust's speed. 🦀🐍

---

## How to Install and Use

Currently, the language **does not have a compiler yet**.  
To try it out:

```bash
# Clone repository via SSH
git clone git@github.com:Silletr/Rython.git

# Go into the folder and run a test lexer
cd Rython/
python3 rython/hand_tests/test_lexer.py
```

> Note: I'm writing this README from my phone, so file paths may slightly differ.
Check the hand_tests/ folder for working examples.


---

Example Code

Examples are in the examples/ folder:

example.py — test outputs

example.ry — example Rython file (interpreter is in progress, see rython/jit/__rust__/src/)


Example Rython code:
```rython
x: int = 5
y: int = 8
z: int = x + y * 3

print_int(z)
print_con("Sum: %1, Values: %2", z, y)
```

---

Commands / Functions

Some notable commands in Rython:

Strictly-dynamic types — Rust automatically determines the size of your variable based on the type hint, e.g. x: int

print_int() — prints integers

print_str() — prints strings

print_float() — prints floats

print_con() — prints concatenated output, e.g., string + int + float, for convenience


This is to avoid confusing outputs when mixing types and to make code more readable. 🤗


---

Roadmap / Future Plans

JIT compilation (in progress) ->

Full compiler to generate .bin_ry binaries ->

Expand standard library -> 

Add more examples, tests, and optimizations



---

How to Contribute

- Anyone is welcome!

- Python devs — tests, examples, bug reports

- Rust devs — runtime optimization, JIT

- Documentation / Design — README improvements, examples, tutorials


- Feel free to DM me or open issues/pull requests. Let's make Rython faster than Python and friendlier than Rust together! 🦀🐍
