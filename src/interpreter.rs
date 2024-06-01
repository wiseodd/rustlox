use anyhow::Result;

use crate::{
    environment::Environment,
    error::RuntimeError,
    expr::Expr,
    lox::Lox,
    stmt::Stmt,
    token::{Literal, TokenType},
};

#[derive(Default)]
pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            environment: Environment::new(None),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Option<Stmt>>) {
        for stmt in statements.into_iter().flatten() {
            self.execute(&stmt);
        }
    }

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
                let _cond: Literal = match self.evaluate(condition) {
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
                let literal: Literal = match self.evaluate(condition) {
                    Ok(literal) => literal,
                    Err(error) => return Lox::runtime_error(error.token, &error.message),
                };

                while is_truthy(literal.clone()) {
                    self.execute(body);
                }
            }
            Stmt::Print { expression: expr } => match self.evaluate(expr) {
                Ok(lit) => println!("{}", stringify(lit)),
                Err(error) => Lox::runtime_error(error.token, &error.message),
            },
            Stmt::Var { name, initializer } => {
                let value: Literal = match initializer {
                    Some(init_expr) => match self.evaluate(init_expr) {
                        Ok(expr_val) => expr_val,
                        Err(_) => Literal::None,
                    },
                    None => Literal::None,
                };

                self.environment.define(name.lexeme.to_owned(), value);
            }
            Stmt::Block { statements } => {
                let env: Environment = Environment::new(Some(self.environment.clone()));
                let previous: Environment = self.environment.clone();
                self.environment = env;

                for stmt in statements.to_owned().iter().flatten() {
                    self.execute(stmt);
                }

                self.environment = previous;
            }
            _ => unreachable!(),
        }
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Literal, RuntimeError> {
        match expr {
            Expr::Literal { value } => Ok(value.clone()),
            Expr::Grouping { expression } => self.evaluate(expression),
            Expr::Assign { name, value } => {
                let value: Literal = self.evaluate(value)?;
                self.environment.assign(name, value.clone())?;
                Ok(value)
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left_lit: Literal = self.evaluate(left)?;

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
            Expr::Unary { operator, right } => {
                // Recursion to get the leaf (always a literal)
                let right: Literal = self.evaluate(right)?;

                // Apply the unary operator
                match operator.token_type {
                    TokenType::Bang => match right {
                        Literal::Boolean(value) => Ok(Literal::Boolean(!value)),
                        _ => Err(RuntimeError {
                            message: "Operand must be a boolean.".to_string(),
                            token: Some(operator.clone()),
                        }),
                    },
                    TokenType::Minus => match right {
                        Literal::Number(value) => Ok(Literal::Number(-value)),
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
            Expr::Variable { name } => self.environment.get(name),
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                // DFS
                let left: Literal = self.evaluate(left)?;
                let right: Literal = self.evaluate(right)?;

                match operator.token_type {
                    TokenType::Minus => match (left, right) {
                        (Literal::Number(val1), Literal::Number(val2)) => {
                            Ok(Literal::Number(val1 - val2))
                        }
                        _ => Err(RuntimeError {
                            message: "Operands must be numbers.".to_string(),
                            token: Some(operator.clone()),
                        }),
                    },
                    TokenType::Slash => match (left, right) {
                        (Literal::Number(val1), Literal::Number(val2)) => {
                            Ok(Literal::Number(val1 / val2))
                        }
                        _ => Err(RuntimeError {
                            message: "Operands must be numbers.".to_string(),
                            token: Some(operator.clone()),
                        }),
                    },
                    TokenType::Plus => match (left, right) {
                        (Literal::Number(val1), Literal::Number(val2)) => {
                            Ok(Literal::Number(val1 + val2))
                        }
                        (Literal::String(val1), Literal::String(val2)) => {
                            let mut res: String = val1.clone();
                            res.push_str(&val2);
                            Ok(Literal::String(res))
                        }
                        _ => Err(RuntimeError {
                            message: "Operands must be both numbers or strings.".to_string(),
                            token: Some(operator.clone()),
                        }),
                    },
                    TokenType::Star => match (left, right) {
                        (Literal::Number(val1), Literal::Number(val2)) => {
                            Ok(Literal::Number(val1 * val2))
                        }
                        _ => Err(RuntimeError {
                            message: "Operands must be numbers.".to_string(),
                            token: Some(operator.clone()),
                        }),
                    },
                    TokenType::Greater => match (left, right) {
                        (Literal::Number(val1), Literal::Number(val2)) => {
                            Ok(Literal::Boolean(val1 > val2))
                        }
                        _ => Err(RuntimeError {
                            message: "Operands must be numbers.".to_string(),
                            token: Some(operator.clone()),
                        }),
                    },
                    TokenType::GreaterEqual => match (left, right) {
                        (Literal::Number(val1), Literal::Number(val2)) => {
                            Ok(Literal::Boolean(val1 >= val2))
                        }
                        _ => Err(RuntimeError {
                            message: "Operands must be numbers.".to_string(),
                            token: Some(operator.clone()),
                        }),
                    },
                    TokenType::Less => match (left, right) {
                        (Literal::Number(val1), Literal::Number(val2)) => {
                            Ok(Literal::Boolean(val1 < val2))
                        }
                        _ => Err(RuntimeError {
                            message: "operands must be numbers.".to_string(),
                            token: Some(operator.clone()),
                        }),
                    },
                    TokenType::LessEqual => match (left, right) {
                        (Literal::Number(val1), Literal::Number(val2)) => {
                            Ok(Literal::Boolean(val1 <= val2))
                        }
                        _ => Err(RuntimeError {
                            message: "Operands must be numbers.".to_string(),
                            token: Some(operator.clone()),
                        }),
                    },
                    TokenType::BangEqual => Ok(Literal::Boolean(!is_equal(left, right))),
                    TokenType::EqualEqual => Ok(Literal::Boolean(is_equal(left, right))),
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

fn is_truthy(a: Literal) -> bool {
    match a {
        Literal::None => false,
        Literal::Boolean(val) => val,
        _ => true,
    }
}

fn is_equal(a: Literal, b: Literal) -> bool {
    match (a, b) {
        (Literal::None, Literal::None) => true,
        (Literal::None, _) => false,
        (_, Literal::None) => false,
        (Literal::Number(val1), Literal::Number(val2)) => val1 == val2,
        (Literal::String(val1), Literal::String(val2)) => val1 == val2,
        (Literal::Boolean(val1), Literal::Boolean(val2)) => val1 == val2,
        _ => false,
    }
}

fn stringify(lit: Literal) -> String {
    match lit {
        Literal::None => "nil".to_owned(),
        Literal::Number(val) => {
            // Integers are also stored as doubles.
            // So we need to cast back.
            let val_str: String = val.to_string();

            match val_str.strip_suffix(".0") {
                Some(stripped) => stripped.to_owned(),
                None => val_str,
            }
        }
        Literal::Boolean(val) => val.to_string(),
        Literal::String(val) => format!("\"{val}\""),
    }
}
