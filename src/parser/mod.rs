mod cursor;
mod roll_parser;

pub use cursor::Cursor;

use std::fmt::Display;

use crate::{lexer::Token, program::DynError};

pub fn parse(cursor: &mut Cursor) -> Result<Expr, DynError> {
    parse_expr(cursor)
}

fn parse_expr(cursor: &mut Cursor) -> Result<Expr, DynError> {
    parse_additive(cursor)
}

fn parse_additive(cursor: &mut Cursor) -> Result<Expr, DynError> {
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

fn parse_multiplicative(cursor: &mut Cursor) -> Result<Expr, DynError> {
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

fn parse_roll(cursor: &mut Cursor) -> Result<Expr, DynError> {
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

    roll_parser::parse(rolls, cursor)
}

fn parse_primary(cursor: &mut Cursor) -> Result<Expr, DynError> {
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
    Reroll {
        amount: Box<Expr>,
        condition: Option<Condition>,
    },
    Explode {
        amount: Box<Expr>,
        condition: Option<Condition>,
    },
}

#[derive(Debug, Clone)]
pub struct Condition {
    pub operator: RelOp,
    pub value: Box<Expr>,
}

impl Condition {
    pub fn new(operator: RelOp, value: Expr) -> Self {
        Self {
            operator,
            value: Box::new(value),
        }
    }
}

#[derive(Debug, Clone)]
pub enum RelOp {
    Equals,
    NotEquals,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}
