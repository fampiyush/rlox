use crate::{
    environment::Environment,
    interpreter::{Exit, Interpreter},
    stmt::Function,
    token::LiteralTypes,
};
use core::fmt;
use std::{cell::RefCell, rc::Rc};

pub enum Callable {
    Function(LoxFunction),
    Class(LoxClass),
}

impl fmt::Debug for Callable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Callable")
    }
}

impl Clone for Callable {
    fn clone(&self) -> Self {
        match self {
            Callable::Function(lox_function) => Callable::Function(lox_function.clone()),
            Callable::Class(class) => Callable::Class(class.clone()),
        }
    }
}

impl PartialEq for Callable {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

#[derive(Clone)]
pub struct LoxFunction {
    pub declaration: Box<Function>,
    pub closure: Rc<RefCell<Environment>>,
}

#[derive(Clone)]
pub struct LoxClass {
    name: String,
}

pub trait LoxCallable {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &[LiteralTypes],
    ) -> Result<LiteralTypes, Exit>;
    fn arity(&self) -> usize;
}

impl LoxFunction {
    pub fn new(declaration: Function, closure: Rc<RefCell<Environment>>) -> Self {
        LoxFunction {
            declaration: Box::new(declaration),
            closure,
        }
    }
}

impl LoxCallable for LoxFunction {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &[LiteralTypes],
    ) -> Result<LiteralTypes, Exit> {
        let mut environment = Environment::new_with_enclosing(Rc::clone(&self.closure));
        for (param, arg) in self.declaration.params.iter().zip(arguments.iter()) {
            environment.define(param.lexeme.clone(), arg.clone())
        }

        let i = interpreter.execute_block(&self.declaration.body, environment);

        match &i {
            Ok(_) => (),
            Err(e) => {
                if let Exit::Return(r) = e {
                    return Ok(r.value.clone());
                } else {
                    return Err(Exit::RuntimeError);
                }
            }
        }
        Ok(LiteralTypes::Nil)
    }

    fn arity(&self) -> usize {
        self.declaration.params.len()
    }
}

impl LoxClass {
    pub fn new(name: String) -> Self {
        LoxClass { name }
    }
}
