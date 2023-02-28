use rust_polish_notation::*;
use std::env;
use std::process;

fn main() {
    env_logger::init();

    let args = env::args();
    let config = Config::build(args).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    match run(config) {
        Ok(val) => println!("Result: \x1b[0;32m{}", val),
        Err(e) => eprintln!("Problem evaluating expression: {}", e)
    };
}
