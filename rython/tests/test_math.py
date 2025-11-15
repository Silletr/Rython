import rython_jit


def test_add():
    assert rython_jit.add(2, 3) == 5
    assert rython_jit.add(-1, 1) == 0
    assert rython_jit.add(10, -5) == 5
