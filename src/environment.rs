use std::collections::HashMap;

use crate::{
    error::RuntimeError,
    token::{Literal, Token},
};

#[derive(Default, Clone, Debug)]
pub struct Environment {
    pub enclosing: Option<Box<Environment>>,
    values: HashMap<String, Literal>,
}

impl Environment {
    pub fn new(enclosing: Option<Environment>) -> Self {
        Environment {
            enclosing: enclosing.map(Box::new),
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Literal) {
        self.values.insert(name, value);
    }

    pub fn get(&self, var_name: &Token) -> Result<Literal, RuntimeError> {
        match self.values.get(&var_name.lexeme) {
            Some(val) => Ok(val.to_owned()),
            None => {
                if let Some(env) = self.enclosing.to_owned() {
                    return env.get(var_name);
                }

                Err(RuntimeError {
                    message: format!("Undefined variable '{}'.", var_name.lexeme),
                    token: Some(var_name.to_owned()),
                })
            }
        }
    }

    pub fn assign(&mut self, var_name: &Token, value: Literal) -> Result<(), RuntimeError> {
        match self.values.contains_key(&var_name.lexeme) {
            true => {
                self.values.insert(var_name.lexeme.to_owned(), value);
                Ok(())
            }
            false => {
                if !self.enclosing.is_none() {
                    let _ = self.enclosing.as_mut().unwrap().assign(var_name, value)?;
                    return Ok(());
                }

                Err(RuntimeError {
                    message: format!("Undefined variable '{}'", var_name.lexeme),
                    token: Some(var_name.to_owned()),
                })
            }
        }
    }
}
