use std::convert::TryInto;
use rand::Rng;
use rand::distributions::Uniform;

use crate::types::Expression;

fn handle_roll(rng: &mut impl Rng, count: u64, sides: u64) -> u64 {
    let die = Uniform::new_inclusive(1, sides);

    rng.sample_iter(&die).take(count.try_into().unwrap()).sum()
}

fn handle_op(left: u64, right: u64, op: char) -> u64 {
    match op {
        '-' => left - right,
        '+' => left + right,
        _ => 0
    }
}

fn eval(rng: &mut impl Rng, expr: Box<Expression>) -> u64 {
    match *expr {
        Expression::Integer(value) => value,
        Expression::Roll(expr) => eval(rng, expr),
        Expression::Term(left_expr, right_expr, op) => handle_op(
            eval(rng, left_expr),
            eval(rng, right_expr),
            op
        ),
        Expression::DiceRoll{ count: left_expr, sides: right_expr } => {
            let left_val = eval(rng, left_expr);
            let right_val = eval(rng, right_expr);

            handle_roll(rng, left_val, right_val)
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::mock::StepRng;

    #[test]
    fn test_eval() {
        let mut rng = StepRng::new(0, 1);
        assert_eq!(eval(&mut rng, Box::new(Expression::Integer(1))), 1);
        assert_eq!(eval(&mut rng, Box::new(Expression::Term(
            Box::new(Expression::Integer(1)),
            Box::new(Expression::Integer(2)),
            '+'
        ))), 3);
        assert_eq!(eval(&mut rng, Box::new(Expression::Roll(
            Box::new(
                Expression::Term(
                    Box::new(Expression::DiceRoll {
                        count: Box::new(Expression::Integer(1)),
                        sides: Box::new(Expression::Integer(6))
                    }),
                    Box::new(Expression::Integer(1)),
                    '+'
                )
            )
        ))), 2);
        assert_eq!(eval(&mut rng, Box::new(Expression::DiceRoll {
            count: Box::new(Expression::Integer(1231239)),
            sides: Box::new(Expression::Integer(410123123))
        })), 1231239);
    }
}
