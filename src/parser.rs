use crate::{
    expr::*,
    stmt::{Block, Class, Expression, Function, If, Print, Return, Stmt, Var, While},
    token::{
        LiteralTypes, Token,
        TokenType::{self, *},
    },
};

static mut UUID: usize = 0;

pub fn uuid_next() -> usize {
    unsafe {
        UUID += 1;
        UUID
    }
}

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
        } else if self.token_match(&[Fun]) {
            self.function("function")
        } else if self.token_match(&[Class]) {
            self.class_declaration()
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

    fn function(&mut self, kind: &str) -> Result<Stmt, ParserError> {
        let name = self.consume(Identifier, &format!("Expect {} name.", kind))?;
        self.consume(LeftParen, &format!("Expect '(' after {} name.", kind))?;

        let mut parameters = Vec::new();

        if !self.check(&RightParen) {
            loop {
                if parameters.len() >= 255 {
                    self.error(self.peek(), "Can't have more than 255 parameters.");
                }
                parameters.push(self.consume(Identifier, "Expect parameter name.")?);
                if !self.token_match(&[Comma]) {
                    break;
                }
            }
        }
        self.consume(RightParen, "Expect ')' after parameters.")?;

        self.consume(LeftBrace, &format!("Expect '{{' before {} body.", kind))?;
        let body = self.block()?;

        Ok(Stmt::Function(Function {
            name,
            params: parameters,
            body,
        }))
    }

    fn class_declaration(&mut self) -> Result<Stmt, ParserError> {
        let name = self.consume(Identifier, "Expect class name.")?;
        self.consume(LeftBrace, "Expect '{' before class body.")?;

        let mut methods = Vec::new();
        while !self.check(&RightBrace) && !self.is_at_end() {
            methods.push(self.function("method")?);
        }

        self.consume(RightBrace, "Expect '}' after class body.")?;

        Ok(Stmt::Class(Class { name, methods }))
    }

    fn var_declaration(&mut self) -> Result<Stmt, ParserError> {
        let name = self.consume(Identifier, "Expect variable name.")?;

        let mut initializer = Expr::Literal(Literal {
            uuid: uuid_next(),
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
        } else if self.token_match(&[If]) {
            return self.if_statement();
        } else if self.token_match(&[While]) {
            return self.while_statement();
        } else if self.token_match(&[For]) {
            return self.for_statement();
        } else if self.token_match(&[Return]) {
            return self.return_statement();
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

    fn if_statement(&mut self) -> Result<Stmt, ParserError> {
        self.consume(LeftParen, "Expect '(' after if.")?;
        let condition = self.expression()?;
        self.consume(RightParen, "Expect ')' after if condition.")?;

        let then_branch = self.statement()?;
        let else_branch = if self.token_match(&[Else]) {
            Some(self.statement()?)
        } else {
            None
        };

        Ok(Stmt::If(If {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(Box::new),
        }))
    }

    fn while_statement(&mut self) -> Result<Stmt, ParserError> {
        self.consume(LeftParen, "Expect '(' after while.")?;
        let condition = self.expression()?;
        self.consume(RightParen, "Expect ')' after while condition.")?;
        let body = self.statement()?;

        Ok(Stmt::While(While {
            condition: Box::new(condition),
            body: Box::new(body),
        }))
    }

    fn for_statement(&mut self) -> Result<Stmt, ParserError> {
        self.consume(LeftParen, "Expect '(' after 'For'.")?;

        let initializer = if self.token_match(&[Semicolon]) {
            None
        } else if self.token_match(&[Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if !self.check(&Semicolon) {
            self.expression()?
        } else {
            Expr::Literal(Literal {
                uuid: uuid_next(),
                value: LiteralTypes::Bool(true),
            })
        };
        self.consume(Semicolon, "Expect ';' after loop condition.")?;

        let increment = if !self.check(&RightParen) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(RightParen, "Expect ')' after for clauses.")?;

        let mut body = self.statement()?;

        if let Some(inc) = increment {
            body = Stmt::Block(Block {
                statements: Vec::from([
                    body,
                    Stmt::Expression(Expression {
                        expression: Box::new(inc),
                    }),
                ]),
            });
        };

        body = Stmt::While(While {
            condition: Box::new(condition),
            body: Box::new(body),
        });

        if let Some(init) = initializer {
            body = Stmt::Block(Block {
                statements: Vec::from([init, body]),
            })
        };

        Ok(body)
    }

    fn return_statement(&mut self) -> Result<Stmt, ParserError> {
        let keyword = self.previous();

        let value = if !self.check(&Semicolon) {
            self.expression()?
        } else {
            Expr::Literal(Literal {
                uuid: uuid_next(),
                value: LiteralTypes::Nil,
            })
        };
        self.consume(Semicolon, "Expect ';' after return value.")?;
        Ok(Stmt::Return(Return {
            keyword,
            value: Box::new(value),
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

            if let Expr::Variable(v) = expr {
                return Ok(Expr::Assignment(Assignment {
                    uuid: uuid_next(),
                    name: v.name,
                    value: Box::new(value),
                }));
            } else if let Expr::Get(g) = expr {
                return Ok(Expr::Set(Set {
                    uuid: uuid_next(),
                    object: g.object,
                    name: g.name,
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
                uuid: uuid_next(),
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
                uuid: uuid_next(),
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
                uuid: uuid_next(),
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
                uuid: uuid_next(),
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
                uuid: uuid_next(),
                operator,
                right: Box::new(right),
            }));
        }

        self.call()
    }

    fn call(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.primary()?;

        loop {
            if self.token_match(&[LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.token_match(&[Dot]) {
                let name = self.consume(Identifier, "Expect property name after '.'")?;
                expr = Expr::Get(Get {
                    uuid: uuid_next(),
                    object: Box::new(expr),
                    name,
                });
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, ParserError> {
        let mut arguments = Vec::new();

        if !self.check(&RightParen) {
            loop {
                if arguments.len() >= 255 {
                    self.error(self.peek(), "Can't have more than 255 arguments.");
                }
                arguments.push(self.expression()?);
                if !self.token_match(&[Comma]) {
                    break;
                }
            }
        }

        let paren = self.consume(RightParen, "Expect ')' after arguments.")?;

        Ok(Expr::Call(Call {
            uuid: uuid_next(),
            callee: Box::new(callee),
            paren,
            arguments,
        }))
    }

    fn primary(&mut self) -> Result<Expr, ParserError> {
        match self.peek().ttype {
            False => {
                self.advance();
                Ok(Expr::Literal(Literal {
                    uuid: uuid_next(),
                    value: LiteralTypes::Bool(false),
                }))
            }
            True => {
                self.advance();
                Ok(Expr::Literal(Literal {
                    uuid: uuid_next(),
                    value: LiteralTypes::Bool(true),
                }))
            }
            Nil => {
                self.advance();
                Ok(Expr::Literal(Literal {
                    uuid: uuid_next(),
                    value: LiteralTypes::Nil,
                }))
            }
            Number | String => {
                self.advance();
                Ok(Expr::Literal(Literal {
                    uuid: uuid_next(),
                    value: self.previous().literal,
                }))
            }
            TokenType::This => {
                self.advance();
                Ok(Expr::This(crate::expr::This {
                    uuid: uuid_next(),
                    keyword: self.previous(),
                }))
            }
            Identifier => {
                self.advance();
                Ok(Expr::Variable(Variable {
                    uuid: uuid_next(),
                    name: self.previous(),
                }))
            }
            LeftParen => {
                self.advance();
                let expr = self.expression()?;
                self.consume(RightParen, "Expect ')' after expression.")?;
                Ok(Expr::Grouping(Grouping {
                    uuid: uuid_next(),
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
                Class | Fun | Var | For | If | While | Print | Return => return,
                _ => self.advance(),
            }
        }
    }
}
