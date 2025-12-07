pub mod discord;
pub mod dynamodb;
pub mod environments;
pub mod error;
pub mod readline;
pub mod repl;

mod call_stack;
mod eval;
mod parser;
mod types;

rust_i18n::i18n!("../locales");
