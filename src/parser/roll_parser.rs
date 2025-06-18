use crate::{lexer::Token, program::DynError};

use super::{cursor::Cursor, parse_expr, parse_primary, Condition, Expr, Modifier, RelOp, Sides};

pub fn parse(rolls: Expr, cursor: &mut Cursor) -> Result<Expr, DynError> {
    let sides = parse_sides(cursor)?;
    let modifiers = parse_modifiers(cursor)?;

    Ok(Expr::Roll {
        rolls: Box::new(rolls),
        sides,
        modifiers,
    })
}

fn parse_sides(cursor: &mut Cursor) -> Result<Sides, DynError> {
    match cursor.bump() {
        Some(Token::Int(value)) => Ok(Sides::Expr(Box::new(Expr::Int(value)))),
        Some(Token::Float(_)) => Err("Cannot use float for number of sides".into()),
        Some(Token::OpenBracket) => {
            let value = parse_expr(cursor)?;

            if !matches!(
                (cursor.first(), cursor.second()),
                (Some(Token::Comma), Some(_)) | (Some(Token::Dot), Some(Token::Dot))
            ) {
                return Err("Expected ',' or '..'".into());
            }

            match cursor.first() {
                Some(Token::Comma) => {
                    let mut values = vec![value];

                    while cursor.first() == Some(Token::Comma) {
                        cursor.bump();
                        values.push(parse_expr(cursor)?);
                    }

                    cursor.expect(Token::CloseBracket)?;

                    Ok(Sides::Values(values))
                }
                Some(Token::Dot) => {
                    cursor.bump(); // pop the first dot
                    cursor.expect(Token::Dot)?; // expect the second dot

                    let min = value;
                    let max = parse_expr(cursor)?;

                    cursor.expect(Token::CloseBracket)?;

                    Ok(Sides::Range {
                        min: Box::new(min),
                        max: Box::new(max),
                    })
                }
                _ => Err("Expected ',' or '..'".into()),
            }
        }
        Some(Token::OpenParen) => {
            let sides = Sides::Expr(Box::new(parse_expr(cursor)?));
            cursor.expect(Token::CloseParen)?;
            Ok(sides)
        }
        Some(Token::F) => Ok(Sides::Fudge),
        _ => Err("Expected sides expression".into()),
    }
}

fn parse_modifiers(cursor: &mut Cursor) -> Result<Vec<Modifier>, DynError> {
    let mut modifiers = vec![];

    loop {
        match (cursor.first(), cursor.second()) {
            (Some(Token::K), Some(Token::L)) => {
                cursor.bump();
                cursor.bump();

                let mut amount = Expr::Int(1);

                if matches!(cursor.first(), Some(Token::Int(_) | Token::OpenParen)) {
                    amount = parse_primary(cursor)?;
                }

                modifiers.push(Modifier::KeepLowest(Box::new(amount)));
            }
            (Some(Token::K), _) => {
                cursor.bump();

                if cursor.first() == Some(Token::H) {
                    cursor.bump();
                }

                let mut amount = Expr::Int(1);

                if matches!(cursor.first(), Some(Token::Int(_) | Token::OpenParen)) {
                    amount = parse_primary(cursor)?;
                }

                modifiers.push(Modifier::KeepHighest(Box::new(amount)));
            }
            (Some(Token::D), Some(Token::H)) => {
                cursor.bump();
                cursor.bump();

                let mut amount = Expr::Int(1);

                if matches!(cursor.first(), Some(Token::Int(_) | Token::OpenParen)) {
                    amount = parse_primary(cursor)?;
                }

                modifiers.push(Modifier::DropHighest(Box::new(amount)));
            }
            (Some(Token::D), _) => {
                cursor.bump();

                if cursor.first() == Some(Token::L) {
                    cursor.bump();
                }

                let mut amount = Expr::Int(1);

                if matches!(cursor.first(), Some(Token::Int(_) | Token::OpenParen)) {
                    amount = parse_primary(cursor)?;
                }

                modifiers.push(Modifier::DropLowest(Box::new(amount)));
            }
            (Some(Token::R), _) => {
                cursor.bump();

                let mut amount = Expr::Int(1);

                if matches!(cursor.first(), Some(Token::Int(_) | Token::OpenParen)) {
                    amount = parse_primary(cursor)?;
                }

                let condition = parse_condition(cursor)?;

                modifiers.push(Modifier::Reroll {
                    amount: Box::new(amount),
                    condition,
                });
            }
            (Some(Token::Exclamation), _) => {
                cursor.bump();

                let mut amount = Expr::Int(1);

                if matches!(cursor.first(), Some(Token::Int(_) | Token::OpenParen)) {
                    amount = parse_primary(cursor)?;
                }

                let condition = parse_condition(cursor)?;

                modifiers.push(Modifier::Explode {
                    amount: Box::new(amount),
                    condition,
                });
            }
            _ => break,
        }
    }

    Ok(modifiers)
}

fn parse_condition(cursor: &mut Cursor) -> Result<Option<Condition>, DynError> {
    match (cursor.first(), cursor.second()) {
        (Some(Token::Less), Some(Token::Equals)) => {
            cursor.bump();
            cursor.bump();
            let value = parse_expr(cursor)?;
            Ok(Some(Condition::new(RelOp::LessEqual, value)))
        }
        (Some(Token::Greater), Some(Token::Equals)) => {
            cursor.bump();
            cursor.bump();
            let value = parse_expr(cursor)?;
            Ok(Some(Condition::new(RelOp::GreaterEqual, value)))
        }
        (Some(Token::Tilde), Some(Token::Equals)) => {
            cursor.bump();
            cursor.bump();
            let value = parse_expr(cursor)?;
            Ok(Some(Condition::new(RelOp::NotEquals, value)))
        }
        (Some(Token::Less), Some(_)) => {
            cursor.bump();
            let value = parse_expr(cursor)?;
            Ok(Some(Condition::new(RelOp::Less, value)))
        }
        (Some(Token::Greater), Some(_)) => {
            cursor.bump();
            let value = parse_expr(cursor)?;
            Ok(Some(Condition::new(RelOp::Greater, value)))
        }
        (Some(Token::Equals), Some(_)) => {
            cursor.bump();
            let value = parse_expr(cursor)?;
            Ok(Some(Condition::new(RelOp::Equals, value)))
        }
        _ => Ok(None),
    }
}
