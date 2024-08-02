use crate::token::{LiteralTypes, Token};
pub enum Expr {
    Binary(Binary),
    Grouping(Grouping),
    Literal(Literal),
    Unary(Unary),
    Variable(Variable),
}

pub struct Binary {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

pub struct Grouping {
    pub expr: Box<Expr>,
}

pub struct Literal {
    pub value: LiteralTypes,
}

pub struct Unary {
    pub operator: Token,
    pub right: Box<Expr>,
}

pub struct Variable {
    pub name: Token,
}

pub trait Visitor<T> {
    fn visit_binary(&self, expr: &Binary) -> T;
    fn visit_grouping(&self, expr: &Grouping) -> T;
    fn visit_literal(&self, expr: &Literal) -> T;
    fn visit_unary(&self, expr: &Unary) -> T;
    fn visit_variable(&self, expr: &Variable) -> T;
}

impl Expr {
    pub fn accept<T>(&self, visitor: &dyn Visitor<T>) -> T {
        match self {
            Expr::Binary(binary) => visitor.visit_binary(binary),
            Expr::Grouping(grouping) => visitor.visit_grouping(grouping),
            Expr::Literal(literal) => visitor.visit_literal(literal),
            Expr::Unary(unary) => visitor.visit_unary(unary),
            Expr::Variable(variable) => visitor.visit_variable(variable),
        }
    }
}
