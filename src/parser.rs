use std::error::Error;

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
    let rolls = parse_primary(tokens)?;

    if let Expr::Float(_) = rolls {
        return Err("Expected integer value for number of rolls".into());
    }

    if tokens.clone().pop() != Some(Token::D) {
        return Ok(rolls);
    }

    tokens.pop(); // pop the 'd'

    let sides = parse_sides(tokens)?;

    Ok(Expr::Roll {
        rolls: Box::new(rolls),
        sides,
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

fn parse_primary(tokens: &mut Vec<Token>) -> Result<Expr, Box<dyn Error>> {
    // int, float, parenthesized expr

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

#[derive(Debug, Clone)]
pub enum Sides {
    Expr(Box<Expr>),
    Range { min: Box<Expr>, max: Box<Expr> },
    Values(Vec<Expr>),
    Fudge,
}
