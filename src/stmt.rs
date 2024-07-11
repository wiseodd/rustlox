use crate::{expr::Expr, token::Token};

#[derive(Debug, Clone)]
pub enum Stmt {
    Block {
        statements: Vec<Option<Box<Stmt>>>,
    },
    Class {
        name: Token,
        superclass: Option<Expr>,
        methods: Vec<Box<Stmt>>,
    },
    Expression {
        expression: Expr,
    },
    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<Option<Box<Stmt>>>,
    },
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        // TODO: Convert to Option<Box<Stmt>>
        else_branch: Box<Option<Stmt>>,
    },
    Print {
        expression: Expr,
    },
    Return {
        keyword: Token,
        value: Option<Expr>,
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
