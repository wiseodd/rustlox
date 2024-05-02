use anyhow::Result;
use std::{
    cmp::Ordering,
    env, fs,
    io::{self, Write},
    process,
};

pub mod scanner;
pub mod token;

fn main() -> Result<()> {
    let mut had_error = false;
    let args: Vec<String> = env::args().collect();

    // The first element of `args` is always the exec. path
    match args.len().cmp(&2) {
        Ordering::Greater => {
            println!("Usage: `rustlox [script]`");
            process::exit(64);
        }
        Ordering::Equal => run_file(args[1].clone(), had_error)?,
        _ => run_prompt(had_error)?,
    };

    Ok(())
}

fn run_file(path: String, had_error: bool) -> Result<()> {
    let program: String = fs::read_to_string(path)?;
    run(program)?;

    if had_error {
        process::exit(65);
    }

    Ok(())
}

fn run_prompt(mut had_error: bool) -> Result<()> {
    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut line: String = String::new();
        match io::stdin().read_line(&mut line) {
            Err(_) => break,             // CTRL+C and other errors
            Ok(0) => break,              // EOF (CTRL+D) signal
            Ok(_) => println!("{line}"), // TODO: the actual thing
        };

        had_error = false;
    }

    Ok(())
}

// TODO: implement!
fn run(source: String) -> Result<()> {
    todo!()
}

fn error(line: u32, message: String, mut had_error: bool) {
    report(line, "".to_string(), message, had_error);
}

fn report(line: u32, loc: String, message: String, mut had_error: bool) {
    println!("[Line {line}] Error {loc}: {message}");
    had_error = true;
}
