use std::env;

use rlox::{handle_error, run_file, run_prompt};

fn main() {
    let arg: Vec<String> = env::args().collect();

    match arg.len() {
        1 => run_prompt(),
        2 => run_file(&arg[1]).unwrap_or_else(|err| {
            handle_error(err.to_string());
        }),
        _ => {
            handle_error("Usage: rlox [script]".to_string());
        }
    }
}
