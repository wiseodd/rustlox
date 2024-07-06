use crate::{
    error::LoxError, expr::Expr, interpreter::Interpreter, lox::Lox, stmt::Stmt, token::Token,
};
use std::collections::HashMap;

// #[derive(Debug, Default, Clone)]
pub struct Resolver {
    interpreter: Interpreter,
    // The value of scopes (bool) indicates whether we have finished resolving the key
    scopes: Vec<HashMap<String, bool>>,
}

impl Resolver {
    pub fn new(interpreter: Interpreter) -> Self {
        Resolver {
            interpreter,
            scopes: vec![],
        }
    }

    pub fn resolve_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Block { statements } => {
                // Nesting behaves like stack
                // When a block is found, add the scope to the stack
                self.begin_scope();
                // Then resolve statements inside the scope
                for stmt in statements.into_iter().flatten() {
                    self.resolve_stmt(stmt);
                }
                // Exiting the scope => popping the stack
                // The immediate outer scope is now the head
                self.end_scope();
            }
            Stmt::Var { name, initializer } => {
                self.declare(name.clone());
                if let Some(init) = initializer {
                    self.resolve_expr(&init);
                }
                self.define(name.clone());
            }
            _ => unreachable!(),
        };
    }

    pub fn resolve_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Variable { name } => {
                if let Some(scope) = self.scopes.last() {
                    if let Some(resolved) = scope.get(&name.lexeme) {
                        if !resolved {
                            Lox::parse_error(
                                name,
                                "Can't read local variable in its own initializer.",
                            );
                        }
                    }
                }
                self.resolve_local(expr, name.clone());
            }
            Expr::Assign { name, value } => {
                // Recursively resolve the value of this assignment since it can
                // contain references to other variables (e.g. `var x = (a == b)`)
                self.resolve_expr(value);
                self.resolve_local(expr, name.clone());
            }
            _ => unreachable!(),
        };
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: Token) {
        if self.scopes.is_empty() {
            return;
        }

        // Put the variable name into the current scope (top of the stack)
        if let Some(scope) = self.scopes.last_mut() {
            // This is just a declaration, so the value is `false`
            // since we haven't finished resolving `name`
            scope.insert(name.lexeme, false);
        }
    }

    fn define(&mut self, name: Token) {
        if self.scopes.is_empty() {
            return;
        }

        // Mark the declared varible as resolved
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.lexeme, true);
        }
    }

    fn resolve_local(&self, expr: &Expr, name: Token) {
        // Starting from the innermost scope (top of the stack), we check for `name`.
        // Then resolve it under the correct scope.
        for i in (self.scopes.len() - 1)..0 {
            if let Some(scope) = self.scopes.get(i) {
                if scope.contains_key(&name.lexeme) {
                    // self.interpreter.resolve(expr, scopes.len() - 1 - i);
                    return;
                }
            }
        }
    }
}
