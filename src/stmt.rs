use crate::{expr::Expr, token::Token};

pub enum Stmt {
    Block {
        statements: Vec<Box<Stmt>>,
    },
    Class {
        name: Token,
        superclass: Expr,
        methods: Vec<Box<Stmt>>,
    },
    Expression {
        expression: Expr,
    },
    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<Box<Stmt>>,
    },
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Box<Stmt>,
    },
    Print {
        expression: Expr,
    },
    Return {
        keyword: Token,
        value: Expr,
    },
    Var {
        name: Token,
        initializer: Expr,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
}
