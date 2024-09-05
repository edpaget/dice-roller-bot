use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{char, space0, space1},
    combinator::{map_res, opt},
    error::ErrorKind,
    multi::{many0, separated_list0, separated_list1},
    sequence::{delimited, preceded, tuple},
    Err::Error,
    IResult,
};

use crate::types::{Expression, Op, Parser, Statement};

// Parser Grammer
//
// Statement <- Roll | SetValue | Help
// SetValue <- (Variable, Expression)
// Roll <- Expression
// Help <- ()
//
// Expression <- Term | DiceRollTemplate | DiceRoll | Integer | Variable
// Term <- (DiceRoll | Integer | Variable) | (DiceRoll | Integer | Variable), Op
// DiceRollTemplate <- (...Variable, => ,...Expression)
// DiceRoll <- (Integer | Null), Integer
// Integer <- [0-9]+
// Variable <- {[A-z][A-z0-9-]+}

fn from_decimal(input: &str) -> Result<i64, std::num::ParseIntError> {
    input.parse::<i64>()
}

fn allowed_char(c: char) -> bool {
    let chars = ",!$;{}[]()=> \t\r\n";

    !chars.contains(c)
}

fn is_digit(c: char) -> bool {
    c.is_ascii_digit()
}

fn lparen(input: &str) -> IResult<&str, char> {
    delimited(space0, char('('), space0)(input)
}

fn rparen(input: &str) -> IResult<&str, char> {
    delimited(space0, char(')'), space0)(input)
}

fn sep_comma(input: &str) -> IResult<&str, char> {
    delimited(space0, char(','), space0)(input)
}

fn variable(input: &str) -> IResult<&str, &str> {
    take_while1(allowed_char)(input)
}

fn variable_ref(input: &str) -> IResult<&str, Expression> {
    let (input, var_name) = delimited(char('{'), variable, char('}'))(input)?;

    Ok((input, Expression::Variable(var_name.to_string())))
}

fn integer(input: &str) -> IResult<&str, Expression> {
    let (input, number) = map_res(take_while(is_digit), from_decimal)(input)?;

    Ok((input, Expression::Integer(number)))
}

fn operation(input: &str) -> IResult<&str, Op> {
    let (input, value) = alt((char('+'), char('-')))(input)?;

    match value {
        '+' => Ok((input, Op::Add)),
        '-' => Ok((input, Op::Subtract)),
        _ => Err(Error(nom::error::Error {
            input,
            code: ErrorKind::Char,
        })),
    }
}

fn term(input: &str) -> IResult<&str, Expression> {
    let (input, (expr1, exprs)) = tuple((
        sub_expression,
        many0(tuple((
            preceded(space1, operation),
            preceded(space1, sub_expression),
        ))),
    ))(input)?;

    Ok((input, exprs.into_iter().fold(expr1, parse_term)))
}

fn parse_term(left_expr: Expression, next: (Op, Expression)) -> Expression {
    let (op, right_expr) = next;

    Expression::Term(Box::new(left_expr), Box::new(right_expr), op)
}

fn dice_roll(input: &str) -> IResult<&str, Expression> {
    let (input, (count, _, sides)) = tuple((
        opt(alt((variable_ref, integer))),
        char('d'),
        alt((variable_ref, integer)),
    ))(input)?;

    if let Some(n) = count {
        Ok((
            input,
            Expression::DiceRoll {
                count: Box::new(n),
                sides: Box::new(sides),
            },
        ))
    } else {
        Ok((
            input,
            Expression::DiceRoll {
                count: Box::new(Expression::Integer(1)),
                sides: Box::new(sides),
            },
        ))
    }
}

fn arg_list(input: &str) -> IResult<&str, Vec<&str>> {
    delimited(lparen, separated_list0(sep_comma, variable), rparen)(input)
}

fn expression_list(input: &str) -> IResult<&str, Vec<Expression>> {
    separated_list1(sep_comma, expression)(input)
}

fn dice_roll_template(input: &str) -> IResult<&str, Expression> {
    let (input, (arg_list, _, expressions)) =
        tuple((arg_list, tag("=>"), delimited(lparen, expression, rparen)))(input)?;
    Ok((
        input,
        Expression::DiceRollTemplate {
            args: arg_list.iter().map(|v| v.to_string()).collect(),
            expressions: vec![expressions],
        },
    ))
}

fn dice_roll_template_call(input: &str) -> IResult<&str, Expression> {
    let (input, (template_expression, args)) = tuple((
        alt((dice_roll_template, variable_ref)),
        delimited(char('('), expression_list, char(')')),
    ))(input)?;

    Ok((
        input,
        Expression::DiceRollTemplateCall {
            template_expression: Box::new(template_expression),
            args,
        },
    ))
}

fn sub_expression(input: &str) -> IResult<&str, Expression> {
    alt((dice_roll, integer, variable_ref))(input)
}

fn expression(input: &str) -> IResult<&str, Expression> {
    alt((
        dice_roll_template_call,
        dice_roll_template,
        term,
        dice_roll,
        integer,
        variable_ref,
    ))(input)
}

fn print_env(input: &str) -> IResult<&str, Statement> {
    let (input, _) = tag("print-env")(input)?;

    Ok((input, Statement::PrintEnv))
}

fn set_value(input: &str) -> IResult<&str, Statement> {
    let (input, (var_name, expr)) = preceded(
        tag("set"),
        tuple((preceded(space1, variable), preceded(space1, expression))),
    )(input)?;

    Ok((
        input,
        Statement::SetValue(var_name.to_string(), Box::new(expr)),
    ))
}

fn roll(input: &str) -> IResult<&str, Statement> {
    let (input, expr) = preceded(tag("roll"), preceded(space1, expression))(input)?;

    Ok((input, Statement::Roll(Box::new(expr))))
}

fn help(input: &str) -> IResult<&str, Statement> {
    let (input, _) = tag("help")(input)?;

    Ok((input, Statement::Help))
}

fn command(input: &str) -> IResult<&str, Statement> {
    preceded(char('!'), alt((roll, set_value, print_env, help)))(input)
}

#[derive(Default)]
pub struct StatementParser;

impl Parser<()> for StatementParser {
    fn parse(&self, input: &str) -> Result<Statement, ()> {
        match command(input) {
            Ok((_, stmt)) => Ok(stmt),
            Err(_) => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_ref() {
        assert_eq!(
            variable_ref("{test010}").unwrap().1,
            Expression::Variable("test010".to_string())
        );
        assert_eq!(
            variable_ref("{another_one_again}").unwrap().1,
            Expression::Variable("another_one_again".to_string())
        );
        assert_eq!(
            variable_ref("{!test_fails}"),
            Err(Error(nom::error::Error {
                input: "!test_fails}",
                code: ErrorKind::TakeWhile1
            }))
        );
    }

    #[test]
    fn test_term() {
        assert_eq!(
            term("1 - 2").unwrap().1,
            Expression::Term(
                Box::new(Expression::Integer(1)),
                Box::new(Expression::Integer(2)),
                Op::Subtract
            )
        );
        assert_eq!(
            term("1 + 2").unwrap().1,
            Expression::Term(
                Box::new(Expression::Integer(1)),
                Box::new(Expression::Integer(2)),
                Op::Add
            )
        );
        assert_eq!(
            term("2d6 + 1").unwrap().1,
            Expression::Term(
                Box::new(Expression::DiceRoll {
                    count: Box::new(Expression::Integer(2)),
                    sides: Box::new(Expression::Integer(6))
                }),
                Box::new(Expression::Integer(1)),
                Op::Add
            )
        );
        assert_eq!(
            term("2d6 + 1 + 1d6 - 10").unwrap().1,
            Expression::Term(
                Box::new(Expression::Term(
                    Box::new(Expression::Term(
                        Box::new(Expression::DiceRoll {
                            count: Box::new(Expression::Integer(2)),
                            sides: Box::new(Expression::Integer(6))
                        }),
                        Box::new(Expression::Integer(1)),
                        Op::Add
                    )),
                    Box::new(Expression::DiceRoll {
                        count: Box::new(Expression::Integer(1)),
                        sides: Box::new(Expression::Integer(6))
                    }),
                    Op::Add
                )),
                Box::new(Expression::Integer(10)),
                Op::Subtract
            )
        );
    }

    #[test]
    fn test_integer() {
        assert_eq!(integer("1"), Ok(("", Expression::Integer(1))));
        assert_eq!(integer("2"), Ok(("", Expression::Integer(2))));
        assert_eq!(
            integer("f"),
            Err(Error(nom::error::Error {
                input: "f",
                code: ErrorKind::MapRes
            }))
        );
    }

    #[test]
    fn test_dice_roll() {
        assert_eq!(
            dice_roll("1d6"),
            Ok((
                "",
                Expression::DiceRoll {
                    count: Box::new(Expression::Integer(1)),
                    sides: Box::new(Expression::Integer(6))
                }
            ))
        );
        assert_eq!(
            dice_roll("1231239d410123123"),
            Ok((
                "",
                Expression::DiceRoll {
                    count: Box::new(Expression::Integer(1231239)),
                    sides: Box::new(Expression::Integer(410123123))
                }
            ))
        );
        assert_eq!(
            dice_roll("69d420"),
            Ok((
                "",
                Expression::DiceRoll {
                    count: Box::new(Expression::Integer(69)),
                    sides: Box::new(Expression::Integer(420))
                }
            ))
        );
        assert_eq!(
            dice_roll("2d8"),
            Ok((
                "",
                Expression::DiceRoll {
                    count: Box::new(Expression::Integer(2)),
                    sides: Box::new(Expression::Integer(8))
                }
            ))
        );
        assert_eq!(
            dice_roll("x9d420"),
            Err(Error(nom::error::Error {
                input: "x9d420",
                code: ErrorKind::Char
            }))
        )
    }

    #[test]
    fn test_dice_roll_template() {
        assert_eq!(
            dice_roll_template("(a,b) => ( {a}d6 + {b} )").unwrap().1,
            Expression::DiceRollTemplate {
                args: vec!["a".to_string(), "b".to_string()],
                expressions: vec![Expression::Term(
                    Box::new(Expression::DiceRoll {
                        count: Box::new(Expression::Variable("a".to_string())),
                        sides: Box::new(Expression::Integer(6))
                    }),
                    Box::new(Expression::Variable("b".to_string())),
                    Op::Add,
                )]
            },
        );
        assert_eq!(
            dice_roll_template("(a,  b ) => ({a}d6 + {b} ) ").unwrap().1,
            Expression::DiceRollTemplate {
                args: vec!["a".to_string(), "b".to_string()],
                expressions: vec![Expression::Term(
                    Box::new(Expression::DiceRoll {
                        count: Box::new(Expression::Variable("a".to_string())),
                        sides: Box::new(Expression::Integer(6))
                    }),
                    Box::new(Expression::Variable("b".to_string())),
                    Op::Add,
                )]
            },
        );
    }

    #[test]
    fn test_set() {
        assert_eq!(
            set_value("set foo 1").unwrap().1,
            Statement::SetValue("foo".to_string(), Box::new(Expression::Integer(1)))
        );
        assert_eq!(
            set_value("set bar 1d6").unwrap().1,
            Statement::SetValue(
                "bar".to_string(),
                Box::new(Expression::DiceRoll {
                    count: Box::new(Expression::Integer(1)),
                    sides: Box::new(Expression::Integer(6))
                })
            )
        );
        assert_eq!(
            set_value("set foo-bar 1d6 + 1").unwrap().1,
            Statement::SetValue(
                "foo-bar".to_string(),
                Box::new(Expression::Term(
                    Box::new(Expression::DiceRoll {
                        count: Box::new(Expression::Integer(1)),
                        sides: Box::new(Expression::Integer(6))
                    }),
                    Box::new(Expression::Integer(1)),
                    Op::Add
                ))
            )
        );
    }

    #[test]
    fn test_roll() {
        assert_eq!(
            roll("roll 1").unwrap().1,
            Statement::Roll(Box::new(Expression::Integer(1)))
        );
        assert_eq!(
            roll("roll 1d6").unwrap().1,
            Statement::Roll(Box::new(Expression::DiceRoll {
                count: Box::new(Expression::Integer(1)),
                sides: Box::new(Expression::Integer(6))
            }))
        );
        assert_eq!(
            roll("roll 1d6 + 1").unwrap().1,
            Statement::Roll(Box::new(Expression::Term(
                Box::new(Expression::DiceRoll {
                    count: Box::new(Expression::Integer(1)),
                    sides: Box::new(Expression::Integer(6))
                }),
                Box::new(Expression::Integer(1)),
                Op::Add
            )))
        );
    }

    #[test]
    fn test_command() {
        assert_eq!(command("!print-env").unwrap().1, Statement::PrintEnv);
        assert_eq!(command("!help").unwrap().1, Statement::Help);
        assert_eq!(
            command("!roll 1").unwrap().1,
            Statement::Roll(Box::new(Expression::Integer(1)))
        );
        assert_eq!(
            command("!roll (a, b) => ( {a}d6 + {b} )(1, 10)").unwrap().1,
            Statement::Roll(Box::new(Expression::DiceRollTemplateCall {
                template_expression: Box::new(Expression::DiceRollTemplate {
                    args: vec!["a".to_string(), "b".to_string()],
                    expressions: vec![Expression::Term(
                        Box::new(Expression::DiceRoll {
                            count: Box::new(Expression::Variable("a".to_string())),
                            sides: Box::new(Expression::Integer(6))
                        }),
                        Box::new(Expression::Variable("b".to_string())),
                        Op::Add,
                    )]
                }),
                args: vec![Expression::Integer(1), Expression::Integer(10)]
            }))
        );
        assert_eq!(
            command("!set bar 1d6").unwrap().1,
            Statement::SetValue(
                "bar".to_string(),
                Box::new(Expression::DiceRoll {
                    count: Box::new(Expression::Integer(1)),
                    sides: Box::new(Expression::Integer(6))
                })
            )
        );
    }
}
