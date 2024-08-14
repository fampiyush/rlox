use crate::{
    environment::Environment,
    interpreter::{Exit, Interpreter},
    report,
    stmt::Function,
    token::{LiteralTypes, Token},
};
use std::{cell::RefCell, rc::Rc};
use std::{collections::HashMap, fmt};

pub enum Callable {
    Function(LoxFunction),
    Class(LoxClass),
    Instance(LoxInstance),
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
            Callable::Instance(ins) => Callable::Instance(ins.clone()),
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
    methods: HashMap<String, LoxFunction>,
}

#[derive(Clone)]
pub struct LoxInstance {
    class: LoxClass,
    fields: HashMap<String, LiteralTypes>,
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

impl fmt::Display for LoxFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} function", self.declaration.name.lexeme)
    }
}

impl LoxClass {
    pub fn new(name: String, methods: HashMap<String, LoxFunction>) -> Self {
        LoxClass { name, methods }
    }

    pub fn find_method(&self, name: &str) -> Option<&LoxFunction> {
        self.methods.get(name)
    }
}

impl LoxCallable for LoxClass {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &[LiteralTypes],
    ) -> Result<LiteralTypes, Exit> {
        let instance = LoxInstance::new(self.clone());
        Ok(LiteralTypes::Callable(Callable::Instance(instance)))
    }

    fn arity(&self) -> usize {
        0
    }
}

impl LoxInstance {
    pub fn new(class: LoxClass) -> Self {
        LoxInstance {
            class,
            fields: HashMap::new(),
        }
    }

    pub fn get(&mut self, name: &Token) -> Result<LiteralTypes, Exit> {
        if self.fields.contains_key(&name.lexeme) {
            Ok(self.fields.get(&name.lexeme).unwrap().clone())
        } else if let Some(method) = self.class.find_method(&name.lexeme) {
            Ok(LiteralTypes::Callable(Callable::Function(method.clone())))
        } else {
            report(name.line, &format!("Undefined property {}.", name.lexeme));
            Err(Exit::RuntimeError)
        }
    }

    pub fn set(&mut self, name: &Token, value: &LiteralTypes) {
        self.fields.insert(name.lexeme.clone(), value.clone());
    }
}

impl fmt::Display for LoxInstance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} instance", self.class.name)
    }
}
