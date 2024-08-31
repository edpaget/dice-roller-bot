use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::char,
    combinator::{map_res, opt},
    error::ErrorKind,
    multi::many0,
    sequence::{preceded, tuple},
    Err::Error,
    IResult,
};

use crate::types::{Expression, Op, Statement};

// Parser Grammer
//
// Statement <- Roll | SetValue
// SetValue <- (Variable, Expression)
// Roll <- Expression
//
// Expression <- Term | DiceRoll | Integer | Variable
// Term <- (DiceRoll | Integer | Variable) | (DiceRoll | Integer | Variable), Op
// DiceRoll <- (Integer | Null), Integer
// Integer <- [0-9]+
// Variable <- [A-z][A-z0-9-]+

fn from_decimal(input: &str) -> Result<i64, std::num::ParseIntError> {
    i64::from_str_radix(input, 10)
}

fn allowed_char(c: char) -> bool {
    let chars = "!$; \t\r\n";

    !chars.contains(c)
}

fn is_digit(c: char) -> bool {
    c.is_digit(10)
}

fn sp(input: &str) -> IResult<&str, &str> {
    let chars = " \t\r\n";

    take_while(move |c| chars.contains(c))(input)
}

fn variable(input: &str) -> IResult<&str, Expression> {
    let (input, variable) = take_while1(allowed_char)(input)?;

    Ok((input, Expression::Variable(variable.to_string())))
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
            preceded(sp, operation),
            preceded(sp, sub_expression),
        ))),
    ))(input)?;

    Ok((input, exprs.into_iter().fold(expr1, parse_term)))
}

fn parse_term(left_expr: Expression, next: (Op, Expression)) -> Expression {
    let (op, right_expr) = next;

    Expression::Term(Box::new(left_expr), Box::new(right_expr), op)
}

fn dice_roll(input: &str) -> IResult<&str, Expression> {
    let (input, (count, _, sides)) = tuple((opt(integer), char('d'), integer))(input)?;

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

fn sub_expression(input: &str) -> IResult<&str, Expression> {
    alt((dice_roll, integer, variable))(input)
}

fn expression(input: &str) -> IResult<&str, Expression> {
    alt((term, dice_roll, integer, variable))(input)
}

fn print_env(input: &str) -> IResult<&str, Statement> {
    let (input, _) = tag("print_env")(input)?;

    Ok((input, Statement::PrintEnv))
}

fn set_value(input: &str) -> IResult<&str, Statement> {
    let (input, (variable, expr)) = preceded(
        tag("set"),
        tuple((preceded(sp, variable), preceded(sp, expression))),
    )(input)?;

    match variable {
        Expression::Variable(var_name) => {
            Ok((input, Statement::SetValue(var_name, Box::new(expr))))
        }
        _ => Err(Error(nom::error::Error {
            input,
            code: ErrorKind::Tag,
        })),
    }
}

fn roll(input: &str) -> IResult<&str, Statement> {
    let (input, expr) = preceded(tag("roll"), preceded(sp, expression))(input)?;

    Ok((input, Statement::Roll(Box::new(expr))))
}

pub fn command(input: &str) -> IResult<&str, Statement> {
    preceded(char('!'), alt((roll, set_value, print_env)))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable() {
        assert_eq!(
            variable("test010").unwrap().1,
            Expression::Variable("test010".to_string())
        );
        assert_eq!(
            variable("another_one_again").unwrap().1,
            Expression::Variable("another_one_again".to_string())
        );
        assert_eq!(
            variable("!test_fails"),
            Err(Error(nom::error::Error {
                input: "!test_fails",
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
        assert_eq!(command("!print_env").unwrap().1, Statement::PrintEnv);
        assert_eq!(
            command("!roll 1").unwrap().1,
            Statement::Roll(Box::new(Expression::Integer(1)))
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
