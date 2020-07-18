use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::{char},
    combinator::map_res,
    multi::many0,
    sequence::{preceded, tuple},
};

use crate::types::Expression;

fn from_decimal(input: &str) -> Result<u64, std::num::ParseIntError> {
    u64::from_str_radix(input, 10)
}

fn is_digit(c: char) -> bool {
    c.is_digit(10)
}

fn sp(input: &str) -> IResult<&str, &str> {
    let chars = " \t\r\n";

    take_while(move |c| chars.contains(c))(input)
}

fn integer(input: &str) -> IResult<&str, Expression> {
    let (input, number) = map_res(
        take_while(is_digit),
        from_decimal
    )(input)?;

    Ok((input, Expression::Integer(number)))
}

fn term(input: &str) -> IResult<&str, Expression> {
    let (input, (expr1, exprs)) = tuple((
        alt((dice_roll, integer)),
        many0(tuple((preceded(sp, alt((char('+'), char('-')))),
                     preceded(sp, alt((dice_roll, integer)))))),
    ))(input)?;

    Ok((input, exprs.into_iter().fold(expr1, parse_term)))
}

fn parse_term(left_expr: Expression, next: (char, Expression)) -> Expression {
    let (op, right_expr) = next;

    Expression::Term(Box::new(left_expr), Box::new(right_expr), op)
}

fn dice_roll(input: &str) -> IResult<&str, Expression> {
    let (input, (count, _, sides)) = tuple((integer, char('d'), integer))(input)?;

    Ok((input, Expression::DiceRoll {
        count: Box::new(count),
        sides: Box::new(sides)
    }))
}

pub fn roll(input: &str) -> IResult<&str, Expression> {
    let (input, expr) = preceded(
        tag("!roll"),
        preceded(sp, alt((term, dice_roll, integer)))
    )(input)?;

    Ok((input, Expression::Roll(Box::new(expr))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::{
        Err::Error,
        error::ErrorKind,
    };

    #[test]
    fn test_term() {
        assert_eq!(term("1 - 2").unwrap().1, Expression::Term(
            Box::new(Expression::Integer(1)),
            Box::new(Expression::Integer(2)),
            '-'
        ));
        assert_eq!(term("1 + 2").unwrap().1, Expression::Term(
            Box::new(Expression::Integer(1)),
            Box::new(Expression::Integer(2)),
            '+'
        ));
        assert_eq!(term("2d6 + 1").unwrap().1, Expression::Term(
            Box::new(Expression::DiceRoll {
                count: Box::new(Expression::Integer(2)),
                sides: Box::new(Expression::Integer(6))
            }),
            Box::new(Expression::Integer(1)),
            '+'
        ));
        assert_eq!(term("2d6 + 1 + 1d6 - 10").unwrap().1, Expression::Term(
            Box::new(Expression::Term(
                Box::new(Expression::Term(
                    Box::new(Expression::DiceRoll {
                        count: Box::new(Expression::Integer(2)),
                        sides: Box::new(Expression::Integer(6))
                    }),
                    Box::new(Expression::Integer(1)),
                    '+'
                )),
                Box::new(Expression::DiceRoll {
                    count: Box::new(Expression::Integer(1)),
                    sides: Box::new(Expression::Integer(6))
                }),
                '+'
            )),
            Box::new(Expression::Integer(10)),
            '-'
        ));
    }

    #[test]
    fn test_integer() {
        assert_eq!(integer("1"), Ok(("", Expression::Integer(1))));
        assert_eq!(integer("2"), Ok(("", Expression::Integer(2))));
        assert_eq!(integer("f"), Err(Error(("f", ErrorKind::MapRes))));
    }

    #[test]
    fn test_dice_roll() {
        assert_eq!(dice_roll("1d6"), Ok(("", Expression::DiceRoll {
            count: Box::new(Expression::Integer(1)),
            sides: Box::new(Expression::Integer(6))
        })));
        assert_eq!(dice_roll("1231239d410123123"), Ok(("", Expression::DiceRoll {
            count: Box::new(Expression::Integer(1231239)),
            sides: Box::new(Expression::Integer(410123123))
        })));
        assert_eq!(dice_roll("69d420"), Ok(("", Expression::DiceRoll {
            count: Box::new(Expression::Integer(69)),
            sides: Box::new(Expression::Integer(420))
        })));
        assert_eq!(dice_roll("2d8"), Ok(("", Expression::DiceRoll {
            count: Box::new(Expression::Integer(2)),
            sides: Box::new(Expression::Integer(8))
        })));
        assert_eq!(dice_roll("x9d420"), Err(Error(("x9d420", ErrorKind::MapRes))))
    }

    #[test]
    fn test_roll() {
        assert_eq!(roll("!roll 1").unwrap().1, Expression::Roll(
            Box::new(Expression::Integer(1))
        ));
        assert_eq!(roll("!roll 1d6").unwrap().1, Expression::Roll(
            Box::new(Expression::DiceRoll {
                count: Box::new(Expression::Integer(1)),
                sides: Box::new(Expression::Integer(6))
            })
        ));
        assert_eq!(roll("!roll 1d6 + 1").unwrap().1, Expression::Roll(
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
        ));
    }
}
