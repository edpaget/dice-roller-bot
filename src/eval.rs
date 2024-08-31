use rand::distributions::Uniform;
use rand::Rng;
use rust_i18n::t;
use std::{convert::TryInto, fmt::Display};

use crate::types::{Environment, Expression, Op, Statement, Visitor};

fn handle_roll(rng: &mut impl Rng, count: i64, sides: i64) -> i64 {
    let die = Uniform::new_inclusive(1, sides);

    rng.sample_iter(&die).take(count.try_into().unwrap()).sum()
}

fn handle_op(left: i64, right: i64, op: Op) -> i64 {
    match op {
        Op::Subtract => left - right,
        Op::Add => left + right,
    }
}

pub struct EvalVisitor<'a, T: Rng + ?Sized, E: Environment + Display> {
    rng: &'a mut T,
    env: &'a mut E,
    user: &'a String,
}

impl<'a, T: Rng, E: Environment + Display> EvalVisitor<'a, T, E> {
    pub fn new(rng: &'a mut T, env: &'a mut E, user: &'a String) -> Self {
        EvalVisitor { rng, env, user }
    }
}

impl<'a, T: Rng, E: Environment + Display> Visitor<Option<String>, Option<i64>>
    for EvalVisitor<'a, T, E>
{
    fn visit_expression(&mut self, expr: Box<Expression>) -> Option<i64> {
        match *expr {
            Expression::Integer(value) => Some(value),
            Expression::Term(left_expr, right_expr, op) => Some(handle_op(
                self.visit_expression(left_expr).unwrap(),
                self.visit_expression(right_expr).unwrap(),
                op,
            )),
            Expression::DiceRoll {
                count: left_expr,
                sides: right_expr,
            } => {
                let left_val = self.visit_expression(left_expr).unwrap();
                let right_val = self.visit_expression(right_expr).unwrap();

                Some(handle_roll(self.rng, left_val, right_val))
            }
            Expression::Variable(variable_name) => {
                self.visit_expression(self.env.get(self.user, &variable_name).unwrap())
            }
        }
    }
    fn visit_statement(&mut self, stmt: Box<Statement>) -> Option<String> {
        match *stmt {
            Statement::Help => Some(t!("help-general").to_string()),
            Statement::PrintEnv => Some(format!("{}", self.env)),
            Statement::Roll(expr) => Some(format!("{}", self.visit_expression(expr).unwrap())),
            Statement::SetValue(variable, expr) => {
                let value = Expression::Integer(self.visit_expression(expr).unwrap());
                let return_string = format!("{:?} => {:?}", variable, value);
                self.env.set(self.user, &variable, Box::new(value));
                Some(return_string)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environments::hash_map_environment::HashMapEnvironment;
    use rand::rngs::mock::StepRng;

    #[test]
    fn test_eval() {
        let mut rng = StepRng::new(0, 1);
        let mut env = HashMapEnvironment::new();
        let user = String::from("user");
        let mut visitor = EvalVisitor::new(&mut rng, &mut env, &user);
        assert_eq!(
            visitor
                .visit_expression(Box::new(Expression::Integer(1)))
                .unwrap(),
            1
        );
        assert_eq!(
            visitor
                .visit_expression(Box::new(Expression::Term(
                    Box::new(Expression::Integer(1)),
                    Box::new(Expression::Integer(2)),
                    Op::Add
                )))
                .unwrap(),
            3
        );
        assert_eq!(
            visitor
                .visit_statement(Box::new(Statement::Roll(Box::new(Expression::Term(
                    Box::new(Expression::DiceRoll {
                        count: Box::new(Expression::Integer(1)),
                        sides: Box::new(Expression::Integer(6))
                    }),
                    Box::new(Expression::Integer(1)),
                    Op::Add
                )))))
                .unwrap(),
            "2"
        );
        assert_eq!(
            visitor
                .visit_expression(Box::new(Expression::DiceRoll {
                    count: Box::new(Expression::Integer(1231239)),
                    sides: Box::new(Expression::Integer(410123123))
                }))
                .unwrap(),
            1231239
        );
    }
}
