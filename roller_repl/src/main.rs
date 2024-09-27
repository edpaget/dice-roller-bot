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

#[tokio::main]
async fn repl_with_db() -> Result<()> {
    let ddb_client = roller_lang::dynamodb::DDBClient::with_default_table(
        roller_lang::dynamodb::make_client(true)
            .await
            .expect("failed to start dynamo client"),
    );
    let mut repl = roller_lang::repl::REPL::new(ddb_client);
    roller_lang::readline::init(&mut repl).await
}

#[tokio::main]
async fn std_repl() -> Result<()> {
    let mut repl = roller_lang::repl::REPL::default();
    roller_lang::readline::init(&mut repl).await
}
