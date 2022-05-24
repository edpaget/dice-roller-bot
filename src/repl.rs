use std::io::{self, Write};

use crate::environments::hash_map_environment::HashMapEnvironment;
use crate::parser::command;
use crate::eval::EvalVisitor;
use crate::types::Visitor;

use rustyline::error::ReadlineError;
use rustyline::Editor;

pub fn init() {
    let mut rl = Editor::<()>::new();
    let mut env = HashMapEnvironment::new();
    let mut rng = rand::thread_rng();
    let mut visitor = EvalVisitor{
        rng: &mut rng,
        env: &mut env,
        user: &String::from("User"),
    };

    loop {
        let readline = rl.readline(">> ");

        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                let result = match command(&line[..]) {
                    Ok((_, stmt)) => format!("{}\n", visitor.visit_statement(Box::new(stmt)).unwrap()),
                    Err(err) => format!("{}\n", err),
                };
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
