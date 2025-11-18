[![Python](https://img.shields.io/badge/python-3.11+-blue)](https://www.python.org)
[![Rust](https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white)
---
## Introduction
I think already all heared about Rust, Python, yep? Then I can try to suggest you this repo - it's will be **programming language**, but based on Rust (compiler, JIT in future, all math operation), and Python (syntax from python but a little bit different)

---
## HowTo install and use
First at first - I must ping it - for now this lang doesn't have a compiler, so you need clone repository by SSH (If you have his) by: 
`git clone git@github.com:Silletr/Rython`, and then 
`cd Rython/ && python3 rython/hand_tests/test_lexer.py` (as I remember it's right file (**I writing this README in phone, so can't get access to files**))
---
## Example code 
Well, examples is already in examples/ folder (with .gitignore and etc) - example.py needs for test output we need, example.ry - file for testing output when I will code interpreter (now it's not exist, you can see interpreter progress in `rython/jit/__rust__/src/` as I remember (but not exactly sure, srry), but example code also here:
```python
x: int = 5
y: int = 8
z: int = x + y * 3
print_int(z)
print_con("Sum: %1, Values: %2", z, y)
```
---
## What about commands
Well, here all interesting - as I said I will create a language with Python-like syntax, but with some differents, and you already see it in example -
strictly-dynamic types - Rust will define your type basing only in `x: int` <- **on this part with "int"**
print_con -> print_concanetante
print_int -> print_integer
print_str -> print_string
prinr_flo -> print_float
That's will be do for evade confusing integer in ouput, when you need for example string, but you have float - just for convenience 🤗