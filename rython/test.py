#!/usr/bin/env python3
import sys
from parser.code_parser import BasicParser
import rython_jit


# —————————————— Types Dispatcher ——————————————
def run_var_decl(ast):
    val = run_ast(ast.value)
    globals()[ast.name] = val
    return val


def run_number(ast):
    return ast.value


def run_float(ast):
    return ast.value


def run_string(ast):
    return ast.value


def run_var(ast):
    return globals()[ast.name]


def run_binop(ast):
    left = run_ast(ast.left)
    right = run_ast(ast.right)
    op_map = {
        "+": rython_jit.add,
        "-": lambda left, right: left - right,
        "*": rython_jit.multiply,
        "/": lambda left, right: left / right,
    }
    func = op_map.get(ast.op)
    if func is None:
        raise ValueError(f"Unknown binary operator {ast.op}")
    return func(left, right)


def run_call(ast):
    if ast.func == "print_int":
        val = run_ast(ast.args[0])
        print(val)
        return val
    elif ast.func == "print_con":
        output = ast.args[0].value
        for i, arg in enumerate(ast.args[1:], start=1):
            val = run_ast(arg)
            output = output.replace(f"%{i}", str(val))
        print(output)
        return output
    elif ast.func == "print_str":
        output = run_ast(ast.args[0])
        print(output)
        return output

    elif ast.func in rython_jit.__all__:
        func = getattr(rython_jit, ast.func)
        args = [run_ast(arg) for arg in ast.args]
        return func(*args)

    else:
        raise ValueError(f"Unknown function {ast.func}")


DISPATCH_TABLE = {
    "VarDecl": run_var_decl,
    "Number": run_number,
    "Float": run_float,
    "BinOp": run_binop,
    "String": run_string,
    "Var": run_var,
    "Call": run_call,
    "Program": lambda ast: run_ast(ast.body),
}


def run_ast(ast):
    if isinstance(ast, list):
        for stmt in ast:
            run_ast(stmt)
    else:
        cls_name = ast.__class__.__name__
        handler = DISPATCH_TABLE.get(cls_name)
        if handler is None:
            raise TypeError(f"No handler for AST node type: {cls_name}")
        return handler(ast)


# —————————————— CLI ——————————————


def main():
    if len(sys.argv) < 2:
        print("Usage: ./rython_runner.py <file.ry>")
        return

    filename = sys.argv[1]
    with open(filename, "r", encoding="utf-8") as f:
        code = f.read()

    parser = BasicParser()
    ast = parser.parse(code)

    run_ast(ast)


if __name__ == "__main__":
    main()
