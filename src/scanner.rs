use crate::{
    report,
    token::{Token, TokenType},
};

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Scanner {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens
            .push(Token::new(TokenType::Eof, String::new(), None, self.line));

        self.tokens.clone()
    }

    fn scan_token(&mut self) {
        let c: u8 = self.advance();
        match c {
            b'(' => self.add_token(TokenType::LeftParen, None),
            b')' => self.add_token(TokenType::RightParen, None),
            b'{' => self.add_token(TokenType::LeftBrace, None),
            b'}' => self.add_token(TokenType::RightBrace, None),
            b',' => self.add_token(TokenType::Comma, None),
            b'.' => self.add_token(TokenType::Dot, None),
            b'-' => self.add_token(TokenType::Minus, None),
            b'+' => self.add_token(TokenType::Plus, None),
            b';' => self.add_token(TokenType::Semicolon, None),
            b'*' => self.add_token(TokenType::Star, None),
            b'!' => {
                let is_equal = self.next(b'=');
                self.add_token(
                    if is_equal {
                        TokenType::BangEqual
                    } else {
                        TokenType::Bang
                    },
                    None,
                );
            }
            b'=' => {
                let is_equal = self.next(b'=');
                self.add_token(
                    if is_equal {
                        TokenType::EqualEqual
                    } else {
                        TokenType::Equal
                    },
                    None,
                );
            }
            b'<' => {
                let is_equal = self.next(b'=');
                self.add_token(
                    if is_equal {
                        TokenType::LessEqual
                    } else {
                        TokenType::Less
                    },
                    None,
                );
            }
            b'>' => {
                let is_equal = self.next(b'=');
                self.add_token(
                    if is_equal {
                        TokenType::GreaterEqual
                    } else {
                        TokenType::Greater
                    },
                    None,
                );
            }
            b'/' => {
                let slash = self.next(b'/');
                if slash {
                    while self.peek() != b'\n' && !self.is_at_end() {
                        self.current += 1;
                    }
                } else {
                    self.add_token(TokenType::Slash, None)
                }
            }
            b'\r' | b' ' | b'\t' => {}
            b'\n' => self.line += 1,
            _ => {
                report(self.line, "Unexpected Character");
            }
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> u8 {
        let c = self.source.as_bytes()[self.current];
        self.current += 1;
        c
    }

    fn add_token(&mut self, ttype: TokenType, literal: Option<String>) {
        let lexeme = self.source[self.start..self.current].to_string();
        self.tokens
            .push(Token::new(ttype, lexeme, literal, self.line))
    }

    fn next(&mut self, expected: u8) -> bool {
        if self.is_at_end() {
            return false;
        };

        if self.source.as_bytes()[self.current] != expected {
            return false;
        }

        self.current += 1;
        true
    }

    fn peek(&self) -> u8 {
        if self.is_at_end() {
            return b'\0';
        }
        return self.source.as_bytes()[self.current];
    }
}
