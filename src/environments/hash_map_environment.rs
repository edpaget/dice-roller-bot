use core::fmt;
use std::{collections::HashMap, fmt::Display};

use crate::types::{Environment, Expression};

#[derive(Clone)]
pub struct HashMapEnvironment {
    env: HashMap<String, HashMap<String, Expression>>,
}

impl HashMapEnvironment {
    pub fn new() -> Self {
        HashMapEnvironment {
            env: HashMap::new(),
        }
    }
}

impl Environment for HashMapEnvironment {
    fn get(&self, user_name: &str, var_name: &str) -> Option<&Expression> {
        self.env.get(user_name).unwrap().get(var_name)
    }

    fn set(&mut self, user_name: &str, var_name: &str, result: &Expression) {
        if let Some(user_map) = self.env.get_mut(user_name) {
            user_map.insert(var_name.to_string(), result.clone());
        } else {
            let mut new_user_map = HashMap::new();
            new_user_map.insert(var_name.to_string(), result.clone());
            self.env.insert(user_name.to_string(), new_user_map);
        }
    }
}

impl Display for HashMapEnvironment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.env)
    }
}
