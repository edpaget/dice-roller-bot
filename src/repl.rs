use std::io::{self, Write};

use crate::environments::hash_map_environment::HashMapEnvironment;
use crate::eval::EvalVisitor;
use crate::parser::command;
use crate::types::Visitor;

use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};

pub fn init() -> Result<()> {
    let mut rl = DefaultEditor::new()?;
    let mut env = HashMapEnvironment::new();
    let mut rng = rand::thread_rng();
    let user = String::from("User");
    let mut visitor = EvalVisitor::new(&mut rng, &mut env, &user);

    loop {
        let readline = rl.readline(">> ");

        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                let result = match command(&line[..]) {
                    Ok((_, stmt)) => {
                        format!("{}\n", visitor.visit_statement(Box::new(stmt)).unwrap())
                    }
                    Err(err) => format!("{}\n", err),
                };
                io::stdout()
                    .write_all(result.to_string().as_bytes())
                    .unwrap();
                io::stdout().flush().unwrap();
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    Ok(())
}
