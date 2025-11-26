from .ast import BasicParser


def parse_code(code: str):
    parser = BasicParser()
    return parser.parse(code)
