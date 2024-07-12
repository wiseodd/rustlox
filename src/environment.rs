use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::{error::LoxError, object::Object, token::Token};

type OptPointer<T> = Option<Rc<RefCell<T>>>;

#[derive(Debug, Default, Clone)]
pub struct Environment {
    pub enclosing: OptPointer<Environment>,
    values: HashMap<String, Object>,
}

impl Environment {
    pub fn new(enclosing: OptPointer<Environment>) -> Self {
        Environment {
            enclosing,
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Object) {
        self.values.insert(name, value);
    }

    pub fn get(&self, var_name: &Token) -> Result<Object, LoxError> {
        match self.values.get(&var_name.lexeme) {
            Some(val) => Ok(val.to_owned()),
            None => {
                if let Some(env) = &self.enclosing {
                    return env.borrow_mut().get(var_name);
                }

                Err(LoxError::RuntimeError {
                    message: format!("Undefined variable '{}'.", var_name.lexeme),
                    token: Some(var_name.to_owned()),
                })
            }
        }
    }

    pub fn assign(&mut self, var_name: &Token, value: Object) -> Result<(), LoxError> {
        match self.values.contains_key(&var_name.lexeme) {
            true => {
                self.values.insert(var_name.lexeme.to_owned(), value);
                Ok(())
            }
            false => {
                if let Some(env) = &self.enclosing {
                    let _ = env.borrow_mut().assign(var_name, value)?;
                    return Ok(());
                }

                Err(LoxError::RuntimeError {
                    message: format!("Undefined variable '{}'", var_name.lexeme),
                    token: Some(var_name.to_owned()),
                })
            }
        }
    }
}

pub fn get_at(
    environment: Rc<RefCell<Environment>>,
    distance: usize,
    name: String,
) -> Result<Object, LoxError> {
    if let Some(val) = ancestor(environment, distance)
        .borrow_mut()
        .values
        .get(&name)
    {
        return Ok(val.clone());
    }

    Ok(Object::None)
}

pub fn assign_at(
    environment: Rc<RefCell<Environment>>,
    distance: usize,
    name: Token,
    value: Object,
) -> Result<(), LoxError> {
    ancestor(environment.clone(), distance)
        .borrow_mut()
        .values
        .insert(name.lexeme, value);

    Ok(())
}

fn ancestor(environment: Rc<RefCell<Environment>>, distance: usize) -> Rc<RefCell<Environment>> {
    let mut env = Some(environment.clone());

    for _ in 0..distance {
        env = env.unwrap().borrow_mut().enclosing.clone();
    }

    env.unwrap()
}
