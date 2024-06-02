use anyhow::Result;
use lox::Lox;

use std::{cmp::Ordering, env, process};

pub mod ast;
pub mod callable;
pub mod environment;
pub mod error;
pub mod expr;
pub mod interpreter;
pub mod lox;
pub mod parser;
pub mod scanner;
pub mod stmt;
pub mod token;

fn main() -> Result<()> {
    let mut lox: Lox = Lox::new();
    let args: Vec<String> = env::args().collect();

    // The first element of `args` is always the exec. path
    match args.len().cmp(&2) {
        Ordering::Greater => {
            println!("Usage: `rustlox [script]`");
            process::exit(64);
        }
        Ordering::Equal => lox.run_file(args[1].clone())?,
        _ => lox.run_prompt()?,
    };

    Ok(())
}
