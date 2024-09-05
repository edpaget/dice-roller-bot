extern crate dice_roller;

use std::env;

fn main() {
    let args = env::args_os();
    if args.len() == 1 {
        println!("No arguments provided. Starting the REPL...\n Use Ctrl+C to quit.",);
        let mut repl = dice_roller::repl::REPL::default();
        let _ = dice_roller::readline::init(&mut repl);
    }
}
