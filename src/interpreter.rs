use anyhow::Result;

use crate::{
    error::RuntimeError,
    expr::Expr,
    lox::Lox,
    token::{Literal, TokenType},
};

pub fn interpret(expr: &Expr) {
    let res: Result<Literal, RuntimeError> = evaluate(expr);

    match res {
        Ok(lit) => {
            // Stringify
            let value_str: String = match lit {
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
                Literal::String(val) => val,
            };

            println!("{value_str}")
        }
        Err(error) => Lox::runtime_error(error.token, &error.message),
    }
}

fn evaluate(expr: &Expr) -> Result<Literal, RuntimeError> {
    match expr {
        Expr::Literal { value } => Ok(value.clone()),
        Expr::Grouping { expression } => evaluate(expression),
        Expr::Unary { operator, right } => {
            // Recursion to get the leaf (always a literal)
            let right: Literal = evaluate(right)?;

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
        Expr::Binary {
            left,
            operator,
            right,
        } => {
            // DFS
            let left: Literal = evaluate(left)?;
            let right: Literal = evaluate(right)?;

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

fn is_equal(a: Literal, b: Literal) -> bool {
    match (a, b) {
        (Literal::None, Literal::None) => true,
        (Literal::None, _) => false,
        (_, Literal::None) => false,
        (Literal::Number(val1), Literal::Number(val2)) => val1 == val2,
        (Literal::String(val1), Literal::String(val2)) => val1 == val2,
        _ => false,
    }
}
