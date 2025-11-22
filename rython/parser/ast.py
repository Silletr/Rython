# parser/ast.py
from ply import yacc
from dataclasses import dataclass
from typing import List, Any
from .lexer import BasicLexer


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
class VarDecl:
    name: str
    type: str
    value: Any


@dataclass
class Program:
    body: List[Any]


# === Parser ===
class BasicParser:
    # ('NAME', 'NUMBER', 'STRING', 'COLON', 'EQUALS')
    tokens = BasicLexer.tokens

    @staticmethod
    def p_program(p):
        """program : statement_list"""
        p[0] = Program(p[1])

    @staticmethod
    def p_statement_list(p):
        """statement_list : statement
        | statement_list statement"""
        if len(p) == 2:
            p[0] = [p[1]]
        else:
            p[0] = p[1] + [p[2]]

    @staticmethod
    def p_statement_vardecl(p):
        "statement : NAME COLON NAME EQUALS expression"
        # x: int = 5
        p[0] = VarDecl(name=p[1], type=p[3], value=p[5])

    @staticmethod
    def p_statement_expr(p):
        "statement : expression"
        p[0] = p[1]

    @staticmethod
    def p_expression_binop(p):
        """expression : expression '+' expression
        | expression '-' expression
        | expression '*' expression
        | expression '/' expression"""
        p[0] = BinOp(p[1], p[2], p[3])

    @staticmethod
    def p_expression_number(p):
        "expression : NUMBER"
        p[0] = Number(p[1])

    @staticmethod
    def p_expression_string(p):
        "expression : STRING"
        p[0] = String(p[1])

    @staticmethod
    def p_expression_name(p):
        "expression : NAME"
        p[0] = Var(p[1])

    @staticmethod
    def p_expression_call(p):
        'expression : NAME "(" args ")"'
        p[0] = Call(p[1], p[3])

    @staticmethod
    def p_args(p):
        """args : expression
        | args "," expression"""
        if len(p) == 2:
            p[0] = [p[1]]
        else:
            p[0] = p[1] + [p[3]]

    @staticmethod
    def p_args_empty(p):
        "args :"
        p[0] = []

    @staticmethod
    def p_error(p):
        if p:
            print(f"Syntax error at '{p.value}' line {p.lineno}")
        else:
            print("Syntax error at EOF")

    def __init__(self):
        self.lexer = BasicLexer()
        self.parser = yacc.yacc(module=self, start="program")

    def parse(self, code: str):
        return self.parser.parse(code, lexer=self.lexer.lexer)
