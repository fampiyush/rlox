use crate::token::{LiteralTypes, Token};
pub enum Expr {
    Assignment(Assignment),
    Binary(Binary),
    Grouping(Grouping),
    Literal(Literal),
    Unary(Unary),
    Variable(Variable),
}

pub struct Assignment {
    pub name: Token,
    pub value: Box<Expr>,
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
    fn visit_assignment(&mut self, expr: &Assignment) -> T;
    fn visit_binary(&mut self, expr: &Binary) -> T;
    fn visit_grouping(&mut self, expr: &Grouping) -> T;
    fn visit_literal(&self, expr: &Literal) -> T;
    fn visit_unary(&mut self, expr: &Unary) -> T;
    fn visit_variable(&mut self, expr: &Variable) -> T;
}

impl Expr {
    pub fn accept<T>(&self, visitor: &mut dyn Visitor<T>) -> T {
        match self {
            Expr::Assignment(assignment) => visitor.visit_assignment(assignment),
            Expr::Binary(binary) => visitor.visit_binary(binary),
            Expr::Grouping(grouping) => visitor.visit_grouping(grouping),
            Expr::Literal(literal) => visitor.visit_literal(literal),
            Expr::Unary(unary) => visitor.visit_unary(unary),
            Expr::Variable(variable) => visitor.visit_variable(variable),
        }
    }
}
