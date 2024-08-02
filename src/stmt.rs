use crate::{expr::Expr, token::Token};

pub enum Stmt {
    Expression(Expression),
    Print(Print),
    Var(Var),
}

pub struct Expression {
    pub expression: Box<Expr>,
}

pub struct Print {
    pub expression: Box<Expr>,
}

pub struct Var {
    pub name: Token,
    pub initializer: Box<Expr>,
}

pub trait Visitor<T> {
    fn visit_expression(&self, expr: &Expression) -> T;
    fn visit_print(&self, expr: &Print) -> T;
    fn visit_var(&self, expr: &Var) -> T;
}

impl Stmt {
    pub fn accept<T>(&self, visitor: &dyn Visitor<T>) -> T {
        match self {
            Stmt::Expression(expression) => visitor.visit_expression(expression),
            Stmt::Print(print) => visitor.visit_print(print),
            Stmt::Var(var) => visitor.visit_var(var),
        }
    }
}
