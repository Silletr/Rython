import pytest
from rython_jit import add, fib, minus, multiply


def test_add(subtests):
    test_cases = [(2, 3, 5), (-1, 1, 0), (10, -5, 5)]

    for a, b, expected in test_cases:
        with subtests.test(a=a, b=b):
            result = add(a, b)
            assert result == expected


def test_minus(subtests):
    test_cases = [(5, 1, 4), (10, 5, 5), (100, 70, 30)]
    for a, b, expected in test_cases:
        with subtests.test(a=a, b=b):
            result = minus(a, b)
            assert result == expected


def test_multiply(subtests):
    test_cases = [(5, 5, 25), (10, 10, 100), (500, 250, 125000)]
    for a, b, expected in test_cases:
        with subtests.test(a=a, b=b):
            result = multiply(a, b)
            assert result == expected


@pytest.mark.parametrize("n,expected", [(23, 28657), (50, 12586269025), (5, 5)])
def test_fib(n, expected):
    assert fib(n) == expected
