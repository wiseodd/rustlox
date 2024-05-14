use std::collections::HashMap;

use crate::{
    error::RuntimeError,
    token::{Literal, Token},
};

#[derive(Default)]
pub struct Environment {
    values: HashMap<String, Literal>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Literal) {
        self.values.insert(name, value);
    }

    pub fn get(&self, var_name: &Token) -> Result<Literal, RuntimeError> {
        match self.values.get(&var_name.lexeme) {
            Some(val) => Ok(val.to_owned()),
            None => Err(RuntimeError {
                message: format!("Undefined variable '{}'.", var_name.lexeme),
                token: Some(var_name.to_owned()),
            }),
        }
    }

    pub fn assign(&mut self, var_name: &Token, value: Literal) -> Result<(), RuntimeError> {
        match self.values.contains_key(&var_name.lexeme) {
            true => {
                self.values.insert(var_name.lexeme.to_owned(), value);
                Ok(())
            }
            false => Err(RuntimeError {
                message: format!("Undefined variable '{}'", var_name.lexeme),
                token: Some(var_name.to_owned()),
            }),
        }
    }
}
