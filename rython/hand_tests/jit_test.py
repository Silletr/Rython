import rython_jit

result = rython_jit.jit_test(10, 20)
print(f"The result of 10 + 20 is {result}")

assert result == 30

print("JIT test passed!")