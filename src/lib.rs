pub mod discord;
pub mod readline;
pub mod repl;

mod environments;
mod eval;
mod parser;
mod types;

rust_i18n::i18n!("locales");
