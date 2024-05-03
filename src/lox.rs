use crate::{scanner::Scanner, token::Token};
use anyhow::Result;
use std::{
    fs,
    io::{self, Write},
    process,
};

pub struct Lox {
    had_error: bool,
}

impl Lox {
    pub fn new() -> Self {
        Lox { had_error: false }
    }

    pub fn run_file(&self, path: String) -> Result<()> {
        let program: String = fs::read_to_string(path)?;
        self.run(program)?;

        if self.had_error {
            process::exit(65);
        }

        Ok(())
    }

    pub fn run_prompt(&mut self) -> Result<()> {
        loop {
            print!("> ");
            io::stdout().flush()?;

            let mut line: String = String::new();
            match io::stdin().read_line(&mut line) {
                Err(_) => break, // CTRL+C and other errors
                Ok(0) => break,  // EOF (CTRL+D) signal
                Ok(_) => self.run(line),
            };

            self.had_error = false;
        }

        Ok(())
    }

    pub fn run(&self, source: String) -> Result<()> {
        let mut scanner: Scanner = Scanner::new(source);
        scanner.scan_tokens();
        let tokens: Vec<Token> = scanner.tokens;

        for token in tokens {
            println!("{token}");
        }

        Ok(())
    }

    pub fn error(&mut self, line: u32, message: String) {
        self.report(line, "".to_string(), message);
    }

    pub fn report(&mut self, line: u32, loc: String, message: String) {
        println!("[Line {line}] Error {loc}: {message}");
        self.had_error = true;
    }
}
