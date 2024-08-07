use crate::{expr::Expr, interpreter::Interpreter, lox::Lox, stmt::Stmt, token::Token};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug, Clone)]
enum FunctionType {
    None,
    Function,
    Initializer,
    Method,
}

#[derive(Debug, Clone)]
enum ClassType {
    None,
    Class,
    Subclass,
}

// #[derive(Debug, Default, Clone)]
pub struct Resolver {
    interpreter: Rc<RefCell<Interpreter>>,
    // The value of scopes (bool) indicates whether we have finished resolving the key
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
    current_class: ClassType,
}

impl Resolver {
    pub fn new(interpreter: Rc<RefCell<Interpreter>>) -> Self {
        Resolver {
            interpreter,
            scopes: vec![],
            current_function: FunctionType::None,
            current_class: ClassType::None,
        }
    }

    pub fn resolve_stmt_list(&mut self, statements: &Vec<Option<Box<Stmt>>>) {
        for stmt in statements.into_iter().flatten() {
            self.resolve_stmt(stmt);
        }
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Block { statements } => {
                // Nesting behaves like stack
                // When a block is found, add the scope to the stack
                self.begin_scope();
                // Then resolve statements inside the scope
                self.resolve_stmt_list(statements);
                // Exiting the scope => popping the stack
                // The immediate outer scope is now the head
                self.end_scope();
            }
            Stmt::Class {
                name,
                superclass,
                methods,
            } => {
                let enclosing_class: ClassType = self.current_class.clone();
                self.current_class = ClassType::Class;

                self.declare(name.clone());
                self.define(name.clone());

                if let Some(Expr::Variable {
                    name: superclass_name,
                }) = superclass
                {
                    if name.lexeme.eq(&superclass_name.lexeme) {
                        Lox::parse_error(superclass_name, "A class cannot inherit from itself.");
                    }
                }

                if !superclass.is_none() {
                    self.current_class = ClassType::Subclass;
                    self.resolve_expr(&superclass.clone().unwrap());

                    self.begin_scope();
                    self.scopes
                        .last_mut()
                        .unwrap()
                        .insert("super".to_owned(), true);
                }

                self.begin_scope();
                self.scopes
                    .last_mut()
                    .unwrap()
                    .insert("this".to_owned(), true);

                for method in methods {
                    match *method.to_owned() {
                        Stmt::Function { params, body, .. } => {
                            let declaration: FunctionType;
                            if name.lexeme.eq("init") {
                                declaration = FunctionType::Initializer;
                            } else {
                                declaration = FunctionType::Method
                            }

                            self.resolve_function(&params, &body, declaration)
                        }
                        _ => unreachable!(),
                    }
                }

                if !superclass.is_none() {
                    self.end_scope();
                }

                self.end_scope();

                self.current_class = enclosing_class;
            }
            Stmt::Var { name, initializer } => {
                self.declare(name.clone());
                if let Some(init) = initializer {
                    self.resolve_expr(&init);
                }
                self.define(name.clone());
            }
            Stmt::Function { name, params, body } => {
                self.declare(name.clone());
                self.define(name.clone());
                self.resolve_function(params, body, FunctionType::Function);
            }
            Stmt::Expression { expression } => self.resolve_expr(expression),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expr(condition);
                self.resolve_stmt(then_branch);

                if let Some(else_stmt) = else_branch.as_ref() {
                    self.resolve_stmt(else_stmt);
                }
            }
            Stmt::Print { expression } => self.resolve_expr(expression),
            Stmt::Return { value, keyword } => {
                match self.current_function {
                    FunctionType::None => {
                        Lox::parse_error(keyword, "Can't return from top-level code.")
                    }
                    _ => (),
                };

                if let Some(expr) = value {
                    match self.current_function {
                        FunctionType::Initializer => {
                            Lox::parse_error(keyword, "Can't return a value from an initializer")
                        }
                        _ => self.resolve_expr(expr),
                    }
                }
            }
            Stmt::While { condition, body } => {
                self.resolve_expr(condition);
                self.resolve_stmt(body);
            }
        };
    }

    fn resolve_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Variable { name } => {
                if !self.scopes.is_empty() {
                    if let Some(resolved) = self.scopes.last().unwrap().get(&name.lexeme) {
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
            Expr::Binary { left, right, .. } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            Expr::Call {
                callee, arguments, ..
            } => {
                self.resolve_expr(callee);

                for arg in arguments.iter() {
                    self.resolve_expr(arg);
                }
            }
            Expr::Get { object, .. } => self.resolve_expr(object),
            Expr::Set { object, value, .. } => {
                self.resolve_expr(value);
                self.resolve_expr(object);
            }
            Expr::Super { keyword, .. } => {
                if matches!(self.current_class, ClassType::None) {
                    Lox::parse_error(keyword, "Can't use 'super' outside of a class.");
                } else if !matches!(self.current_class, ClassType::Subclass) {
                    Lox::parse_error(keyword, "Can't use 'super' in a class with no superclass.");
                }

                self.resolve_local(&expr, keyword.clone())
            }
            Expr::This { keyword } => match self.current_class {
                ClassType::None => {
                    Lox::parse_error(keyword, "Can't use 'this' outside of a class.")
                }
                _ => self.resolve_local(expr, keyword.clone()),
            },
            Expr::Grouping { expression } => self.resolve_expr(expression),
            Expr::Literal { .. } => (),
            Expr::Logical { left, right, .. } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            Expr::Unary { right, .. } => {
                self.resolve_expr(right);
            }
        };
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: Token) {
        // Put the variable name into the current scope (top of the stack)
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(&name.lexeme) {
                Lox::parse_error(&name, "Already a variable with this name in this scope.");
            }

            // This is just a declaration, so the value is `false`
            // since we haven't finished resolving `name`
            scope.insert(name.lexeme, false);
        }
    }

    fn define(&mut self, name: Token) {
        // Mark the declared varible as resolved
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.lexeme, true);
        }
    }

    fn resolve_local(&self, expr: &Expr, name: Token) {
        // Starting from the innermost scope (top of the stack), we check for `name`.
        // Then resolve it under the correct scope.
        // If we don't find it in `self.scopes`, we assume that it's global or undefined.
        for i in (0..self.scopes.len()).rev() {
            if self.scopes.get(i).unwrap().contains_key(&name.lexeme) {
                self.interpreter
                    .borrow_mut()
                    .resolve(expr.clone(), self.scopes.len() - 1 - i);
            }
        }
    }

    fn resolve_function(
        &mut self,
        params: &Vec<Token>,
        body: &Vec<Option<Box<Stmt>>>,
        func_type: FunctionType,
    ) {
        let enclosing_func: FunctionType = self.current_function.clone();
        self.current_function = func_type;

        // Activate the function's scope
        self.begin_scope();

        // Resolve all arguments
        for param in params {
            self.declare(param.clone());
            self.define(param.clone());
        }

        // Resolve the body block
        self.resolve_stmt_list(body);

        // Back to the outer scope
        self.end_scope();

        self.current_function = enclosing_func;
    }
}
