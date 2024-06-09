use crate::{
    environment::Environment,
    interpreter::Interpreter,
    stmt::Stmt,
    token::{Literal, Token},
};
use core::fmt;
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone)]
pub enum LoxCallable {
    Native {
        arity: usize,
        body: Box<fn(&Vec<Literal>) -> Literal>,
    },
    User {
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
        closure: Rc<RefCell<Environment>>,
        is_initializer: bool,
    },
}

impl LoxCallable {
    pub fn arity(&self) -> usize {
        match self {
            LoxCallable::Native { arity, .. } => *arity,
            LoxCallable::User { params, .. } => params.len(),
        }
    }

    pub fn call(&self, interpreter: &Interpreter, arguments: Vec<Literal>) -> Literal {
        match self {
            LoxCallable::Native { body, .. } => body(&arguments),
            LoxCallable::User {
                name,
                params,
                body,
                closure,
                is_initializer,
            } => todo!(),
        }
    }
}

impl fmt::Display for LoxCallable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoxCallable::Native { .. } => write!(f, "<native fn>"),
            LoxCallable::User { name, .. } => write!(f, "<fn {}>", name.lexeme),
        }
    }
}
