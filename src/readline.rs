use std::io::{self, Write};

use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};

use crate::repl::{REPLContext, REPL};
use crate::types::Environment;

pub async fn init<E: Environment + Clone>(repl: &mut REPL<E>) -> Result<()> {
    let mut rl = DefaultEditor::new()?;
    let ctx = &REPLContext::new("repl".to_string(), "user".to_string());

    loop {
        let readline = rl.readline(">> ");

        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                match repl.exec(ctx, &line[..]).await {
                    Ok(eval_result) => {
                        let result = format!("{}\n", eval_result);

                        io::stdout()
                            .write_all(result.to_string().as_bytes())
                            .unwrap();
                        io::stdout().flush().unwrap();
                    }
                    Err(_) => {
                        println!("HERE");
                        io::stdout()
                            .write_all("Something went wrong".to_string().as_bytes())
                            .unwrap();
                        io::stdout().flush().unwrap();
                    }
                }
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
