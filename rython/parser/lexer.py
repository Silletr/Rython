from ply import lex


class BasicLexer:
    tokens = ("NAME", "NUMBER", "FLOAT", "STRING", "COLON", "EQUALS")
    literals = {"=", "+", "-", "/", "*", "(", ")", ",", ";"}

    t_ignore = " \t"

    # === Commentaries ===
    @lex.TOKEN(r"\#.*")
    def t_COMMENT(self, t):
        pass

    @lex.TOKEN(r"---[\s\S]*?---")
    def t_MULTILINE_COMMENT(self, t):
        self.lexer.lineno += t.value.count("\n")
        pass

    # === Tokens ===
    @lex.TOKEN(r"[a-zA-Z_][a-zA-Z0-9_]*")
    def t_NAME(self, t):
        return t

    @lex.TOKEN(r'"[^"]*"')
    def t_STRING(self, t):
        t.value = t.value[1:-1]
        return t

    @lex.TOKEN(r"\d+\.\d*|\.\d+")
    def t_FLOAT(self, t):
        t.value = float(t.value)
        t.type = "FLOAT"
        return t

    @lex.TOKEN(r"\d+")
    def t_INT(self, t):
        t.value = int(t.value)
        t.type = "NUMBER"
        return t

    @lex.TOKEN(r":")
    def t_COLON(self, t):
        return t

    @lex.TOKEN(r"=")
    def t_EQUALS(self, t):
        return t

    # === New lines ===
    @lex.TOKEN(r"\n+")
    def t_newline(self, t):
        self.lexer.lineno += t.value.count("\n")

    # === Errors ===
    def t_error(self, t):
        print(f"Illegal char '{t.value[0]}' at line {self.lexer.lineno}")
        t.lexer.skip(1)

    def __init__(self):
        self.lexer = lex.lex(object=self)
