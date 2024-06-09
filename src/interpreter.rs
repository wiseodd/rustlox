use std::{
    cell::RefCell,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    callable::LoxCallable,
    environment::Environment,
    error::RuntimeError,
    expr::Expr,
    lox::Lox,
    object::Object,
    stmt::Stmt,
    token::{Literal, TokenType},
};

type Pointer<T> = Rc<RefCell<T>>;

#[derive(Default)]
pub struct Interpreter {
    globals: Pointer<Environment>,
    environment: Pointer<Environment>,
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Rc::new(RefCell::new(Environment::new(None)));

        let clock: Object = Object::Callable(LoxCallable::Native {
            arity: 0,
            body: Box::new(|_arguments: &Vec<Object>| {
                Object::Number(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs_f64(),
                )
            }),
        });
        globals.borrow_mut().define("clock".to_string(), clock);

        Interpreter {
            globals: globals.clone(),
            environment: globals.clone(),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Option<Stmt>>) {
        for stmt in statements.into_iter().flatten() {
            self.execute(&stmt);
        }
    }

    // TODO: Modularize
    fn execute(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expression { expression: expr } => match self.evaluate(expr) {
                Ok(_) => (),
                Err(error) => Lox::runtime_error(error.token, &error.message),
            },
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let _cond: Object = match self.evaluate(condition) {
                    Ok(literal) => literal,
                    Err(error) => return Lox::runtime_error(error.token, &error.message),
                };

                if is_truthy(_cond) {
                    self.execute(then_branch);
                } else {
                    match &**else_branch {
                        Some(else_stmt) => self.execute(else_stmt),
                        _ => (), // do nothing
                    };
                }
            }
            Stmt::While { condition, body } => {
                while is_truthy(match self.evaluate(condition) {
                    Ok(literal) => literal,
                    Err(error) => return Lox::runtime_error(error.token, &error.message),
                }) {
                    self.execute(body);
                }
            }
            Stmt::Print { expression: expr } => match self.evaluate(expr) {
                Ok(lit) => println!("{}", stringify(lit)),
                Err(error) => Lox::runtime_error(error.token, &error.message),
            },
            Stmt::Var { name, initializer } => {
                let value: Object = match initializer {
                    Some(init_expr) => match self.evaluate(init_expr) {
                        Ok(expr_val) => expr_val,
                        Err(_) => Object::None,
                    },
                    None => Object::None,
                };

                let mut env = RefCell::borrow_mut(&self.environment);
                env.define(name.lexeme.to_owned(), value);
            }
            Stmt::Block { statements } => {
                let previous = self.environment.clone();
                self.environment = Rc::new(RefCell::new(Environment::new(Some(
                    self.environment.clone(),
                ))));

                for stmt in statements.to_owned().iter().flatten() {
                    self.execute(stmt);
                }

                self.environment = previous;
            }
            _ => unreachable!(),
        }
    }

    // TODO: Modularize
    fn evaluate(&mut self, expr: &Expr) -> Result<Object, RuntimeError> {
        match expr {
            Expr::Literal { value } => match value {
                Literal::String(val) => Ok(Object::String(val.clone())),
                Literal::Number(val) => Ok(Object::Number(val.clone())),
                Literal::Boolean(val) => Ok(Object::Boolean(val.clone())),
                Literal::None => Ok(Object::None),
            },
            Expr::Grouping { expression } => self.evaluate(expression),
            Expr::Assign { name, value } => {
                let val: Object = self.evaluate(value)?;
                self.environment.borrow_mut().assign(name, val.clone())?;
                Ok(val)
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left_lit: Object = self.evaluate(left)?;

                match operator.token_type {
                    TokenType::Or => {
                        if is_truthy(left_lit.clone()) {
                            return Ok(left_lit);
                        }
                    }
                    _ => {
                        if !is_truthy(left_lit.clone()) {
                            return Ok(left_lit);
                        }
                    }
                }

                self.evaluate(right)
            }
            Expr::Call {
                callee,
                paren,
                arguments,
            } => {
                let mut arguments_vals: Vec<Object> = vec![];
                for arg in arguments.iter() {
                    arguments_vals.push(self.evaluate(arg)?);
                }

                let function: LoxCallable = match self.evaluate(callee)? {
                    Object::Callable(func) => func,
                    _ => {
                        return Err(RuntimeError {
                            message: "Callee of a function must be a LoxCallable".to_string(),
                            token: Some(paren.clone()),
                        })
                    }
                };

                if arguments_vals.len() != function.arity() {
                    return Err(RuntimeError {
                        message: format!(
                            "Expected {} arguments but got {}.",
                            function.arity(),
                            arguments.len()
                        ),
                        token: Some(paren.clone()),
                    });
                }

                return Ok(function.call(&self, &arguments_vals));
            }
            Expr::Unary { operator, right } => {
                // Recursion to get the leaf (always a literal)
                let right: Object = self.evaluate(right)?;

                // Apply the unary operator
                match operator.token_type {
                    TokenType::Bang => match right {
                        Object::Boolean(value) => Ok(Object::Boolean(!value)),
                        _ => Err(RuntimeError {
                            message: "Operand must be a boolean.".to_string(),
                            token: Some(operator.clone()),
                        }),
                    },
                    TokenType::Minus => match right {
                        Object::Number(value) => Ok(Object::Number(-value)),
                        _ => Err(RuntimeError {
                            message: "Operand must be a number.".to_string(),
                            token: Some(operator.clone()),
                        }),
                    },
                    _ => Err(RuntimeError {
                        message: "Invalid operator.".to_string(),
                        token: Some(operator.clone()),
                    }),
                }
            }
            Expr::Variable { name } => self.environment.borrow_mut().get(name),
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                // DFS
                let left: Object = self.evaluate(left)?;
                let right: Object = self.evaluate(right)?;

                match operator.token_type {
                    TokenType::Minus => match (left, right) {
                        (Object::Number(val1), Object::Number(val2)) => {
                            Ok(Object::Number(val1 - val2))
                        }
                        _ => Err(RuntimeError {
                            message: "Operands must be numbers.".to_string(),
                            token: Some(operator.clone()),
                        }),
                    },
                    TokenType::Slash => match (left, right) {
                        (Object::Number(val1), Object::Number(val2)) => {
                            Ok(Object::Number(val1 / val2))
                        }
                        _ => Err(RuntimeError {
                            message: "Operands must be numbers.".to_string(),
                            token: Some(operator.clone()),
                        }),
                    },
                    TokenType::Plus => match (left, right) {
                        (Object::Number(val1), Object::Number(val2)) => {
                            Ok(Object::Number(val1 + val2))
                        }
                        (Object::String(val1), Object::String(val2)) => {
                            let mut res: String = val1.clone();
                            res.push_str(&val2);
                            Ok(Object::String(res))
                        }
                        _ => Err(RuntimeError {
                            message: "Operands must be both numbers or strings.".to_string(),
                            token: Some(operator.clone()),
                        }),
                    },
                    TokenType::Star => match (left, right) {
                        (Object::Number(val1), Object::Number(val2)) => {
                            Ok(Object::Number(val1 * val2))
                        }
                        _ => Err(RuntimeError {
                            message: "Operands must be numbers.".to_string(),
                            token: Some(operator.clone()),
                        }),
                    },
                    TokenType::Greater => match (left, right) {
                        (Object::Number(val1), Object::Number(val2)) => {
                            Ok(Object::Boolean(val1 > val2))
                        }
                        _ => Err(RuntimeError {
                            message: "Operands must be numbers.".to_string(),
                            token: Some(operator.clone()),
                        }),
                    },
                    TokenType::GreaterEqual => match (left, right) {
                        (Object::Number(val1), Object::Number(val2)) => {
                            Ok(Object::Boolean(val1 >= val2))
                        }
                        _ => Err(RuntimeError {
                            message: "Operands must be numbers.".to_string(),
                            token: Some(operator.clone()),
                        }),
                    },
                    TokenType::Less => match (left, right) {
                        (Object::Number(val1), Object::Number(val2)) => {
                            Ok(Object::Boolean(val1 < val2))
                        }
                        _ => Err(RuntimeError {
                            message: "operands must be numbers.".to_string(),
                            token: Some(operator.clone()),
                        }),
                    },
                    TokenType::LessEqual => match (left, right) {
                        (Object::Number(val1), Object::Number(val2)) => {
                            Ok(Object::Boolean(val1 <= val2))
                        }
                        _ => Err(RuntimeError {
                            message: "Operands must be numbers.".to_string(),
                            token: Some(operator.clone()),
                        }),
                    },
                    TokenType::BangEqual => Ok(Object::Boolean(!is_equal(left, right))),
                    TokenType::EqualEqual => Ok(Object::Boolean(is_equal(left, right))),
                    _ => Err(RuntimeError {
                        message: "Invalid operator.".to_string(),
                        token: Some(operator.clone()),
                    }),
                }
            }
            _ => Err(RuntimeError {
                message: "Unsupported expression.".to_owned(),
                token: None,
            }),
        }
    }
}

fn is_truthy(a: Object) -> bool {
    match a {
        Object::None => false,
        Object::Boolean(val) => val,
        _ => true,
    }
}

fn is_equal(a: Object, b: Object) -> bool {
    match (a, b) {
        (Object::None, Object::None) => true,
        (Object::None, _) => false,
        (_, Object::None) => false,
        (Object::Number(val1), Object::Number(val2)) => val1 == val2,
        (Object::String(val1), Object::String(val2)) => val1 == val2,
        (Object::Boolean(val1), Object::Boolean(val2)) => val1 == val2,
        _ => false,
    }
}

fn stringify(lit: Object) -> String {
    match lit {
        Object::None => "nil".to_owned(),
        Object::Number(val) => {
            // Integers are also stored as doubles.
            // So we need to cast back.
            let val_str: String = val.to_string();

            match val_str.strip_suffix(".0") {
                Some(stripped) => stripped.to_owned(),
                None => val_str,
            }
        }
        Object::Boolean(val) => val.to_string(),
        Object::String(val) => format!("{val}"),
        Object::Callable(name) => format!("Callable with name {name}"),
    }
}
