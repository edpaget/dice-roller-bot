use crate::environments::hash_map_environment::HashMapEnvironment;
use crate::eval::EvalVisitor;
use crate::parser::StatementParser;
use crate::types::{Context, Environment, Parser, Visitor};
use rand::rngs::StdRng;
use rand::SeedableRng;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct REPLContext {
    repl_scope: String,
    user_id: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct REPLError;

impl REPLContext {
    pub fn new(repl_scope: String, user_id: String) -> Self {
        REPLContext {
            repl_scope,
            user_id,
        }
    }
}

impl Context for &REPLContext {
    fn user_context_key(&self) -> String {
        format!(
            "scope:{}#scope_type:user#user:{}",
            self.repl_scope, self.user_id
        )
    }

    fn global_context_key(&self) -> String {
        format!("scope:{}#scope_type:global", self.repl_scope)
    }
}

pub struct REPL<E: Environment> {
    parser: StatementParser,
    rng: StdRng,
    environment: E,
}

impl Default for REPL<HashMapEnvironment> {
    fn default() -> Self {
        REPL {
            parser: StatementParser,
            rng: StdRng::from_entropy(),
            environment: HashMapEnvironment::new(),
        }
    }
}

impl<E: Environment + Clone> REPL<E> {
    pub fn exec(&mut self, ctx: &REPLContext, input: &str) -> Result<String, REPLError> {
        match self.parser.parse(input) {
            Ok(ast) => {
                match EvalVisitor::new(&mut self.rng, &mut self.environment)
                    .visit_statement(ctx, &ast)
                {
                    Ok(result) => Ok(result),
                    Err(_) => Err(REPLError {}),
                }
            }
            Err(_) => Err(REPLError {}),
        }
    }
}
