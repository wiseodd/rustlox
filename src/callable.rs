use crate::{
    environment::{self, Environment},
    error::LoxError,
    interpreter::Interpreter,
    object::Object,
    stmt::Stmt,
    token::Token,
};
use core::fmt;
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone)]
pub enum LoxCallable {
    Native {
        arity: usize,
        body: Box<fn(&Vec<Object>) -> Object>,
    },
    User {
        name: Token,
        params: Vec<Token>,
        body: Vec<Option<Box<Stmt>>>,
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

    pub fn call(&self, interpreter: &mut Interpreter, arguments: &Vec<Object>) -> Object {
        match self {
            LoxCallable::Native { body, .. } => body(arguments),
            LoxCallable::User {
                name: _,
                params,
                body,
                closure,
                is_initializer,
            } => {
                let env: Rc<RefCell<Environment>> =
                    Rc::new(RefCell::new(Environment::new(Some(closure.clone()))));

                for i in 0..params.len() {
                    env.borrow_mut().define(
                        params.get(i).unwrap().lexeme.clone(),
                        arguments.get(i).unwrap().clone(),
                    );
                }

                let ret = interpreter.execute_block(body, env.clone());

                let ret_val: Object = match ret {
                    Err(LoxError::Return { value }) => {
                        if *is_initializer {
                            environment::get_at(closure.clone(), 0, "this".to_owned()).unwrap()
                        } else {
                            value
                        }
                    }
                    _ => {
                        if *is_initializer {
                            environment::get_at(closure.clone(), 0, "this".to_owned()).unwrap()
                        } else {
                            Object::None
                        }
                    }
                };

                ret_val
            }
        }
    }

    pub fn bind(&self, instance: Object) -> LoxCallable {
        match self {
            LoxCallable::User {
                name,
                params,
                body,
                closure,
                is_initializer,
            } => {
                let environment = Rc::new(RefCell::new(Environment::new(Some(closure.clone()))));
                environment.borrow_mut().define("this".to_owned(), instance);
                LoxCallable::User {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                    closure: environment,
                    is_initializer: *is_initializer,
                }
            }
            LoxCallable::Native { .. } => unreachable!(),
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
