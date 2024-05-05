use crate::token::{Literal, Token};

#[derive(strum_macros::Display, Debug)]
pub enum Expr {
    Assign {
        name: Token,
        value: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Box<Expr>>,
    },
    Get {
        object: Box<Expr>,
        name: Token,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: Literal,
    },
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Set {
        object: Box<Expr>,
        name: Token,
        value: Box<Expr>,
    },
    Super {
        keyword: Token,
        method: Token,
    },
    This {
        keyword: Token,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Variable {
        name: Token,
    },
}

pub enum Stmt {
    Block(Vec<Box<Stmt>>),
    Class(Token, Expr),
    Expression(Expr),
    Function(Token, Vec<Token>, Vec<Box<Stmt>>),
    If(Expr, Box<Stmt>, Box<Stmt>),
    Print(Expr),
    Return(Token, Expr),
    Var(Token, Expr),
    While(Expr, Box<Stmt>),
}
