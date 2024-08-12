use crate::{expr::Expr, token::Token};

#[derive(Clone)]
pub enum Stmt {
    Expression(Expression),
    Print(Print),
    Var(Var),
    Block(Block),
    If(If),
    While(While),
    Function(Function),
    Return(Return),
    Class(Class),
}

#[derive(Clone)]
pub struct Expression {
    pub expression: Box<Expr>,
}

#[derive(Clone)]
pub struct Print {
    pub expression: Box<Expr>,
}

#[derive(Clone)]
pub struct Var {
    pub name: Token,
    pub initializer: Box<Expr>,
}

#[derive(Clone)]
pub struct Block {
    pub statements: Vec<Stmt>,
}

#[derive(Clone)]
pub struct If {
    pub condition: Box<Expr>,
    pub then_branch: Box<Stmt>,
    pub else_branch: Option<Box<Stmt>>,
}

#[derive(Clone)]
pub struct While {
    pub condition: Box<Expr>,
    pub body: Box<Stmt>,
}

#[derive(Clone)]
pub struct Function {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Vec<Stmt>,
}

#[derive(Clone)]
pub struct Return {
    pub keyword: Token,
    pub value: Box<Expr>,
}

#[derive(Clone)]
pub struct Class {
    pub name: Token,
    pub methods: Vec<Stmt>,
}

pub trait Visitor<T> {
    fn visit_expression(&mut self, stmt: &Expression) -> T;
    fn visit_print(&mut self, stmt: &Print) -> T;
    fn visit_var(&mut self, stmt: &Var) -> T;
    fn visit_block(&mut self, stmt: &Block) -> T;
    fn visit_if(&mut self, stmt: &If) -> T;
    fn visit_while(&mut self, stmt: &While) -> T;
    fn visit_function(&mut self, stmt: &Function) -> T;
    fn visit_return(&mut self, stmt: &Return) -> T;
    fn visit_class(&mut self, stmt: &Class) -> T;
}

impl Stmt {
    pub fn accept<T>(&self, visitor: &mut dyn Visitor<T>) -> T {
        match self {
            Stmt::Expression(expression) => visitor.visit_expression(expression),
            Stmt::Print(print) => visitor.visit_print(print),
            Stmt::Var(var) => visitor.visit_var(var),
            Stmt::Block(block) => visitor.visit_block(block),
            Stmt::If(stmt) => visitor.visit_if(stmt),
            Stmt::While(stmt) => visitor.visit_while(stmt),
            Stmt::Function(fun) => visitor.visit_function(fun),
            Stmt::Return(r) => visitor.visit_return(r),
            Stmt::Class(class) => visitor.visit_class(class),
        }
    }
}
