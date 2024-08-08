use crate::lox_callable::Callable;

#[derive(Debug, Clone)]
pub struct Token {
    pub ttype: TokenType,
    pub lexeme: String,
    pub literal: LiteralTypes,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralTypes {
    String(String),
    Number(f64),
    Bool(bool),
    Nil,
    Callable(Callable),
}

impl Token {
    pub fn new(ttype: TokenType, lexeme: String, literal: LiteralTypes, line: usize) -> Self {
        Token {
            ttype,
            lexeme,
            literal,
            line,
        }
    }

    pub fn show(&self) -> String {
        format!(
            "line:{} ttype:{:?} lexeme:{} literal:{:?}",
            self.line, self.ttype, self.lexeme, self.literal
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
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
