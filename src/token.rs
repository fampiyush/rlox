#[derive(Clone)]
pub struct Token {
    ttype: TokenType,
    lexeme: String,
    literal: Option<String>,
    line: usize,
}

impl Token {
    pub fn new(ttype: TokenType, lexeme: String, literal: Option<String>, line: usize) -> Self {
        Token {
            ttype,
            lexeme,
            literal,
            line,
        }
    }

    pub fn show(&self) -> String {
        format!(
            "{} {} {}",
            self.line,
            self.lexeme,
            self.literal.as_ref().unwrap()
        )
    }
}

#[derive(Clone)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}
