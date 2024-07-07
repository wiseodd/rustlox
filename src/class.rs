use core::fmt;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{error::LoxError, object::Object, token::Token};

#[derive(Clone, Debug)]
pub struct LoxClass {
    pub name: String,
    methods: HashMap<String, Object>,
}

impl LoxClass {
    pub fn new(name: String, methods: HashMap<String, Object>) -> Self {
        LoxClass { name, methods }
    }

    pub fn find_method(&self, name: String) -> Option<&Object> {
        self.methods.get(&name)
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
            return Ok(field.clone());
        } else if let Some(method) = self.class.borrow().find_method(name.lexeme.to_owned()) {
            return Ok(method.clone());
        }

        Err(LoxError::RuntimeError {
            message: format!("Undefined property '{}'.", name.lexeme.to_owned()),
            token: Some(name),
        })
    }

    pub fn set(&mut self, name: Token, value: Object) {
        self.fields.insert(name.lexeme, value);
    }
}

impl fmt::Display for LoxInstance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} instance", self.class.borrow())
    }
}
