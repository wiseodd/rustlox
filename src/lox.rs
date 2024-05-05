use crate::{
    ast,
    expr::Expr,
    parser::Parser,
    scanner::Scanner,
    token::{Token, TokenType},
};
use anyhow::{anyhow, Result};
use rustyline::error::ReadlineError;
use std::{fs, process};

static mut HAD_ERROR: bool = false;

#[derive(Default)]
pub struct Lox {}

impl Lox {
    pub fn new() -> Self {
        Lox {}
    }

    pub fn run_file(&mut self, path: String) -> Result<()> {
        let program: String = fs::read_to_string(path)?;
        self.run(program);

        unsafe {
            if HAD_ERROR {
                process::exit(65);
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
            }
        }

        Ok(())
    }

    pub fn run(&mut self, source: String) {
        let mut scanner: Scanner = Scanner::new(source);
        let tokens: Vec<Token> = scanner.scan_tokens().unwrap().clone();

        let mut parser: Parser = Parser::new(tokens);
        let expression: Expr = parser.parse().unwrap();

        unsafe {
            if HAD_ERROR {
                return;
            }
        }

        println!("{}", ast::print(expression));
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

    pub fn report(line: usize, loc: &str, message: &str) {
        println!("[Line {line}] Error {loc}: {message}");

        unsafe {
            HAD_ERROR = true;
        }
    }
}
