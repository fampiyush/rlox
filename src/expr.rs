use crate::token::{LiteralTypes, Token};
pub enum Expr {
    Binary(Binary),
    Grouping(Grouping),
    Literal(Literal),
    Unary(Unary),
}

pub struct Binary {
    left: Box<Expr>,
    operator: Token,
    right: Box<Expr>,
}

pub struct Grouping {
    expr: Box<Expr>,
}

pub struct Literal {
    value: LiteralTypes,
}

pub struct Unary {
    operator: Token,
    right: Box<Expr>,
}

pub trait Visitor<T> {
    fn visit_binary(&self, expr: &Binary) -> T;
    fn visit_grouping(&self, expr: &Grouping) -> T;
    fn visit_literal(&self, expr: &Literal) -> T;
    fn visit_unary(&self, expr: &Unary) -> T;
}

impl Binary {
    pub fn accept<T>(&self, visitor: &dyn Visitor<T>) -> T {
        visitor.visit_binary(self)
    }
}

impl Grouping {
    pub fn accept<T>(&self, visitor: &dyn Visitor<T>) -> T {
        visitor.visit_grouping(self)
    }
}

impl Literal {
    pub fn accept<T>(&self, visitor: &dyn Visitor<T>) -> T {
        visitor.visit_literal(self)
    }
}

impl Unary {
    pub fn accept<T>(&self, visitor: &dyn Visitor<T>) -> T {
        visitor.visit_unary(self)
    }
}
