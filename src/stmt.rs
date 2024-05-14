use crate::{expr::Expr, token::Token};

#[derive(Debug, Clone)]
pub enum Stmt {
    Block {
        statements: Box<Vec<Option<Stmt>>>,
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
        initializer: Option<Expr>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
}
