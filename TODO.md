| Part | Status | Subtasks |
|------|--------|----------|
| **1. Python part (test.py)** | ☑ | 1.1 ☑ Parse `example.ry` to AST<br>1.2 ☑ Check types and syntax<br>1.3 ☑ Call `rython_jit.{function_name}` |
| **2. Rust part** | ☐ | 2.1 ☐ Convert AST -> Cranelift IR<br>2.2 ☐ Compile IR -> JIT<br>2.3 ☐ Return callable function |
| **3. Python execution** | ☑ | 3.1 ☑ Execute `function_name()` -> compile code |
