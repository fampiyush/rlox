use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::report;
use crate::{
    interpreter::Exit,
    token::{LiteralTypes, Token},
};

#[derive(Debug, Clone, Default)]
pub struct Environment {
    pub values: HashMap<String, LiteralTypes>,
    pub enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Default::default()
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

    pub fn get(&self, name: &Token) -> Result<LiteralTypes, Exit> {
        if self.values.contains_key(&name.lexeme) {
            Ok(self.values.get(&name.lexeme).unwrap().clone())
        } else if self.enclosing.is_some() {
            Ok(self.enclosing.as_ref().unwrap().borrow().get(name)?)
        } else {
            report(name.line, &format!("Undefined variable '{}'.", name.lexeme));
            Err(Exit::RuntimeError {})
        }
    }

    pub fn assign(&mut self, name: &Token, value: LiteralTypes) -> Result<(), Exit> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), value);
            Ok(())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.borrow_mut().assign(name, value)?;
            Ok(())
        } else {
            report(name.line, &format!("Undefined variable '{}'.", name.lexeme));
            Err(Exit::RuntimeError {})
        }
    }

    pub fn get_at(&self, distance: usize, name: Token) -> Result<LiteralTypes, Exit> {
        if distance == 0 {
            self.get(&name)
        } else {
            self.enclosing
                .as_ref()
                .unwrap()
                .borrow()
                .get_at(distance - 1, name)
        }
    }

    pub fn assign_at(&mut self, distance: usize, name: Token, value: LiteralTypes) {
        if distance == 0 {
            self.define(name.lexeme, value);
        } else {
            self.enclosing
                .as_ref()
                .unwrap()
                .borrow_mut()
                .assign_at(distance - 1, name, value);
        }
    }
}
