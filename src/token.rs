use std::{fmt, hash::Hash};

#[derive(strum_macros::Display, Eq, PartialEq, Clone, Debug, Hash)]
pub enum TokenType {
    // Single-character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    // One or two character tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    // Literals
    Identifier,
    String,
    Number,
    // Keywords
    And,
    Class,
    Else,
    False,
    Fn,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    // Etc
    Eof,
}

#[derive(strum_macros::Display, Clone, Debug, PartialEq)]
pub enum Literal {
    String(String),
    Number(f64),
    Boolean(bool),
    None,
}
impl Hash for Literal {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Literal::String(val) => val.hash(state),
            Literal::Number(val) => val.to_bits().hash(state),
            Literal::Boolean(val) => val.hash(state),
            Literal::None => 0u64.hash(state),
        }
    }
}
impl Eq for Literal {}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Literal,
    pub line: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, literal: Literal, line: usize) -> Self {
        Self {
            token_type,
            lexeme,
            literal,
            line,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Token( type: {}, lexeme: \"{}\", literal: \"{}\", line: {} )",
            self.token_type, self.lexeme, self.literal, self.line
        )
    }
}
