mod cursor;
pub use cursor::*;

use std::{error::Error, fmt::Display};

use crate::lexer::Token;

pub fn parse(cursor: &mut Cursor) -> Result<Expr, Box<dyn Error>> {
    parse_expr(cursor)
}

fn parse_expr(cursor: &mut Cursor) -> Result<Expr, Box<dyn Error>> {
    parse_additive(cursor)
}

fn parse_additive(cursor: &mut Cursor) -> Result<Expr, Box<dyn Error>> {
    let mut expr = parse_multiplicative(cursor)?;

    while let Some(token) = cursor.first() {
        match token {
            Token::Add => {
                cursor.bump();
                let right = parse_multiplicative(cursor)?;

                expr = Expr::Additive {
                    left: Box::new(expr),
                    operator: BinOp::Add,
                    right: Box::new(right),
                };
            }
            Token::Sub => {
                cursor.bump();
                let right = parse_multiplicative(cursor)?;

                expr = Expr::Additive {
                    left: Box::new(expr),
                    operator: BinOp::Sub,
                    right: Box::new(right),
                };
            }
            _ => break,
        }
    }

    Ok(expr)
}

fn parse_multiplicative(cursor: &mut Cursor) -> Result<Expr, Box<dyn Error>> {
    let mut expr = parse_roll(cursor)?;

    while let Some(token) = cursor.first() {
        match token {
            Token::Mul => {
                cursor.bump();
                let right = parse_roll(cursor)?;

                expr = Expr::Multiplicative {
                    left: Box::new(expr),
                    operator: BinOp::Mul,
                    right: Box::new(right),
                };
            }
            Token::Div => {
                cursor.bump();
                let right = parse_roll(cursor)?;

                expr = Expr::Multiplicative {
                    left: Box::new(expr),
                    operator: BinOp::Div,
                    right: Box::new(right),
                };
            }
            Token::Mod => {
                cursor.bump();
                let right = parse_roll(cursor)?;

                expr = Expr::Multiplicative {
                    left: Box::new(expr),
                    operator: BinOp::Mod,
                    right: Box::new(right),
                };
            }
            _ => break,
        }
    }

    Ok(expr)
}

fn parse_roll(cursor: &mut Cursor) -> Result<Expr, Box<dyn Error>> {
    let rolls = if cursor.first() != Some(Token::D) {
        Some(parse_primary(cursor)?)
    } else {
        None
    };

    let rolls = match (rolls, cursor.first()) {
        (Some(Expr::Float(v)), _) => return Ok(Expr::Float(v)),
        (Some(t), Some(Token::D)) => {
            cursor.bump();
            t
        }
        (Some(t), _) => return Ok(t),
        (None, _) => {
            cursor.bump();
            Expr::Int(1)
        }
    };

    let sides = parse_sides(cursor)?;
    let modifiers = parse_modifiers(cursor)?;

    Ok(Expr::Roll {
        rolls: Box::new(rolls),
        sides,
        modifiers,
    })
}

fn parse_sides(cursor: &mut Cursor) -> Result<Sides, Box<dyn Error>> {
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

            match cursor.bump() {
                Some(Token::Comma) => {
                    let mut values = vec![value];

                    while cursor.first() != Some(Token::Comma) {
                        values.push(parse_expr(cursor)?);
                    }

                    cursor.expect(Token::CloseBracket)?;

                    Ok(Sides::Values(values))
                }
                Some(Token::Dot) => {
                    cursor.bump(); // pop the second dot

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

fn parse_modifiers(cursor: &mut Cursor) -> Result<Vec<Modifier>, Box<dyn Error>> {
    let mut modifiers = vec![];

    loop {
        match (cursor.first(), cursor.second()) {
            (Some(Token::K), Some(Token::L)) => {
                cursor.bump();
                cursor.bump();

                if !matches!(cursor.first(), Some(Token::Int(_) | Token::OpenParen)) {
                    modifiers.push(Modifier::KeepLowest(Box::new(Expr::Int(1))));
                    continue;
                }

                let value = parse_primary(cursor)?;
                modifiers.push(Modifier::KeepLowest(Box::new(value)));
            }
            (Some(Token::K), _) => {
                cursor.bump();

                if cursor.first() != Some(Token::H) {
                    cursor.bump();
                }

                if !matches!(cursor.first(), Some(Token::Int(_) | Token::OpenParen)) {
                    modifiers.push(Modifier::KeepHighest(Box::new(Expr::Int(1))));
                    continue;
                }

                let value = parse_primary(cursor)?;
                modifiers.push(Modifier::KeepHighest(Box::new(value)));
            }
            (Some(Token::D), Some(Token::H)) => {
                cursor.bump();
                cursor.bump();

                if !matches!(cursor.first(), Some(Token::Int(_) | Token::OpenParen)) {
                    modifiers.push(Modifier::DropHighest(Box::new(Expr::Int(1))));
                    continue;
                }

                let value = parse_primary(cursor)?;
                modifiers.push(Modifier::DropHighest(Box::new(value)));
            }
            (Some(Token::D), _) => {
                cursor.bump();

                if !matches!(cursor.first(), Some(Token::Int(_) | Token::OpenParen)) {
                    modifiers.push(Modifier::DropLowest(Box::new(Expr::Int(1))));
                    continue;
                }

                let value = parse_primary(cursor)?;
                modifiers.push(Modifier::DropLowest(Box::new(value)));
            }
            (Some(Token::R), _) => {
                cursor.bump();

                if !matches!(cursor.first(), Some(Token::Int(_) | Token::OpenParen)) {
                    modifiers.push(Modifier::Reroll(Box::new(Expr::Int(1))));
                    continue;
                }

                let value = parse_primary(cursor)?;
                modifiers.push(Modifier::Reroll(Box::new(value)));
            }
            (Some(Token::Exclamation), _) => {
                cursor.bump();

                if !matches!(cursor.first(), Some(Token::Int(_) | Token::OpenParen)) {
                    modifiers.push(Modifier::Explode(Box::new(Expr::Int(1))));
                    continue;
                }

                let value = parse_primary(cursor)?;
                modifiers.push(Modifier::Explode(Box::new(value)));
            }
            _ => break,
        }
    }

    Ok(modifiers)
}

fn parse_primary(cursor: &mut Cursor) -> Result<Expr, Box<dyn Error>> {
    match cursor.bump() {
        Some(Token::Int(value)) => Ok(Expr::Int(value)),
        Some(Token::Float(value)) => Ok(Expr::Float(value)),
        Some(Token::OpenParen) => {
            let expr = parse_expr(cursor)?;
            cursor.expect(Token::CloseParen)?;
            Ok(expr)
        }
        _ => Err("Expected primary expression".into()),
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Int(i64),
    Float(f64),
    Additive {
        left: Box<Expr>,
        operator: BinOp,
        right: Box<Expr>,
    },
    Multiplicative {
        left: Box<Expr>,
        operator: BinOp,
        right: Box<Expr>,
    },
    Roll {
        rolls: Box<Expr>,
        sides: Sides,
        modifiers: Vec<Modifier>,
    },
}

#[derive(Debug, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

impl Display for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string_representation = match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Mod => "%",
        };

        write!(f, "{string_representation}")
    }
}

#[derive(Debug, Clone)]
pub enum Sides {
    Expr(Box<Expr>),
    Range { min: Box<Expr>, max: Box<Expr> },
    Values(Vec<Expr>),
    Fudge,
}

#[derive(Debug, Clone)]
pub enum Modifier {
    KeepHighest(Box<Expr>),
    KeepLowest(Box<Expr>),
    DropHighest(Box<Expr>),
    DropLowest(Box<Expr>),
    Reroll(Box<Expr>),
    Explode(Box<Expr>),
}
