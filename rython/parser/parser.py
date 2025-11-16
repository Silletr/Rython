from .lexer import BasicLexer
from .ast import BasicParser


def parse_code(code: str):
    lexer = BasicLexer()
    parser = BasicParser()
    return parser.parse(code)
