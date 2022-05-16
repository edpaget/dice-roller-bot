use std::convert::TryInto;
use rand::Rng;
use rand::distributions::Uniform;

use crate::types::{Environment, Expression, Op};

enum EvaluatorOutput<'a> {
    RollResult(u64, &'a str),
    SetResult(&'a str)
}

fn handle_roll(rng: &mut impl Rng, count: u64, sides: u64) -> EvaluatorOutput {
    let die = Uniform::new_inclusive(1, sides);
    let rolls = rng.sample_iter(&die).take(count.try_into().unwrap());
    let roll_result = &rolls.map( |roll| roll.to_string() ).collect::<Vec<String>>().join(" + ");

    EvaluatorOutput::RollResult(
        rolls.sum(),
        format!("roll({})", roll_result)
    )
}

fn handle_op(left: EvaluatorOutput, right: EvaluatorOutput, op: Op) -> u64 {
    match op {
        Op::Subtract => left - right,
        Op::Add => left + right,
    }
}

pub fn eval(
    user: &String,
    env: &mut Environment,
    rng: &mut impl Rng,
    expr: Box<Expression>
) -> Result<EvaluatorOutput, &str>{
    match *expr {
        Expression::Integer(value) => value,
        Expression::Roll(expr) => eval(user, env, rng, expr),
        Expression::Term(left_expr, right_expr, op) => handle_op(
            eval(user, env, rng, left_expr),
            eval(user, env, rng, right_expr),
            op
        ),
        Expression::DiceRoll{ count: left_expr, sides: right_expr } => {
            let left_val = eval(user, env, rng, left_expr);
            let right_val = eval(user, env, rng, right_expr);

            Ok(handle_roll(rng, left_val, right_val))
        },
        Expression::Variable(variable_name) => {
            if let Some(variable_expr) = env.get(user, variable_name) {
                eval(user, env, rng, variable_expr)
            } else {
                Err("Can't find variable")
            }
        }
        Expression::SetEnv(variable, expr) => {
            if let Expression::Variable(var_name) = *variable {
                env.add(user, var_name, expr);
                Ok(EvaluatorOutput::SetResult(format!("Set {}", var_name)))
            } else {
                Err("Failed to set.")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::mock::StepRng;

    #[test]
    fn test_eval() {
        let mut rng = StepRng::new(0, 1);
        let mut env = Environment::new();
        assert_eq!(eval(
            &String::from("user"),
            &mut env,
            &mut rng,
            Box::new(Expression::Integer(1))
        ), 1);
        assert_eq!(eval(
            &String::from("user"),
            &mut env,
            &mut rng, Box::new(Expression::Term(
                Box::new(Expression::Integer(1)),
                Box::new(Expression::Integer(2)),
                Op::Add
            ))), 3);
        assert_eq!(eval(
            &String::from("user"),
            &mut env,
            &mut rng, Box::new(Expression::Roll(
                Box::new(
                    Expression::Term(
                        Box::new(Expression::DiceRoll {
                            count: Box::new(Expression::Integer(1)),
                            sides: Box::new(Expression::Integer(6))
                        }),
                        Box::new(Expression::Integer(1)),
                        Op::Add
                    )
                )
            ))), 2);
        assert_eq!(eval(
            &String::from("user"),
            &mut env,
            &mut rng,
            Box::new(Expression::DiceRoll {
                count: Box::new(Expression::Integer(1231239)),
                sides: Box::new(Expression::Integer(410123123))
            })
        ), 1231239);
    }
}
