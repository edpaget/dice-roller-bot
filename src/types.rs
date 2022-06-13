#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Roll(Box<Expression>),
    SetValue(String, Box<Expression>),
    PrintEnv,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Op{
    Add,
    Subtract,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Variable(String),
    Integer(i64),
    DiceRoll { count: Box<Expression>, sides: Box<Expression> },
    Term(Box<Expression>, Box<Expression>, Op),
}

pub trait Environment {
    fn get(&self, user_name: &String, var_name: &String) -> Option<Box<Expression>>;
    fn set(&mut self, user_name: &String, var_name: &String, value: Box<Expression>);
}

pub trait Visitor<S, E> {
    fn visit_statement(&mut self, stmt: Box<Statement>) -> S;
    fn visit_expression(&mut self, expr: Box<Expression>) -> E;
}
