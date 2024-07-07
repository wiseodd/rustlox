use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    callable::LoxCallable,
    environment::{self, Environment},
    error::LoxError,
    expr::Expr,
    lox::Lox,
    object::Object,
    stmt::Stmt,
    token::{Literal, Token, TokenType},
};

type Pointer<T> = Rc<RefCell<T>>;

#[derive(Default)]
pub struct Interpreter {
    pub globals: Pointer<Environment>,
    pub environment: Pointer<Environment>,
    pub locals: HashMap<Expr, usize>,
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
            locals: HashMap::new(),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Option<Stmt>>) {
        for stmt in statements.into_iter().flatten() {
            let _ = self.execute(&stmt);
        }
    }

    // TODO: Modularize
    pub fn execute(&mut self, stmt: &Stmt) -> Result<(), LoxError> {
        match stmt {
            Stmt::Expression { expression: expr } => match self.evaluate(expr) {
                Ok(_) => Ok(()),
                Err(LoxError::Return { value }) => return Err(LoxError::Return { value }),
                Err(error) => {
                    Lox::runtime_error(error);
                    Ok(())
                }
            },
            Stmt::Function { name, params, body } => {
                let function: LoxCallable = LoxCallable::User {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.to_vec(),
                    closure: self.environment.clone(),
                };
                self.environment
                    .borrow_mut()
                    .define(name.lexeme.clone(), Object::Callable(function));
                Ok(())
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let _cond: Object = match self.evaluate(condition) {
                    Ok(literal) => literal,
                    Err(LoxError::Return { value }) => return Err(LoxError::Return { value }),
                    Err(error) => {
                        Lox::runtime_error(error);
                        return Ok(());
                    }
                };

                if is_truthy(_cond) {
                    self.execute(then_branch)?;
                } else {
                    match &**else_branch {
                        Some(else_stmt) => self.execute(else_stmt),
                        _ => Ok(()), // do nothing
                    }?
                }
                Ok(())
            }
            Stmt::While { condition, body } => {
                while is_truthy(match self.evaluate(condition) {
                    Ok(literal) => literal,
                    Err(LoxError::Return { value }) => return Err(LoxError::Return { value }),
                    Err(error) => {
                        Lox::runtime_error(error);
                        return Ok(());
                    }
                }) {
                    self.execute(body)?;
                }
                Ok(())
            }
            Stmt::Print { expression: expr } => match self.evaluate(expr) {
                Ok(lit) => {
                    println!("{}", stringify(lit));
                    Ok(())
                }
                Err(LoxError::Return { value }) => return Err(LoxError::Return { value }),
                Err(error) => Err(error),
            },
            Stmt::Return { value, .. } => {
                let ret_val: Object = match value {
                    Some(expr) => {
                        //dbg!(expr.clone());
                        let res = self.evaluate(&expr)?;
                        //dbg!(res.clone());
                        res
                    }
                    None => Object::None,
                };

                //dbg!(ret_val.clone());

                Err(LoxError::Return { value: ret_val })
            }
            Stmt::Var { name, initializer } => {
                let value: Object = match initializer {
                    Some(init_expr) => self.evaluate(init_expr)?,
                    None => Object::None,
                };

                // dbg!(name);
                // dbg!(environment.clone());
                // dbg!("----------------------------");
                // let mut input = String::new();
                // let _ = std::io::stdin().read_line(&mut input);

                self.environment
                    .borrow_mut()
                    .define(name.lexeme.to_owned(), value);

                Ok(())
            }
            Stmt::Block { statements } => self.execute_block(
                statements,
                Rc::new(RefCell::new(Environment::new(Some(
                    self.environment.clone(),
                )))),
            ),
            _ => unreachable!(),
        }
    }

    pub fn execute_block(
        &mut self,
        statements: &Vec<Option<Box<Stmt>>>,
        environment: Rc<RefCell<Environment>>,
    ) -> Result<(), LoxError> {
        let previous = self.environment.clone();
        self.environment = environment.clone();

        for stmt in statements.to_owned().iter().flatten() {
            match self.execute(stmt) {
                Ok(()) => (), // All good, do nothing
                Err(err) => {
                    // Restore the original environment even after error
                    self.environment = previous;
                    return Err(err);
                }
            };
        }

        // Restore the original env
        self.environment = previous;
        Ok(())
    }

    pub fn resolve(&mut self, expr: Expr, depth: usize) {
        self.locals.insert(expr, depth);
    }

    // TODO: Modularize
    fn evaluate(&mut self, expr: &Expr) -> Result<Object, LoxError> {
        match expr {
            Expr::Literal { value } => match value {
                Literal::String(val) => Ok(Object::String(val.clone())),
                Literal::Number(val) => Ok(Object::Number(*val)),
                Literal::Boolean(val) => Ok(Object::Boolean(val.clone())),
                Literal::None => Ok(Object::None),
            },
            Expr::Grouping { expression } => self.evaluate(expression),
            Expr::Assign { name, value } => {
                let val: Object = self.evaluate(value)?;

                if let Some(distance) = self.locals.get(expr) {
                    environment::assign_at(
                        self.environment.clone(),
                        *distance,
                        name.clone(),
                        val.clone(),
                    )?;
                } else {
                    self.globals.borrow_mut().assign(name, val.clone())?;
                }

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
                        return Err(LoxError::RuntimeError {
                            message: "Callee of a function must be a LoxCallable".to_string(),
                            token: Some(paren.clone()),
                        })
                    }
                };

                if arguments_vals.len() != function.arity() {
                    return Err(LoxError::RuntimeError {
                        message: format!(
                            "Expected {} arguments but got {}.",
                            function.arity(),
                            arguments.len()
                        ),
                        token: Some(paren.clone()),
                    });
                }

                return Ok(function.call(self, &arguments_vals));
            }
            Expr::Unary { operator, right } => {
                // Recursion to get the leaf (always a literal)
                let right: Object = self.evaluate(right)?;

                // Apply the unary operator
                match operator.token_type {
                    TokenType::Bang => match right {
                        Object::Boolean(value) => Ok(Object::Boolean(!value)),
                        _ => Err(LoxError::RuntimeError {
                            message: "Operand must be a boolean.".to_string(),
                            token: Some(operator.clone()),
                        }),
                    },
                    TokenType::Minus => match right {
                        Object::Number(value) => Ok(Object::Number(-value.clone())),
                        _ => Err(LoxError::RuntimeError {
                            message: "Operand must be a number.".to_string(),
                            token: Some(operator.clone()),
                        }),
                    },
                    _ => Err(LoxError::RuntimeError {
                        message: "Invalid operator.".to_string(),
                        token: Some(operator.clone()),
                    }),
                }
            }
            Expr::Variable { name } => self.look_up_variable(name, expr),
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
                        _ => Err(LoxError::RuntimeError {
                            message: "Operands must be numbers.".to_string(),
                            token: Some(operator.clone()),
                        }),
                    },
                    TokenType::Slash => match (left, right) {
                        (Object::Number(val1), Object::Number(val2)) => {
                            Ok(Object::Number(val1 / val2))
                        }
                        _ => Err(LoxError::RuntimeError {
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
                        _ => Err(LoxError::RuntimeError {
                            message: "Operands must be both numbers or strings.".to_string(),
                            token: Some(operator.clone()),
                        }),
                    },
                    TokenType::Star => match (left, right) {
                        (Object::Number(val1), Object::Number(val2)) => {
                            Ok(Object::Number(val1 * val2))
                        }
                        _ => Err(LoxError::RuntimeError {
                            message: "Operands must be numbers.".to_string(),
                            token: Some(operator.clone()),
                        }),
                    },
                    TokenType::Greater => match (left, right) {
                        (Object::Number(val1), Object::Number(val2)) => {
                            Ok(Object::Boolean(val1 > val2))
                        }
                        _ => Err(LoxError::RuntimeError {
                            message: "Operands must be numbers.".to_string(),
                            token: Some(operator.clone()),
                        }),
                    },
                    TokenType::GreaterEqual => match (left, right) {
                        (Object::Number(val1), Object::Number(val2)) => {
                            Ok(Object::Boolean(val1 >= val2))
                        }
                        _ => Err(LoxError::RuntimeError {
                            message: "Operands must be numbers.".to_string(),
                            token: Some(operator.clone()),
                        }),
                    },
                    TokenType::Less => match (left.clone(), right.clone()) {
                        (Object::Number(val1), Object::Number(val2)) => {
                            Ok(Object::Boolean(val1 < val2))
                        }
                        _ => Err(LoxError::RuntimeError {
                            message: "Operands must be numbers.".to_string(),
                            token: Some(operator.clone()),
                        }),
                    },
                    TokenType::LessEqual => match (left, right) {
                        (Object::Number(val1), Object::Number(val2)) => {
                            Ok(Object::Boolean(val1 <= val2))
                        }
                        _ => Err(LoxError::RuntimeError {
                            message: "Operands must be numbers.".to_string(),
                            token: Some(operator.clone()),
                        }),
                    },
                    TokenType::BangEqual => Ok(Object::Boolean(!is_equal(left, right))),
                    TokenType::EqualEqual => Ok(Object::Boolean(is_equal(left, right))),
                    _ => Err(LoxError::RuntimeError {
                        message: "Invalid operator.".to_string(),
                        token: Some(operator.clone()),
                    }),
                }
            }
            _ => Err(LoxError::RuntimeError {
                message: "Unsupported expression.".to_owned(),
                token: None,
            }),
        }
    }

    fn look_up_variable(&self, name: &Token, expr: &Expr) -> Result<Object, LoxError> {
        if let Some(distance) = self.locals.get(expr) {
            environment::get_at(self.environment.clone(), *distance, name.lexeme.clone())
        } else {
            self.globals.borrow_mut().get(name)
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

fn stringify(obj: Object) -> String {
    match obj {
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
