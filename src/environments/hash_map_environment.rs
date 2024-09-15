use core::fmt;
use std::{collections::HashMap, fmt::Display};

use crate::types::{Context, Environment, Expression};

#[derive(Clone)]
pub struct HashMapEnvironment {
    env: HashMap<String, HashMap<String, Expression>>,
}

impl Default for HashMapEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

impl HashMapEnvironment {
    pub fn from_context_and_initial_values<C: Context>(
        ctx: C,
        values: HashMap<String, Expression>,
    ) -> Self {
        let mut env = HashMap::new();
        env.insert(ctx.user_context_key(), values);
        HashMapEnvironment { env }
    }

    pub fn new() -> Self {
        HashMapEnvironment {
            env: HashMap::new(),
        }
    }
}

impl Environment for HashMapEnvironment {
    async fn get<C: Context>(&self, ctx: C, var_name: &str) -> Option<Expression> {
        Some(
            self.env
                .get(&ctx.user_context_key())
                .unwrap()
                .get(var_name)?
                .clone(),
        )
    }

    async fn set<C: Context>(&mut self, ctx: C, var_name: &str, result: &Expression) {
        if let Some(user_map) = self.env.get_mut(&ctx.user_context_key()) {
            user_map.insert(var_name.to_string(), result.clone());
        } else {
            let mut new_user_map = HashMap::new();
            new_user_map.insert(var_name.to_string(), result.clone());
            self.env
                .insert(ctx.user_context_key().to_string(), new_user_map);
        }
    }

    async fn print<C: Context>(&self, ctx: C) -> String {
        format!("{:?}", self.env.get(&ctx.user_context_key()).unwrap())
    }

    async fn closure<C: Context>(&self, ctx: C) -> Result<HashMap<String, Expression>, ()> {
        match self.env.get(&ctx.user_context_key()) {
            Some(map) => Ok(map.clone()),
            None => Ok(HashMap::new()),
        }
    }
}

impl Display for HashMapEnvironment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.env)
    }
}
