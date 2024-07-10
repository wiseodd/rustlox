use std::{cell::RefCell, rc::Rc};

use crate::{
    callable::LoxCallable,
    class::{LoxClass, LoxInstance},
};

#[derive(strum_macros::Display, Clone, Debug)]
pub enum Object {
    String(String),
    Number(f64),
    Boolean(bool),
    Callable(LoxCallable),
    Class(Rc<RefCell<LoxClass>>),
    Instance(Rc<RefCell<LoxInstance>>),
    None,
}
