use anyhow::Result;
use std::{
    cmp::Ordering,
    env, fs,
    io::{self, Write},
    process,
};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    // The first element of `args` is always the exec. path
    match args.len().cmp(&2) {
        Ordering::Greater => {
            println!("Usage: `rustlox [script]`");
            process::exit(64);
        }
        Ordering::Equal => run_file(args[1].clone())?,
        _ => run_prompt()?,
    };

    Ok(())
}

fn run_file(path: String) -> Result<()> {
    let program: String = fs::read_to_string(path)?;
    run(program)?;
    Ok(())
}

fn run_prompt() -> Result<()> {
    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut line: String = String::new();
        match io::stdin().read_line(&mut line) {
            Err(_) => break,             // CTRL+C and other errors
            Ok(0) => break,              // EOF (CTRL+D) signal
            Ok(_) => println!("{line}"), // TODO: the actual thing
        };

        // TODO: run(line)?;
    }

    Ok(())
}

// TODO: implement!
fn run(source: String) -> Result<()> {
    todo!()
}
