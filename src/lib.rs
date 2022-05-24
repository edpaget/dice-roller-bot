mod types;
mod parser;
mod eval;
mod repl;
mod environments;
pub mod discord;

use std::env;

pub fn start_repl() {
    let args = env::args_os();
    if args.len() == 1 {
        println!(
            "No arguments provided. Starting the REPL...\n Use Ctrl+C to quit.",
        );
        repl::init();
    }
}
