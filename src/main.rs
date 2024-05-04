use anyhow::Result;
use lox::Lox;

use std::{cmp::Ordering, env, process};

pub mod expr;
pub mod lox;
pub mod parser;
pub mod scanner;
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
