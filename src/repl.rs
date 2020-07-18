use std::io::{self, Write};
use parser::parse;
use types::Expression;
use eval;

pub fn init() {
    let mut environment = Environment::new();
    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let result = format!(
            "{}\n",
            display(parse(&input[..]).and_then(
                |expr| evaluate(&mut environment, expr),
            ))
        );
        io::stdout().write(result.to_string().as_bytes()).unwrap();
        io::stdout().flush().unwrap();
    }
}
