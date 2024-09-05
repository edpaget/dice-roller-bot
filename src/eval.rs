use rand::distributions::Uniform;
use rand::Rng;
use rust_i18n::t;
use std::convert::{TryFrom, TryInto};

use crate::types::{Context, Environment, Expression, Op, Statement, Visitor};

impl TryFrom<Expression> for i64 {
    type Error = ();

    fn try_from(value: Expression) -> Result<i64, Self::Error> {
        match value {
            Expression::Integer(value) => Ok(value),
            _ => Err(()),
        }
    }
}

impl TryFrom<Expression> for usize {
    type Error = ();

    fn try_from(value: Expression) -> Result<usize, Self::Error> {
        match value {
            Expression::Integer(value) => Ok(usize::try_from(value).unwrap()),
            _ => Err(()),
        }
    }
}

fn handle_roll(rng: &mut impl Rng, count: Expression, sides: Expression) -> Result<i64, ()> {
    let die = Uniform::new_inclusive(1, i64::try_from(sides)?);

    Ok(rng.sample_iter(&die).take(count.try_into()?).sum())
}

fn handle_op(left: Expression, right: Expression, op: Op) -> Result<i64, ()> {
    match op {
        Op::Subtract => Ok(i64::try_from(left)? - i64::try_from(right)?),
        Op::Add => Ok(i64::try_from(left)? + i64::try_from(right)?),
    }
}

pub struct EvalVisitor<'a, T: Rng + ?Sized, E: Environment> {
    rng: &'a mut T,
    env: &'a mut E,
}

impl<'a, T: Rng, E: Environment> EvalVisitor<'a, T, E> {
    pub fn new(rng: &'a mut T, env: &'a mut E) -> Self {
        EvalVisitor { rng, env }
    }
}

impl<'a, T: Rng, E: Environment + Clone> Visitor<Result<String, ()>, Result<Expression, ()>>
    for EvalVisitor<'a, T, E>
{
    fn visit_expression<C: Context + Copy>(
        &mut self,
        ctx: C,
        expr: &Expression,
    ) -> Result<Expression, ()> {
        match expr {
            Expression::Integer(_) => Ok(expr.clone()),
            Expression::Term(ref left_expr, ref right_expr, op) => {
                Ok(Expression::Integer(handle_op(
                    self.visit_expression(ctx, left_expr)?,
                    self.visit_expression(ctx, right_expr)?,
                    op.clone(),
                )?))
            }
            Expression::DiceRoll {
                count: ref left_expr,
                sides: ref right_expr,
            } => {
                let left_val = self.visit_expression(ctx, left_expr)?;
                let right_val = self.visit_expression(ctx, right_expr)?;

                Ok(Expression::Integer(handle_roll(
                    self.rng, left_val, right_val,
                )?))
            }
            Expression::Variable(variable_name) => match self.env.get(ctx, variable_name) {
                Some(env_expr) => Ok(env_expr.clone()),
                None => Err(()),
            },
            Expression::DiceRollTemplate {
                args: _,
                expressions: _,
            } => Ok(expr.clone()),
            Expression::DiceRollTemplateCall {
                ref template_expression,
                args,
            } => match self.visit_expression(ctx, template_expression)? {
                Expression::DiceRollTemplate {
                    args: arg_names,
                    expressions,
                } => {
                    let mut new_env = self.env.clone();
                    for (arg_name, arg) in arg_names.iter().zip(args) {
                        new_env.set(ctx, arg_name, &self.visit_expression(ctx, arg)?)
                    }

                    let result: Result<Vec<Expression>, _> = expressions
                        .iter()
                        .map(|expr: &Expression| -> Result<Expression, ()> {
                            EvalVisitor::new(self.rng, &mut new_env).visit_expression(ctx, expr)
                        })
                        .collect();

                    Ok(result?.last().unwrap().clone())
                }
                _ => Err(()),
            },
        }
    }
    fn visit_statement<C: Context + Copy>(
        &mut self,
        ctx: C,
        stmt: &Statement,
    ) -> Result<String, ()> {
        match stmt {
            Statement::Help => Ok(t!("help-general").to_string()),
            Statement::PrintEnv => Ok(self.env.print(ctx).to_string()),
            Statement::Roll(ref expr) => Ok(format!(
                "{}",
                i64::try_from(self.visit_expression(ctx, expr)?)?
            )),
            Statement::SetValue(variable, ref expr) => {
                let value = self.visit_expression(ctx, expr)?;
                let return_string = format!("{:?} => {:?}", variable, value);
                self.env.set(ctx, variable, &value);
                Ok(return_string)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environments::hash_map_environment::HashMapEnvironment;
    use rand::rngs::mock::StepRng;

    struct TestCtx;

    impl Context for &TestCtx {
        fn user_context_key(&self) -> String {
            format!("scope:{}#scope_type:user#user:{}", "test", "test_user")
        }

        fn global_context_key(&self) -> String {
            format!("scope:{}#scope_type:user#user:{}", "test", "global")
        }
    }

    #[test]
    fn test_eval() {
        let mut rng = StepRng::new(0, 1);
        let mut env = HashMapEnvironment::new();
        let mut visitor = EvalVisitor::new(&mut rng, &mut env);
        let ctx = &TestCtx {};
        assert_eq!(
            visitor
                .visit_expression(ctx, &Box::new(Expression::Integer(1)))
                .unwrap(),
            Expression::Integer(1)
        );
        assert_eq!(
            visitor
                .visit_expression(
                    ctx,
                    &Box::new(Expression::Term(
                        Box::new(Expression::Integer(1)),
                        Box::new(Expression::Integer(2)),
                        Op::Add
                    ))
                )
                .unwrap(),
            Expression::Integer(3)
        );
        assert_eq!(
            visitor
                .visit_statement(
                    ctx,
                    &Box::new(Statement::Roll(Box::new(Expression::Term(
                        Box::new(Expression::DiceRoll {
                            count: Box::new(Expression::Integer(1)),
                            sides: Box::new(Expression::Integer(6))
                        }),
                        Box::new(Expression::Integer(1)),
                        Op::Add
                    ))))
                )
                .unwrap(),
            "2"
        );
        assert_eq!(
            visitor
                .visit_expression(
                    ctx,
                    &Box::new(Expression::DiceRoll {
                        count: Box::new(Expression::Integer(1231239)),
                        sides: Box::new(Expression::Integer(410123123))
                    })
                )
                .unwrap(),
            Expression::Integer(1231239)
        );
        assert_eq!(
            visitor
                .visit_expression(
                    ctx,
                    &Box::new(Expression::DiceRollTemplateCall {
                        template_expression: Box::new(Expression::DiceRollTemplate {
                            args: vec![],
                            expressions: vec![Expression::DiceRoll {
                                count: Box::new(Expression::Integer(1)),
                                sides: Box::new(Expression::Integer(4)),
                            }]
                        }),
                        args: vec![],
                    })
                )
                .unwrap(),
            Expression::Integer(1),
        );
    }
}
