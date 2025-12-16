import pytest
from rython_jit import add, fibonacci, minus, multiply, divide
from time import perf_counter


def timed(func, *args):
    start = perf_counter()
    result = func(*args)
    end = perf_counter()
    print(f"\n⏱ {func.__name__}{args} = {end - start:.8f} sec")
    return result


def test_add(subtests):
    test_cases = [(2, 3, 5), (-1, 1, 0), (10, -5, 5)]
    for a, b, expected in test_cases:
        with subtests.test(a=a, b=b):
            result = timed(add, a, b)
            if result != expected:
                raise AssertionError


def test_minus(subtests):
    test_cases = [(5, 1, 4), (10, 5, 5), (100, 70, 30)]
    for a, b, expected in test_cases:
        with subtests.test(a=a, b=b):
            result = timed(minus, a, b)
            if result != expected:
                raise AssertionError


def test_multiply(subtests):
    test_cases = [(5, 5, 25), (10, 10, 100), (500, 250, 125000)]
    for a, b, expected in test_cases:
        with subtests.test(a=a, b=b):
            result = timed(multiply, a, b)
            if result != expected:
                raise AssertionError


fib_num = 150
fib_res = 6792540214324356296


@pytest.mark.parametrize("n,expected", [(23, 28657), (fib_num, fib_res), (5, 5)])
def test_fib(n, expected):
    result = timed(fibonacci, n)
    if result != expected:
        raise AssertionError


def test_divide(subtests):
    test_cases = [(10, 5, 2), (9, 5, 1.8), (-5, 4, -1.25)]
    for a, b, excepted in test_cases:
        with subtests.test(a=a, b=b):
            result = timed(divide, a, b)
            if result != excepted:
                raise AssertionError
