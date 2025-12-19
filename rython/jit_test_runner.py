#!/usr/bin/env python3
from rython.jit.__rust__.bridge import compile_native

# A simple Rython code snippet to test the JIT bridge
rython_code = """
function main() -> int:
    return 42
"""

def main():
    print("--- Testing Rython JIT Bridge ---")
    print(f"Sending code to Rust JIT:\n{rython_code}")
    
    # This should call the 'compile_to_native' function in Rust
    result = compile_native(rython_code)
    
    print(f"Received from Rust: {result}")
    print("---------------------------------")

if __name__ == "__main__":
    main()

