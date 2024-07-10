use crate::{
    error::LoxError,
    interpreter::Interpreter,
    parser::Parser,
    resolver::Resolver,
    scanner::Scanner,
    stmt::Stmt,
    token::{Token, TokenType},
};
use anyhow::{anyhow, Result};
use rustyline::error::ReadlineError;
use std::{cell::RefCell, fs, process, rc::Rc};

static mut HAD_ERROR: bool = false;
static mut HAD_RUNTIME_ERROR: bool = false;

#[derive(Default)]
pub struct Lox {
    interpreter: Rc<RefCell<Interpreter>>,
}

impl Lox {
    pub fn new() -> Self {
        Lox {
            interpreter: Rc::new(RefCell::new(Interpreter::new())),
        }
    }

    pub fn run_file(&mut self, path: String) -> Result<()> {
        let program: String = fs::read_to_string(path)?;
        self.run(program);

        unsafe {
            if HAD_ERROR {
                process::exit(65);
            }
            if HAD_RUNTIME_ERROR {
                process::exit(70);
            }
        }

        Ok(())
    }

    pub fn run_prompt(&mut self) -> Result<()> {
        let mut rl = rustyline::DefaultEditor::new()?;

        loop {
            match rl.readline("\n>> ") {
                Ok(line) => self.run(line),
                Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                    println!("Kill signal received. Exiting...");
                    break;
                }
                Err(err) => return Err(anyhow!("Error: {err:?}")),
            };

            unsafe {
                HAD_ERROR = false;
                HAD_RUNTIME_ERROR = false;
            }
        }

        Ok(())
    }

    pub fn run(&mut self, source: String) {
        let mut scanner: Scanner = Scanner::new(source);
        let tokens: Vec<Token> = scanner.scan_tokens().unwrap().clone();

        let mut parser: Parser = Parser::new(tokens);
        let statements: Vec<Option<Stmt>> = parser.parse();

        unsafe {
            if HAD_ERROR {
                return;
            }
        }

        // Resolver does a static analysis. If it doesn't throw an error, then
        // the syntax is clean and the interpreter can run confidently.
        let mut resolver = Resolver::new(self.interpreter.clone());
        // Vec<Option<Stmt>> -> Vec<Option<Box<Stmt>>>
        resolver.resolve_stmt_list(
            &statements
                .iter()
                .map(|x| match x {
                    Some(stmt) => Some(Box::new(stmt.clone())),
                    None => None,
                })
                .collect(),
        );

        unsafe {
            if HAD_ERROR {
                return;
            }
        }

        self.interpreter.borrow_mut().interpret(statements);
    }

    pub fn error(line: usize, message: &str) {
        Lox::report(line, "", message);
    }

    pub fn parse_error(token: &Token, message: &str) {
        match token.token_type {
            TokenType::Eof => Lox::report(token.line, "at end", message),
            _ => Lox::report(token.line, &format!("at '{}'", token.lexeme), message),
        }
    }

    pub fn runtime_error(error: LoxError) {
        match error {
            LoxError::RuntimeError { message, token } => {
                match token {
                    Some(token) => println!("{}\n[line {}]", message, token.line),
                    None => println!("{}", message),
                }
                unsafe {
                    HAD_RUNTIME_ERROR = true;
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn report(line: usize, loc: &str, message: &str) {
        println!("[Line {line}] Error {loc}: {message}");

        unsafe {
            HAD_ERROR = true;
        }
    }
}
