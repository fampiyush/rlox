use crate::{
    expr::*,
    token::{
        LiteralTypes, Token,
        TokenType::{self, *},
    },
};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

pub struct ParserError {}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Expr, ParserError> {
        self.expression()
    }

    fn expression(&mut self) -> Result<Expr, ParserError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.comparison();

        while self.token_match(&[BangEqual, EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Ok(Expr::Binary(Binary {
                left: Box::new(expr?),
                operator,
                right: Box::new(right),
            }))
        }

        expr
    }

    fn comparison(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.term();

        while self.token_match(&[Greater, GreaterEqual, Less, LessEqual]) {
            let operator = self.previous();
            let right = self.term()?;
            expr = Ok(Expr::Binary(Binary {
                left: Box::new(expr?),
                operator,
                right: Box::new(right),
            }))
        }

        expr
    }

    fn term(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.factor();

        while self.token_match(&[Minus, Plus]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Ok(Expr::Binary(Binary {
                left: Box::new(expr?),
                operator,
                right: Box::new(right),
            }))
        }

        expr
    }

    fn factor(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.unary();

        while self.token_match(&[Slash, Star]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Ok(Expr::Binary(Binary {
                left: Box::new(expr?),
                operator,
                right: Box::new(right),
            }))
        }

        expr
    }

    fn unary(&mut self) -> Result<Expr, ParserError> {
        if self.token_match(&[Bang, Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(Expr::Unary(Unary {
                operator,
                right: Box::new(right),
            }));
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, ParserError> {
        match self.peek().ttype {
            False => {
                self.advance();
                Ok(Expr::Literal(Literal {
                    value: LiteralTypes::Bool(false),
                }))
            }
            True => {
                self.advance();
                Ok(Expr::Literal(Literal {
                    value: LiteralTypes::Bool(true),
                }))
            }
            Nil => {
                self.advance();
                Ok(Expr::Literal(Literal {
                    value: LiteralTypes::Nil,
                }))
            }
            Number | String => {
                self.advance();
                Ok(Expr::Literal(Literal {
                    value: self.previous().literal,
                }))
            }
            LeftParen => {
                self.advance();
                let expr = self.expression()?;
                self.consume(RightParen, "Expect ')' after expression.")?;
                Ok(Expr::Grouping(Grouping {
                    expr: Box::new(expr),
                }))
            }
            _ => {
                self.error(self.peek(), "Expect expression.");
                Err(ParserError {})
            }
        }
    }

    fn token_match(&mut self, tokens: &[TokenType]) -> bool {
        for t in tokens.iter() {
            if self.check(t) {
                self.advance();
                return true;
            }
        }

        false
    }

    fn check(&self, ttype: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        self.peek().ttype == *ttype
    }

    fn is_at_end(&self) -> bool {
        self.peek().ttype == TokenType::Eof
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.current += 1;
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn consume(&self, ttype: TokenType, message: &str) -> Result<(), ParserError> {
        if !self.check(&ttype) {
            self.error(self.peek(), message);
            return Err(ParserError {});
        }

        Ok(())
    }

    fn error(&self, token: &Token, message: &str) {
        crate::error(token.clone(), message);
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().ttype == Semicolon {
                return;
            }

            match self.peek().ttype {
                Class | Fun | Var | For | If | While | Print => (),
                Return => return,
                _ => self.advance(),
            }
        }
    }
}
