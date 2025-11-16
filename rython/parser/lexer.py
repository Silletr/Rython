# parser/lexer.py
from ply import lex


class BasicLexer:
    tokens = ('NAME', 'NUMBER', 'STRING')  # All names we using

    # Literals
    literals = {'=', '+', '-', '/', '*', '(', ')', ',', ';'}
    ignore = ' \t'

    # === Regular expression as t-functions ===

    @lex.TOKEN(r'[a-zA-Z_][a-zA-Z0-9_]*')
    def t_NAME(self, t):
        return t

    @lex.TOKEN(r'"[^"]*"')
    def t_STRING(self, t):
        t.value = t.value[1:-1]
        return t

    @lex.TOKEN(r'\d+')
    def t_NUMBER(self, t):
        t.value = int(t.value)
        return t

    @lex.TOKEN(r'//.*')
    def t_COMMENT(self, t):
        pass

    @lex.TOKEN(r'\n+')
    def t_newline(self, t):
        self.lexer.lineno += t.value.count('\n')

    # === ERROR ===
    def t_error(self, t):
        print(f"Illegal char '{t.value[0]}' at line {self.lexer.lineno}")
        t.lexer.skip(1)

    # === INIT ===
    def __init__(self):
        self.lexer = lex.lex(object=self)
