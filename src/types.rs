#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Roll(Box<Expression>),
    SetValue(String, Box<Expression>),
    PrintEnv,
    Help,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Op {
    Add,
    Subtract,
}

#[derive(Debug, PartialEq, Clone)]
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

pub trait Environment {
    fn get(&self, user_name: &str, var_name: &str) -> Option<&Expression>;
    fn set(&mut self, user_name: &str, var_name: &str, value: &Expression);
}

pub trait Visitor<S, E> {
    fn visit_statement(&mut self, stmt: &Statement) -> S;
    fn visit_expression(&mut self, expr: &Expression) -> E;
}
