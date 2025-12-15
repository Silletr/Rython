from rython_jit import function_define  # pyright: ignore


def test_function_define():
    code = """
function_define main(a: int) -> int:
    print_int(a)
"""
    result = function_define(code)
    assert result == code
