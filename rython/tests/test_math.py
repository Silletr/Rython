import rython_jit
import pytest


def test_add(subtests):
    test_cases = [(2, 3, 5), (-1, 1, 0), (10, -5, 5)]

    for a, b, expected in test_cases:
        with subtests.test(a=a, b=b):
            result = rython_jit.add(a, b)
            assert result == expected


@pytest.mark.parametrize("n,expected", [(23, 28657), (50, 12586269025), (5, 5)])
def test_fib(n, expected):
    assert rython_jit.fib(n) == expected
