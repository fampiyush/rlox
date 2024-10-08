use crate::token::TokenType;
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
    Instance(Rc<RefCell<LoxInstance>>),
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
    pub is_initializer: bool,
}

#[derive(Clone)]
pub struct LoxClass {
    pub name: String,
    pub super_class: Option<Box<LoxClass>>,
    pub methods: HashMap<String, LoxFunction>,
}

#[derive(Clone)]
pub struct LoxInstance {
    pub class: Rc<LoxClass>,
    pub fields: HashMap<String, LiteralTypes>,
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
    pub fn new(
        declaration: Function,
        closure: Rc<RefCell<Environment>>,
        is_initializer: bool,
    ) -> Self {
        LoxFunction {
            declaration: Box::new(declaration),
            closure,
            is_initializer,
        }
    }

    pub fn bind(&self, instance: Rc<RefCell<LoxInstance>>) -> LoxFunction {
        let environment = Rc::new(RefCell::new(Environment::new_with_enclosing(Rc::clone(
            &self.closure,
        ))));
        environment.borrow_mut().define(
            "this".to_string(),
            LiteralTypes::Callable(Callable::Instance(instance)),
        );
        LoxFunction {
            declaration: self.declaration.clone(),
            closure: environment,
            is_initializer: self.is_initializer,
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
        if self.is_initializer {
            return self.closure.borrow().get_at(
                0,
                Token {
                    ttype: TokenType::This,
                    lexeme: "this".to_string(),
                    literal: LiteralTypes::Nil,
                    line: self.declaration.name.line,
                },
            );
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
    pub fn new(
        name: String,
        super_class: Option<LoxClass>,
        methods: HashMap<String, LoxFunction>,
    ) -> Self {
        LoxClass {
            name,
            super_class: super_class.map(Box::new),
            methods,
        }
    }

    pub fn find_method(&self, name: &str) -> Option<&LoxFunction> {
        let f = self.methods.get(name);
        if f.is_none() {
            if let Some(sc) = &self.super_class {
                return sc.find_method(name);
            }
        }

        f
    }
}

impl LoxCallable for LoxClass {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &[LiteralTypes],
    ) -> Result<LiteralTypes, Exit> {
        let instance = Rc::new(RefCell::new(LoxInstance::new(Rc::new(self.clone()))));

        let initializer = self.find_method("init");
        if let Some(init) = initializer {
            init.bind(Rc::clone(&instance))
                .call(interpreter, arguments)?;
        }

        Ok(LiteralTypes::Callable(Callable::Instance(Rc::clone(
            &instance,
        ))))
    }

    fn arity(&self) -> usize {
        let initializer = self.find_method("init");
        if let Some(init) = initializer {
            init.arity()
        } else {
            0
        }
    }
}

impl LoxInstance {
    pub fn new(class: Rc<LoxClass>) -> Self {
        LoxInstance {
            class,
            fields: HashMap::new(),
        }
    }

    pub fn get(&mut self, name: &Token) -> Result<LiteralTypes, Exit> {
        if self.fields.contains_key(&name.lexeme) {
            Ok(self.fields.get(&name.lexeme).unwrap().clone())
        } else if let Some(method) = self.class.find_method(&name.lexeme) {
            Ok(LiteralTypes::Callable(Callable::Function(
                method.bind(Rc::new(RefCell::new(self.to_owned()))),
            )))
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
