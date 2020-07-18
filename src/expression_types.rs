#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Integer(u64),
    DiceRoll { count: Box<Expression>, sides: Box<Expression> },
    Term(Box<Expression>, Box<Expression>, char),
    Roll(Box<Expression>),
}
