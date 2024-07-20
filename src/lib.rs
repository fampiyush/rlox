use ::std::{error::Error, fs, io, process};
use std::io::Write;

pub fn handle_error(err: String) {
    eprintln!("{}", err);
    process::exit(1);
}

pub fn run_prompt() {
    loop {
        print!(">> ");
        let mut line = String::new();
        let _ = io::stdout().flush();
        io::stdin().read_line(&mut line).unwrap();
        run(&line);
    }
}

pub fn run_file(arg: &str) -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string(arg)?;
    run(&content);
    Ok(())
}

fn run(content: &str) {
    if content.trim().to_lowercase() == "exit" {
        process::exit(0);
    }
    print!("{}", content);
}
