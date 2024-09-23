use rand::distributions::Uniform;
use rand::Rng;
use rust_i18n::t;
use std::convert::{TryFrom, TryInto};

use crate::{
    call_stack::{Control, ControlStack},
    environments::hash_map_environment::HashMapEnvironment,
    error::RollerError,
    types::{Context, Environment, Expression, Op, Statement, Visitor},
};

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

pub struct EvalVisitor<'a, T: Rng + ?Sized, E: Environment, C: Context> {
    rng: &'a mut T,
    env: &'a mut E,
    ctx: C,
}

impl<'a, T: Rng, E: Environment, C: Context> EvalVisitor<'a, T, E, C> {
    pub fn new(rng: &'a mut T, env: &'a mut E, ctx: C) -> Self {
        EvalVisitor { rng, env, ctx }
    }
}

impl<'a, T: Rng, E: Environment + Clone, C: Context + Copy + Send>
    Visitor<Result<String, RollerError>, Result<Expression, RollerError>>
    for EvalVisitor<'a, T, E, C>
{
    async fn visit_expression(&mut self, expr: &Expression) -> Result<Expression, RollerError> {
        let mut stack = ControlStack::new(expr.clone());

        while stack.size_call() > 0 {
            match stack.peek_call()? {
                Expression::Term(left_expr, right_expr, _)
                | Expression::DiceRoll {
                    count: left_expr,
                    sides: right_expr,
                } => match stack.push_to_call_stack(&[*left_expr, *right_expr]) {
                    Control::Wait => continue,
                    Control::Continue => (),
                },
                Expression::DiceRollTemplateCall {
                    template_expression,
                    args,
                } => {
                    let mut calls = vec![*template_expression];
                    for arg in args {
                        calls.push(arg)
                    }
                    match stack.push_to_call_stack(calls.as_slice()) {
                        Control::Wait => continue,
                        Control::Continue => (),
                    }
                }
                _ => (),
            }

            match stack.pop_call()? {
                expr @ Expression::Integer(_) => {
                    stack.push_return(expr.clone());
                }
                Expression::Term(_, _, op) => {
                    let left = stack.pop_return()?;
                    let right = stack.pop_return()?;
                    stack.push_return(Expression::Integer(handle_op(left, right, op.clone())?));
                }
                Expression::DiceRoll { count: _, sides: _ } => {
                    let count = stack.pop_return()?;
                    let sides = stack.pop_return()?;

                    stack.push_return(Expression::Integer(handle_roll(self.rng, count, sides)?));
                }
                Expression::Variable(variable_name) => {
                    match self.env.get(self.ctx, &variable_name).await {
                        Some(env_expr) => {
                            stack.push_return(env_expr);
                        }
                        None => {
                            return Err(RollerError::EvalError(format!(
                                "failed to lookup variable {}",
                                variable_name
                            )))
                        }
                    }
                }
                expr @ Expression::DiceRollTemplate {
                    args: _,
                    expressions: _,
                } => {
                    stack.push_return(expr.clone());
                }
                Expression::DiceRollTemplateCall {
                    template_expression: _,
                    args: _,
                } => match stack.pop_return() {
                    Ok(Expression::DiceRollTemplate {
                        args: arg_names,
                        expressions,
                    }) => {
                        let closure = self.env.closure(self.ctx).await?;
                        let mut new_env =
                            HashMapEnvironment::from_context_and_initial_values(self.ctx, closure);

                        for arg_name in arg_names {
                            let arg = stack.pop_return()?;
                            new_env.set(self.ctx, &arg_name, &arg).await
                        }

                        // For now just support one expression in a template
                        match expressions.last() {
                            Some(expr) => {
                                stack.push_return(
                                    Box::pin(
                                        EvalVisitor::new(self.rng, &mut new_env, self.ctx)
                                            .visit_expression(expr),
                                    )
                                    .await?
                                    .clone(),
                                );
                            }
                            None => {
                                return Err(RollerError::EvalError(
                                    "missing body for dice roll template".to_string(),
                                ))
                            }
                        }
                    }
                    _ => return Err(RollerError::EvalError("not callable".to_string())),
                },
            }
        }

        match stack.pop_return() {
            Ok(expr) => Ok(expr),
            Err(_) => Err(RollerError::EvalError(
                "evaluation did not produce a result".to_string(),
            )),
        }
    }

    async fn visit_statement(&mut self, stmt: &Statement) -> Result<String, RollerError> {
        match stmt {
            Statement::Help => Ok(t!("help-general").to_string()),
            Statement::PrintEnv => Ok(self.env.print(self.ctx).await.to_string()),
            Statement::Roll(ref expr) => Ok(format!(
                "{}",
                i64::try_from(self.visit_expression(expr).await?)?
            )),
            Statement::SetValue(variable, ref expr) => {
                let value = self.visit_expression(expr).await?;
                let return_string = format!("{:?} => {:?}", variable, value);
                self.env.set(self.ctx, variable, &value).await;
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

    #[tokio::test]
    async fn test_eval() {
        let mut rng = StepRng::new(0, 1);
        let mut env = HashMapEnvironment::new();
        let mut visitor = EvalVisitor::new(&mut rng, &mut env, &TestCtx {});
        assert_eq!(
            visitor
                .visit_expression(&Box::new(Expression::Integer(1)))
                .await
                .unwrap(),
            Expression::Integer(1)
        );
        assert_eq!(
            visitor
                .visit_expression(&Box::new(Expression::Term(
                    Box::new(Expression::Integer(1)),
                    Box::new(Expression::Integer(2)),
                    Op::Add
                )))
                .await
                .unwrap(),
            Expression::Integer(3)
        );
        assert_eq!(
            visitor
                .visit_expression(&Box::new(Expression::Term(
                    Box::new(Expression::Integer(1)),
                    Box::new(Expression::Integer(2)),
                    Op::Subtract,
                )))
                .await
                .unwrap(),
            Expression::Integer(-1)
        );
        assert_eq!(
            visitor
                .visit_statement(&Box::new(Statement::Roll(Box::new(Expression::Term(
                    Box::new(Expression::DiceRoll {
                        count: Box::new(Expression::Integer(1)),
                        sides: Box::new(Expression::Integer(6))
                    }),
                    Box::new(Expression::Integer(1)),
                    Op::Add
                )))))
                .await
                .unwrap(),
            "2"
        );
        assert_eq!(
            visitor
                .visit_statement(&Box::new(Statement::Roll(Box::new(Expression::Term(
                    Box::new(Expression::Integer(1)),
                    Box::new(Expression::DiceRoll {
                        count: Box::new(Expression::Integer(1)),
                        sides: Box::new(Expression::Integer(6))
                    }),
                    Op::Add
                )))))
                .await
                .unwrap(),
            "2"
        );
        assert_eq!(
            visitor
                .visit_expression(&Box::new(Expression::DiceRoll {
                    count: Box::new(Expression::Integer(1231239)),
                    sides: Box::new(Expression::Integer(410123123))
                }))
                .await
                .unwrap(),
            Expression::Integer(1231239)
        );
        assert_eq!(
            visitor
                .visit_expression(&Box::new(Expression::DiceRollTemplateCall {
                    template_expression: Box::new(Expression::DiceRollTemplate {
                        args: vec![],
                        expressions: vec![Expression::DiceRoll {
                            count: Box::new(Expression::Integer(1)),
                            sides: Box::new(Expression::Integer(4)),
                        }]
                    }),
                    args: vec![],
                }))
                .await
                .unwrap(),
            Expression::Integer(1),
        );
        assert_eq!(
            visitor
                .visit_expression(&Box::new(Expression::DiceRollTemplateCall {
                    template_expression: Box::new(Expression::DiceRollTemplate {
                        args: vec!["A".to_string(), "B".to_string()],
                        expressions: vec![Expression::Term(
                            Box::new(Expression::DiceRoll {
                                count: Box::new(Expression::Variable("A".to_string())),
                                sides: Box::new(Expression::Integer(4)),
                            }),
                            Box::new(Expression::Variable("B".to_string())),
                            Op::Add,
                        )]
                    }),
                    args: vec![Expression::Integer(2), Expression::Integer(6)],
                }))
                .await
                .unwrap(),
            Expression::Integer(8),
        );
        assert_eq!(
            visitor
                .visit_expression(&Box::new(Expression::DiceRollTemplateCall {
                    template_expression: Box::new(Expression::DiceRollTemplate {
                        args: vec!["A".to_string(), "B".to_string()],
                        expressions: vec![Expression::Term(
                            Box::new(Expression::DiceRoll {
                                count: Box::new(Expression::Variable("A".to_string())),
                                sides: Box::new(Expression::Integer(4)),
                            }),
                            Box::new(Expression::Variable("B".to_string())),
                            Op::Subtract,
                        )]
                    }),
                    args: vec![Expression::Integer(2), Expression::Integer(6)],
                }))
                .await
                .unwrap(),
            Expression::Integer(-4),
        );
    }
}
