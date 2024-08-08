use crate::{
    interpreter::{Interpreter, RuntimeError},
    token::LiteralTypes,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Callable {
    Function(Function),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {}

pub trait LoxCallable {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &[LiteralTypes],
    ) -> Result<LiteralTypes, RuntimeError>;
    fn arity(&self) -> usize;
}

impl LoxCallable for Function {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &[LiteralTypes],
    ) -> Result<LiteralTypes, RuntimeError> {
        todo!();
    }

    fn arity(&self) -> usize {
        0
    }
}
