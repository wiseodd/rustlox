use crate::{scanner::Scanner, token::Token};
use anyhow::Result;
use rustyline::error::ReadlineError;
use std::{fs, process};

static mut HAD_ERROR: bool = false;

pub struct Lox {}

impl Lox {
    pub fn new() -> Self {
        Lox {}
    }

    pub fn run_file(&mut self, path: String) -> Result<()> {
        let program: String = fs::read_to_string(path)?;
        self.run(program)?;

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
                Ok(line) => self.run(line)?,
                Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                    println!("Kill signal received. Exiting...");
                    break;
                }
                Err(err) => println!("Error: {err:?}"),
            };

            unsafe {
                HAD_ERROR = false;
            }
        }

        Ok(())
    }

    pub fn run(&mut self, source: String) -> Result<()> {
        let mut scanner: Scanner = Scanner::new(source);
        scanner.scan_tokens();
        let tokens: Vec<Token> = scanner.tokens;

        for token in tokens {
            println!("{token}");
        }

        Ok(())
    }

    pub fn error(line: usize, message: &str) {
        Lox::report(line, "", message);
    }

    pub fn report(line: usize, loc: &str, message: &str) {
        println!("[Line {line}] Error {loc}: {message}");

        unsafe {
            HAD_ERROR = true;
        }
    }
}
