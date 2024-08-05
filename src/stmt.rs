use crate::{expr::Expr, token::Token};

pub enum Stmt {
    Expression(Expression),
    Print(Print),
    Var(Var),
    Block(Block),
    If(If),
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

pub struct Block {
    pub statements: Vec<Stmt>,
}

pub struct If {
    pub condition: Box<Expr>,
    pub then_branch: Box<Stmt>,
    pub else_branch: Option<Box<Stmt>>,
}

pub trait Visitor<T> {
    fn visit_expression(&mut self, stmt: &Expression) -> T;
    fn visit_print(&mut self, stmt: &Print) -> T;
    fn visit_var(&mut self, stmt: &Var) -> T;
    fn visit_block(&mut self, stmt: &Block) -> T;
    fn visit_if(&mut self, stmt: &If) -> T;
}

impl Stmt {
    pub fn accept<T>(&self, visitor: &mut dyn Visitor<T>) -> T {
        match self {
            Stmt::Expression(expression) => visitor.visit_expression(expression),
            Stmt::Print(print) => visitor.visit_print(print),
            Stmt::Var(var) => visitor.visit_var(var),
            Stmt::Block(block) => visitor.visit_block(block),
            Stmt::If(stmt) => visitor.visit_if(stmt),
        }
    }
}
