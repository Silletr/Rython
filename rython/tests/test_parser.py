from ..parser.code_parser import BasicParser

parser = BasicParser()

code = """
print(2 + 3 * hello("world", 42))
"""

ast = parser.parse(code)
print(ast)
