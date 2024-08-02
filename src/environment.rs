use std::collections::HashMap;

use crate::report;
use crate::{
    interpreter::RuntimeError,
    token::{LiteralTypes, Token},
};

pub struct Environment {
    values: HashMap<String, LiteralTypes>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: LiteralTypes) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<LiteralTypes, RuntimeError> {
        if self.values.contains_key(&name.lexeme) {
            Ok(self.values.get(&name.lexeme).unwrap().clone())
        } else {
            report(name.line, &format!("Undefined variable '{}'.", name.lexeme));
            Err(RuntimeError {})
        }
    }
}
