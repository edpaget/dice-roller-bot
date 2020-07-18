use std::io::{self, Write};
use crate::parser::roll;
use crate::eval::eval;

use rustyline::error::ReadlineError;
use rustyline::Editor;

pub fn init() {
    let mut rl = Editor::<()>::new();

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                let mut rng = rand::thread_rng();
                let result = format!(
                    "{}\n",
                    eval(&mut rng, Box::new(roll(&line[..]).unwrap().1))
                );
                io::stdout().write(result.to_string().as_bytes()).unwrap();
                io::stdout().flush().unwrap();
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
}
