extern crate dice_roller;

use clap::Parser;
use rustyline::Result;

/// REPL for dice-roller commands
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Persist environment to DynamoDB
    #[arg(short, long, default_value_t = true)]
    dynamodb: bool,
}

fn main() {
    let args = Args::parse();
    println!("No dice roll statement. Starting the REPL...\n Use Ctrl+C to quit.",);
    let repl = if args.dynamodb {
        repl_with_db()
    } else {
        std_repl()
    };
    match repl {
        Ok(()) => println!("Closing REPL"),
        Err(err) => println!("Error: {:?}", err),
    }
}

fn repl_with_db() -> Result<()> {
    let rt = tokio::runtime::Runtime::new().expect("failed to start tokio runtime");
    let client = rt
        .block_on(dice_roller::dynamodb::make_client(true))
        .expect("failed to start dynamo client");
    let mut repl = dice_roller::repl::REPL::new(&client, rt.handle());
    dice_roller::readline::init(&mut repl)
}

fn std_repl() -> Result<()> {
    let mut repl = dice_roller::repl::REPL::default();
    dice_roller::readline::init(&mut repl)
}
