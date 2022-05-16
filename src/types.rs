use std::collections::HashMap;
use std::result::Result;

#[derive(Debug, PartialEq, Clone)]
pub enum Op{
    Add,
    Subtract,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Variable(String),
    Integer(u64),
    DiceRoll { count: Box<Expression>, sides: Box<Expression> },
    Term(Box<Expression>, Box<Expression>, Op),
    Roll(Box<Expression>),
    SetEnv(Box<Expression>, Box<Expression>),
}

#[derive(Clone)]
pub struct Environment(HashMap<String, HashMap<String, Box<Expression>>>);

impl Environment {
    pub fn new() -> Environment {
        Environment(HashMap::new())
    }

    pub fn get(&self, user_name: &String, var_name: String) -> Option<Box<Expression>> {
        self.0.get(user_name).unwrap().get(&var_name).map(|e| e.clone())
    }

    pub fn add(&mut self, user_name: &String, var_name: String, result: Box<Expression>) -> &mut Environment {
        if let Some(user_map) = self.0.get_mut(user_name) {
            user_map.insert(var_name, result);
        } else {
            let mut new_user_map = HashMap::new();
            new_user_map.insert(var_name, result);
            &self.0.insert(user_name.to_string(), new_user_map);
        }
        self
    }
}

pub type EvaluatorResult<'a> = Result<(&'a str, Option<&'a str>), &'a str>;
