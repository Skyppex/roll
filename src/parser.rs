use std::{error::Error, fmt::Display};

use crate::tokenizer::Token;

pub fn parse(tokens: &mut Vec<Token>) -> Result<Expr, Box<dyn Error>> {
    tokens.reverse(); // reverse so we can pop from the end
    parse_expr(tokens)
}

fn parse_expr(tokens: &mut Vec<Token>) -> Result<Expr, Box<dyn Error>> {
    parse_additive(tokens)
}

fn parse_additive(tokens: &mut Vec<Token>) -> Result<Expr, Box<dyn Error>> {
    let mut expr = parse_multiplicative(tokens)?;

    while let Some(token) = tokens.clone().pop() {
        match token {
            Token::Add => {
                tokens.pop();
                let right = parse_multiplicative(tokens)?;

                expr = Expr::Additive {
                    left: Box::new(expr),
                    operator: BinOp::Add,
                    right: Box::new(right),
                };
            }
            Token::Sub => {
                tokens.pop();
                let right = parse_multiplicative(tokens)?;

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

fn parse_multiplicative(tokens: &mut Vec<Token>) -> Result<Expr, Box<dyn Error>> {
    let mut expr = parse_roll(tokens)?;

    while let Some(token) = tokens.clone().pop() {
        match token {
            Token::Mul => {
                tokens.pop();
                let right = parse_roll(tokens)?;

                expr = Expr::Multiplicative {
                    left: Box::new(expr),
                    operator: BinOp::Mul,
                    right: Box::new(right),
                };
            }
            Token::Div => {
                tokens.pop();
                let right = parse_roll(tokens)?;

                expr = Expr::Multiplicative {
                    left: Box::new(expr),
                    operator: BinOp::Div,
                    right: Box::new(right),
                };
            }
            Token::Mod => {
                tokens.pop();
                let right = parse_roll(tokens)?;

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

fn parse_roll(tokens: &mut Vec<Token>) -> Result<Expr, Box<dyn Error>> {
    let rolls = if tokens.clone().pop() != Some(Token::D) {
        Some(parse_primary(tokens)?)
    } else {
        None
    };

    let rolls = match (rolls, tokens.clone().pop()) {
        (Some(Expr::Float(v)), _) => return Ok(Expr::Float(v)),
        (Some(t), Some(Token::D)) => {
            tokens.pop();
            t
        }
        (Some(t), _) => return Ok(t),
        (None, _) => {
            tokens.pop();
            Expr::Int(1)
        }
    };

    let sides = parse_sides(tokens)?;
    let modifiers = parse_modifiers(tokens)?;

    Ok(Expr::Roll {
        rolls: Box::new(rolls),
        sides,
        modifiers,
    })
}

fn parse_sides(tokens: &mut Vec<Token>) -> Result<Sides, Box<dyn Error>> {
    match tokens.pop() {
        Some(Token::Int(value)) => Ok(Sides::Expr(Box::new(Expr::Int(value)))),
        Some(Token::Float(_)) => Err("Cannot use float for number of sides".into()),
        Some(Token::OpenBracket) => {
            let value = parse_expr(tokens)?;

            if !matches!(
                (tokens.clone().pop(), tokens.clone().pop()),
                (Some(Token::Comma), Some(_)) | (Some(Token::Dot), Some(Token::Dot))
            ) {
                return Err("Expected ',' or '..'".into());
            }

            match tokens.pop() {
                Some(Token::Comma) => {
                    let mut values = vec![value];

                    while tokens.clone().pop() != Some(Token::Comma) {
                        values.push(parse_expr(tokens)?);
                    }

                    if tokens.pop() != Some(Token::CloseBracket) {
                        return Err("Expected ']'".into());
                    }

                    Ok(Sides::Values(values))
                }
                Some(Token::Dot) => {
                    tokens.pop(); // pop the second dot

                    let min = value;
                    let max = parse_expr(tokens)?;

                    if tokens.pop() != Some(Token::CloseBracket) {
                        return Err("Expected ']'".into());
                    }

                    Ok(Sides::Range {
                        min: Box::new(min),
                        max: Box::new(max),
                    })
                }
                _ => Err("Expected ',' or '..'".into()),
            }
        }
        Some(Token::OpenParen) => {
            let sides = Sides::Expr(Box::new(parse_expr(tokens)?));

            if tokens.pop() != Some(Token::CloseParen) {
                return Err("Expected closing parenthesis".into());
            }

            Ok(sides)
        }
        Some(Token::F) => Ok(Sides::Fudge),
        _ => Err("Expected sides expression".into()),
    }
}

fn parse_modifiers(tokens: &mut Vec<Token>) -> Result<Vec<Modifier>, Box<dyn Error>> {
    let mut modifiers = vec![];

    let mut clone = tokens.clone();
    let mut first = clone.pop();
    let mut second = clone.pop();

    loop {
        match (&first, &second) {
            (Some(Token::K), Some(Token::L)) => {
                tokens.pop();
                tokens.pop();

                if !matches!(tokens.clone().pop(), Some(Token::Int(_) | Token::OpenParen)) {
                    modifiers.push(Modifier::KeepLowest(Box::new(Expr::Int(1))));

                    let mut clone = tokens.clone();
                    first = clone.pop();
                    second = clone.pop();
                    continue;
                }

                let value = parse_primary(tokens)?;
                modifiers.push(Modifier::KeepLowest(Box::new(value)));

                let mut clone = tokens.clone();
                first = clone.pop();
                second = clone.pop();
            }
            (Some(Token::K), _) => {
                tokens.pop();

                if tokens.clone().pop() != Some(Token::H) {
                    tokens.pop();
                }

                if !matches!(tokens.clone().pop(), Some(Token::Int(_) | Token::OpenParen)) {
                    modifiers.push(Modifier::KeepHighest(Box::new(Expr::Int(1))));

                    let mut clone = tokens.clone();
                    first = clone.pop();
                    second = clone.pop();
                    continue;
                }

                let value = parse_primary(tokens)?;
                modifiers.push(Modifier::KeepHighest(Box::new(value)));

                let mut clone = tokens.clone();
                first = clone.pop();
                second = clone.pop();
            }
            (Some(Token::D), Some(Token::H)) => {
                tokens.pop();
                tokens.pop();

                if !matches!(tokens.clone().pop(), Some(Token::Int(_) | Token::OpenParen)) {
                    modifiers.push(Modifier::DropHighest(Box::new(Expr::Int(1))));

                    let mut clone = tokens.clone();
                    first = clone.pop();
                    second = clone.pop();
                    continue;
                }

                let value = parse_primary(tokens)?;
                modifiers.push(Modifier::DropHighest(Box::new(value)));

                let mut clone = tokens.clone();
                first = clone.pop();
                second = clone.pop();
            }
            (Some(Token::D), _) => {
                tokens.pop();

                if !matches!(tokens.clone().pop(), Some(Token::Int(_) | Token::OpenParen)) {
                    modifiers.push(Modifier::DropLowest(Box::new(Expr::Int(1))));

                    let mut clone = tokens.clone();
                    first = clone.pop();
                    second = clone.pop();
                    continue;
                }

                let value = parse_primary(tokens)?;
                modifiers.push(Modifier::DropLowest(Box::new(value)));

                let mut clone = tokens.clone();
                first = clone.pop();
                second = clone.pop();
            }
            (Some(Token::R), _) => {
                tokens.pop();

                if !matches!(tokens.clone().pop(), Some(Token::Int(_) | Token::OpenParen)) {
                    modifiers.push(Modifier::Reroll(Box::new(Expr::Int(1))));

                    let mut clone = tokens.clone();
                    first = clone.pop();
                    second = clone.pop();
                    continue;
                }

                let value = parse_primary(tokens)?;
                modifiers.push(Modifier::Reroll(Box::new(value)));

                let mut clone = tokens.clone();
                first = clone.pop();
                second = clone.pop();
            }
            (Some(Token::Exclamation), _) => {
                tokens.pop();

                if !matches!(tokens.clone().pop(), Some(Token::Int(_) | Token::OpenParen)) {
                    modifiers.push(Modifier::Explode(Box::new(Expr::Int(1))));

                    let mut clone = tokens.clone();
                    first = clone.pop();
                    second = clone.pop();
                    continue;
                }

                let value = parse_primary(tokens)?;
                modifiers.push(Modifier::Explode(Box::new(value)));

                let mut clone = tokens.clone();
                first = clone.pop();
                second = clone.pop();
            }
            _ => break,
        }
    }

    Ok(modifiers)
}

fn parse_primary(tokens: &mut Vec<Token>) -> Result<Expr, Box<dyn Error>> {
    match tokens.pop() {
        Some(Token::Int(value)) => Ok(Expr::Int(value)),
        Some(Token::Float(value)) => Ok(Expr::Float(value)),
        Some(Token::OpenParen) => {
            let expr = parse_expr(tokens)?;

            if tokens.pop() != Some(Token::CloseParen) {
                return Err("Expected closing parenthesis".into());
            }

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
