use crate::token::Token;

#[derive(Debug)]
pub struct ParseError {}

#[derive(Debug)]
pub struct RuntimeError {
    pub message: String,
    pub token: Option<Token>,
}
