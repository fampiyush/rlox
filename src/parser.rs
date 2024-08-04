use crate::{
    expr::*,
    stmt::{Block, Expression, Print, Stmt, Var},
    token::{
        LiteralTypes, Token,
        TokenType::{self, *},
    },
};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

#[derive(Debug)]
pub struct ParserError {}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParserError> {
        let mut statements: Vec<Stmt> = Vec::new();
        let mut error = false;
        while !self.is_at_end() {
            let s = self.declaration();
            match &s {
                Ok(_) => statements.push(s.unwrap()),
                Err(_) => error = true,
            }
        }

        if error {
            Err(ParserError {})
        } else {
            Ok(statements)
        }
    }

    fn declaration(&mut self) -> Result<Stmt, ParserError> {
        let res = if self.token_match(&[Var]) {
            self.var_declaration()
        } else {
            self.statement()
        };

        match &res {
            Ok(_) => res,
            Err(_) => {
                self.synchronize();
                Err(ParserError {})
            }
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt, ParserError> {
        let name = self.consume(Identifier, "Expect variable name.")?;

        let mut initializer = Expr::Literal(Literal {
            value: LiteralTypes::Nil,
        });
        if self.token_match(&[Equal]) {
            initializer = self.expression()?;
        }

        self.consume(Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Var(Var {
            name,
            initializer: Box::new(initializer),
        }))
    }

    fn statement(&mut self) -> Result<Stmt, ParserError> {
        if self.token_match(&[Print]) {
            return self.print_statement();
        } else if self.token_match(&[LeftBrace]) {
            return Ok(Stmt::Block(Block {
                statements: self.block()?,
            }));
        }

        self.expression_statement()
    }

    fn block(&mut self) -> Result<Vec<Stmt>, ParserError> {
        let mut statements: Vec<Stmt> = Vec::new();

        while !self.check(&RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(RightBrace, "Expect '}' after block.")?;
        Ok(statements)
    }

    fn print_statement(&mut self) -> Result<Stmt, ParserError> {
        let value = self.expression()?;
        self.consume(Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print(Print {
            expression: Box::new(value),
        }))
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParserError> {
        let expr = self.expression()?;
        self.consume(Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Expression(Expression {
            expression: Box::new(expr),
        }))
    }

    fn expression(&mut self) -> Result<Expr, ParserError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParserError> {
        let expr = self.equality()?;

        if self.token_match(&[Equal]) {
            let equals = self.previous();
            let value = self.assignment()?;

            if let Expr::Variable(Variable { name }) = expr {
                return Ok(Expr::Assignment(Assignment {
                    name,
                    value: Box::new(value),
                }));
            } else {
                self.error(&equals, "Invalid assignment target.");
                return Err(ParserError {});
            }
        }

        Ok(expr)
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
            Identifier => {
                self.advance();
                Ok(Expr::Variable(Variable {
                    name: self.previous(),
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
                self.advance();
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

    fn consume(&mut self, ttype: TokenType, message: &str) -> Result<Token, ParserError> {
        if !self.check(&ttype) {
            self.error(&self.previous(), message);
            return Err(ParserError {});
        }

        self.advance();
        Ok(self.previous())
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
