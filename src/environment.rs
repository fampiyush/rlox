use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::report;
use crate::{
    interpreter::RuntimeError,
    token::{LiteralTypes, Token},
};

#[derive(Clone)]
pub struct Environment {
    values: HashMap<String, LiteralTypes>,
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn new_with_enclosing(enclosing: Rc<RefCell<Environment>>) -> Self {
        Environment {
            values: HashMap::new(),
            enclosing: Some(enclosing),
        }
    }

    pub fn define(&mut self, name: String, value: LiteralTypes) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<LiteralTypes, RuntimeError> {
        if self.values.contains_key(&name.lexeme) {
            Ok(self.values.get(&name.lexeme).unwrap().clone())
        } else if self.enclosing.is_some() {
            Ok(self.enclosing.as_ref().unwrap().borrow().get(name)?)
        } else {
            report(name.line, &format!("Undefined variable '{}'.", name.lexeme));
            Err(RuntimeError {})
        }
    }

    pub fn assign(&mut self, name: &Token, value: LiteralTypes) -> Result<(), RuntimeError> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), value);
            Ok(())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.borrow_mut().assign(name, value)?;
            Ok(())
        } else {
            report(name.line, &format!("Undefined variable '{}'.", name.lexeme));
            Err(RuntimeError {})
        }
    }
}
