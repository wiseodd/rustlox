use crate::{object::Object, token::Token};

#[derive(Debug, Clone)]
pub enum LoxError {
    ParseError,
    RuntimeError {
        message: String,
        token: Option<Token>,
    },
    Return {
        value: Object,
    },
}
