use core::fmt;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{error::LoxError, object::Object, token::Token};

#[derive(Clone, Debug)]
pub struct LoxClass {
    pub name: String,
}

impl LoxClass {
    pub fn new(name: String) -> Self {
        LoxClass { name }
    }
}

impl fmt::Display for LoxClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Clone, Debug)]
pub struct LoxInstance {
    class: Rc<RefCell<LoxClass>>,
    fields: HashMap<String, Object>,
}

impl LoxInstance {
    pub fn new(class: Rc<RefCell<LoxClass>>) -> Self {
        LoxInstance {
            class,
            fields: HashMap::new(),
        }
    }

    pub fn get(&self, name: Token) -> Result<Object, LoxError> {
        if let Some(field) = self.fields.get(&name.lexeme) {
            Ok(field.clone())
        } else {
            Err(LoxError::RuntimeError {
                message: format!("Undefined property '{}'.", name.lexeme),
                token: Some(name),
            })
        }
    }
}

impl fmt::Display for LoxInstance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} instance", self.class.borrow())
    }
}
