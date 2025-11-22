#!/usr/bin/env python3
import sys
from parser import BasicParser
import rython_jit


def run_ast(ast):
    if isinstance(ast, list):
        for stmt in ast:
            run_ast(stmt)

    elif ast.__class__.__name__ == "Program":
        return run_ast(ast.body)

    else:
        if ast.__class__.__name__ == "VarDecl":
            val = run_ast(ast.value)
            globals()[ast.name] = val
            return val

        elif ast.__class__.__name__ == "Number":
            return ast.value

        elif ast.__class__.__name__ == "BinOp":
            left = run_ast(ast.left)
            right = run_ast(ast.right)
            if ast.op == "+":
                return rython_jit.add(left, right)
            elif ast.op == "-":
                return left - right
            elif ast.op == "*":
                return rython_jit.multiply(left, right)
            elif ast.op == "/":
                return left / right

        elif ast.__class__.__name__ == "String":
            return ast.value

        elif ast.__class__.__name__ == "Var":
            return globals()[ast.name]

        elif ast.__class__.__name__ == "Call":
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
