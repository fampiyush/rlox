use crate::token::{LiteralTypes, Token};
use std::hash::Hash;

#[derive(Debug, Clone)]
pub enum Expr {
    Assignment(Assignment),
    Binary(Binary),
    Grouping(Grouping),
    Literal(Literal),
    Unary(Unary),
    Variable(Variable),
    Call(Call),
    Get(Get),
    Set(Set),
    This(This),
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub uuid: usize,
    pub name: Token,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Binary {
    pub uuid: usize,
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Grouping {
    pub uuid: usize,
    pub expr: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Literal {
    pub uuid: usize,
    pub value: LiteralTypes,
}

#[derive(Debug, Clone)]
pub struct Unary {
    pub uuid: usize,
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub uuid: usize,
    pub name: Token,
}

#[derive(Debug, Clone)]
pub struct Call {
    pub uuid: usize,
    pub callee: Box<Expr>,
    pub paren: Token,
    pub arguments: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct Get {
    pub uuid: usize,
    pub object: Box<Expr>,
    pub name: Token,
}

#[derive(Debug, Clone)]
pub struct Set {
    pub uuid: usize,
    pub object: Box<Expr>,
    pub name: Token,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct This {
    pub uuid: usize,
    pub keyword: Token,
}

pub trait Visitor<T> {
    fn visit_assignment(&mut self, expr: &Assignment) -> T;
    fn visit_binary(&mut self, expr: &Binary) -> T;
    fn visit_grouping(&mut self, expr: &Grouping) -> T;
    fn visit_literal(&self, expr: &Literal) -> T;
    fn visit_unary(&mut self, expr: &Unary) -> T;
    fn visit_variable(&mut self, expr: &Variable) -> T;
    fn visit_call(&mut self, expr: &Call) -> T;
    fn visit_get(&mut self, expr: &Get) -> T;
    fn visit_set(&mut self, expr: &Set) -> T;
    fn visit_this(&mut self, expr: &This) -> T;
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
            Expr::Call(call) => visitor.visit_call(call),
            Expr::Get(get) => visitor.visit_get(get),
            Expr::Set(set) => visitor.visit_set(set),
            Expr::This(this) => visitor.visit_this(this),
        }
    }

    fn get_uid(&self) -> usize {
        match self {
            Expr::Assignment(e) => e.uuid,
            Expr::Binary(e) => e.uuid,
            Expr::Grouping(e) => e.uuid,
            Expr::Literal(e) => e.uuid,
            Expr::Unary(e) => e.uuid,
            Expr::Variable(e) => e.uuid,
            Expr::Call(e) => e.uuid,
            Expr::Get(e) => e.uuid,
            Expr::Set(e) => e.uuid,
            Expr::This(e) => e.uuid,
        }
    }
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        self.get_uid() == other.get_uid()
    }
}

impl Eq for Expr {}

impl Hash for Expr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // core::mem::discriminant(self).hash(state);
        self.get_uid().hash(state);
    }
}
