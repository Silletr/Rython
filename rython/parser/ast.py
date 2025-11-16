# parser/ast.py
from ply import yacc
from dataclasses import dataclass
from typing import List, Any
from .lexer import BasicLexer  # ← Import lexer from ./lexer.py


# === AST Nodes ===
@dataclass
class Number:
    value: int


@dataclass
class String:
    value: str


@dataclass
class Var:
    name: str


@dataclass
class BinOp:
    left: Any
    op: str
    right: Any


@dataclass
class Call:
    func: str
    args: List[Any]


@dataclass
class Program:
    body: List[Any]


# === Parser ===
class BasicParser:
    tokens = BasicLexer.tokens

    def p_statement_expr(self, p):
        'statement : expression'
        p[0] = p[1]

    def p_expression_binop(self, p):
        '''expression : expression '+' expression
                      | expression '-' expression
                      | expression '*' expression
                      | expression '/' expression'''
        p[0] = BinOp(p[1], p[2], p[3])

    def p_expression_number(self, p):
        'expression : NUMBER'
        p[0] = Number(p[1])

    def p_expression_string(self, p):
        'expression : STRING'
        p[0] = String(p[1][1:-1])  # без лапок

    def p_expression_name(self, p):
        'expression : NAME'
        p[0] = Var(p[1])

    def p_expression_call(self, p):
        'expression : NAME "(" args ")"'
        p[0] = Call(p[1], p[3])

    def p_args_list(self, p):
        '''args : expression
                | args "," expression'''
        if len(p) == 2:
            p[0] = [p[1]]
        else:
            p[0] = p[1] + [p[3]]

    def p_args_empty(self, p):
        'args :'
        p[0] = []

    def p_error(self, p):
        if p:
            print(f"Syntax error at '{p.value}' line {p.lineno}")
        else:
            print("Syntax error at EOF")

    def __init__(self):
        self.lexer = BasicLexer()
        self.parser = yacc.yacc(module=self)

    def parse(self, code: str):
        return self.parser.parse(code, lexer=self.lexer.lexer)
