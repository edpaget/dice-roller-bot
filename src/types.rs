use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    fn get<C: Context + Send>(
        &self,
        ctx: C,
        var_name: &str,
    ) -> impl std::future::Future<Output = Option<Expression>> + Send;
    fn set<C: Context + Send>(
        &mut self,
        ctx: C,
        var_name: &str,
        value: &Expression,
    ) -> impl std::future::Future<Output = ()> + Send;
    fn print<C: Context + Send>(&self, ctx: C) -> impl std::future::Future<Output = String> + Send;
    fn closure<C: Context + Send>(
        &self,
        ctx: C,
    ) -> impl std::future::Future<Output = Result<HashMap<String, Expression>, ()>> + Send;
}

pub trait Visitor<S, E> {
    async fn visit_expression(&mut self, expr: &Expression) -> E;
    async fn visit_statement(&mut self, stmt: &Statement) -> S;
}

pub trait Parser<E> {
    fn parse(&self, input: &str) -> Result<Statement, E>;
}
