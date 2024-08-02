use ::std::{error::Error, fs, io, process};
use std::io::Write;

use interpreter::Interpreter;
use parser::Parser;
use scanner::Scanner;
use token::{Token, TokenType};

mod ast_printer;
mod environment;
mod expr;
mod interpreter;
mod parser;
mod scanner;
mod stmt;
mod token;

// Error display with exit
pub fn handle_error(err: String) {
    eprintln!("{}", err);
    process::exit(1);
}

// For handling language errors
pub fn report(line: usize, message: &str) {
    let err = format!("[Line {}] Error: {}", line, message);
    eprintln!("{}", err);
}

pub fn error(token: Token, message: &str) {
    if token.ttype == TokenType::Eof {
        report(token.line, &("at end ".to_owned() + message));
    } else {
        report(
            token.line,
            &("at '".to_owned() + &token.lexeme + "'. " + message),
        );
    }
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
    let content = fs::read_to_string(arg);
    match &content {
        Ok(c) => {
            run(c);
            Ok(())
        }
        Err(_) => Err(format!("Error reading file '{}'", arg).into()),
    }
}

fn run(content: &str) {
    if content.trim().to_lowercase() == "exit" {
        process::exit(0);
    }
    let mut scanner = Scanner::new(content.trim().to_string());
    let tokens = scanner.scan_tokens();

    let mut parser = Parser::new(tokens);
    let statements = parser.parse();

    match &statements {
        Ok(e) => {
            let mut interpreter = Interpreter::new();
            let interpreted = interpreter.interpret(e);

            match &interpreted {
                Ok(_) => (),
                Err(_) => process::exit(70),
            }
        }
        Err(_) => process::exit(65),
    }
}
