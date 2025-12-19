#!/usr/bin/env python3
try:
    from bridge import compile_native, rython_jit
except ImportError:
    from rython.jit.__rust__.bridge import compile_native, rython_jit

# A simple Rython code snippet to test the JIT bridge
rython_code = """
function main() -> int:
    return 42
"""

def main():
    print("--- Testing Rython JIT Bridge ---")
    print(f"Sending code to Rust JIT:\n{rython_code}")
    
    # This should call the 'compile_to_native' function in Rust
    result = compile_native(rython_code.strip())
    print(f"Received from Rust (IR):\n{result}")

    print("\n--- Running JIT Execution ---")
    val = rython_jit.jit_run(rython_code.strip())
    print(f"JIT Execution Result: {val}")
    
    if val == 42:
        print("SUCCESS: JIT returned expected value 42!")
    else:
        print(f"FAILURE: JIT returned {val}, expected 42")
    # --- GC Allocation Test ---
    print("\n--- Testing GC Allocation (Strings) ---")
    gc_code = """
function main() -> str:
    return "Hello GC World"
"""
    # The JIT function returns an i64, so for strings it's the pointer address
    ptr_val = rython_jit.jit_run(gc_code.strip())
    print(f"Allocated string at pointer: {hex(ptr_val)}")
    if ptr_val != 0:
        print("SUCCESS: GC allocated memory successfully!")
    else:
        print("FAILURE: GC returned null pointer")
    print("---------------------------------")


if __name__ == "__main__":
    main()

