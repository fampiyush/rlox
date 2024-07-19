use ::std::{error::Error, fs, process};

pub fn handle_error(err: String) {
    eprintln!("{}", err);
    process::exit(1);
}

pub fn run_prompt() {
    todo!();
}

pub fn run_file(arg: &str) -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string(arg)?;
    dbg!(content);
    Ok(())
}
