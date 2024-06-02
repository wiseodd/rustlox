use crate::{interpreter::Interpreter, token::Literal};

pub trait LoxCallable {
    fn arity(&self) -> usize;
    fn call(&self, interpreter: &Interpreter, arguments: Vec<Literal>) -> Literal;
}
