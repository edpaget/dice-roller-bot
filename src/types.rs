use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Roll(Box<Expression>),
    SetValue(String, Box<Expression>),
    PrintEnv,
    Help,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "op_type")]
pub enum Op {
    Add,
    Subtract,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(
    rename_all = "snake_case",
    tag = "expression_type",
    content = "expression"
)]
pub enum Expression {
    Variable(String),
    Integer(i64),
    DiceRollTemplate {
        args: Vec<String>,
        expressions: Vec<Expression>,
    },
    DiceRollTemplateCall {
        template_expression: Box<Expression>,
        args: Vec<Expression>,
    },
    DiceRoll {
        count: Box<Expression>,
        sides: Box<Expression>,
    },
    Term(Box<Expression>, Box<Expression>, Op),
}

pub trait Context {
    fn user_context_key(&self) -> String;
    fn global_context_key(&self) -> String;
}

pub trait Environment {
    fn get<C: Context>(&self, ctx: C, var_name: &str) -> Option<Expression>;
    fn set<C: Context>(&mut self, ctx: C, var_name: &str, value: &Expression);
    fn print<C: Context>(&self, ctx: C) -> String;
}

pub trait Visitor<S, E> {
    fn visit_statement(&mut self, stmt: &Statement) -> S;
    fn visit_expression(&mut self, expr: &Expression) -> E;
}

pub trait Parser<E> {
    fn parse(&self, input: &str) -> Result<Statement, E>;
}
