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
}
