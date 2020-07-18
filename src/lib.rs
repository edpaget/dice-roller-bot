mod types;
mod parser;
mod eval;
mod repl;
pub mod discord;

use std::env;

pub fn main() {
    let args = env::args_os();
    if args.len() == 1 {
        println!(
            "No arguments provided. Starting the REPL...\n Use Ctrl+C to quit.",
        );
        repl::init();
    }
}
