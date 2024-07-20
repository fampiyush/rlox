use ::std::{error::Error, fs, io, process};
use std::io::Write;

// Error display with exit
pub fn handle_error(err: String) {
    eprintln!("{}", err);
    process::exit(1);
}

// For handling language errors
pub fn report(line: usize, message: &str) {
    let err = format!("[Line {}] Error: {}", line, message);
    handle_error(err);
}

// Called when no argument is provided
pub fn run_prompt() {
    loop {
        print!(">> ");
        let mut line = String::new();
        let _ = io::stdout().flush();
        io::stdin().read_line(&mut line).unwrap();
        run(&line);
    }
}

// Called when an argument is provided
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
