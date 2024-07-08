use core::fmt;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{callable::LoxCallable, error::LoxError, object::Object, token::Token};

#[derive(Clone, Debug)]
pub struct LoxClass {
    pub name: String,
    pub methods: HashMap<String, LoxCallable>,
}

impl LoxClass {
    pub fn new(name: String, methods: HashMap<String, LoxCallable>) -> Self {
        LoxClass { name, methods }
    }

    pub fn find_method(&self, name: &str) -> Option<LoxCallable> {
        self.methods.get(name).map(|c| c.clone())
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
    pub fn new(class: Rc<RefCell<LoxClass>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(LoxInstance {
            class,
            fields: HashMap::new(),
        }))
    }

    // Kinda ugly to require `instance_ref`, which is the same as `&self`.
    // But I see no other way.
    pub fn get(&self, name: Token, instance_ref: Rc<RefCell<Self>>) -> Result<Object, LoxError> {
        if let Some(field) = self.fields.get(&name.lexeme) {
            return Ok(field.clone());
        } else if let Some(method) = self.class.borrow().find_method(&name.lexeme) {
            return Ok(Object::Callable(
                method.bind(Object::Instance(instance_ref.clone())),
            ));
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
