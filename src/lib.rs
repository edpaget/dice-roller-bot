pub mod discord;
mod environments;
mod eval;
mod parser;
mod repl;
mod types;

use std::env;

pub fn start_repl() {
    let args = env::args_os();
    if args.len() == 1 {
        println!("No arguments provided. Starting the REPL...\n Use Ctrl+C to quit.",);
        repl::init();
    }
}
