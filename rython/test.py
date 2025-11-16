from parser import BasicParser

parser = BasicParser()

with open("../examples/example.ry", "r", encoding="utf-8") as f:
    code = f.read()

ast = parser.parse(code)
print(ast)
