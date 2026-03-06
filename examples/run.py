import rython

with open("example.ry") as f:
    code = f.read()

result = rython.compile_to_native(code)
print(f"\nProgram returned: {result}")