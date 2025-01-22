use rand::Rng;

use crate::parser::{BinOp, Expr, Sides};

pub fn eval(tree: &Expr) -> Result<DiceRollResult, Box<dyn std::error::Error>> {
    match tree {
        Expr::Int(v) => Ok(DiceRollResult {
            result: *v as f64,
            explanation: v.to_string(),
        }),
        Expr::Float(v) => Ok(DiceRollResult {
            result: *v,
            explanation: v.to_string(),
        }),
        Expr::Additive {
            left,
            operator,
            right,
        } => eval_additive(left, operator, right),
        Expr::Multiplicative {
            left,
            operator,
            right,
        } => eval_multiplicative(left, operator, right),
        Expr::Roll { rolls, sides } => eval_roll(rolls, sides),
    }
}

fn eval_additive(
    left: &Expr,
    operator: &BinOp,
    right: &Expr,
) -> Result<DiceRollResult, Box<dyn std::error::Error>> {
    let DiceRollResult {
        result: left,
        explanation: left_explanation,
    } = eval(left)?;

    let DiceRollResult {
        result: right,
        explanation: right_explanation,
    } = eval(right)?;

    Ok(DiceRollResult {
        result: match operator {
            BinOp::Add => left + right,
            BinOp::Sub => left - right,
            other => unreachable!("{other:?} is not an additive operator"),
        },
        explanation: format!("{} {} {}", left_explanation, operator, right_explanation),
    })
}

fn eval_multiplicative(
    left: &Expr,
    operator: &BinOp,
    right: &Expr,
) -> Result<DiceRollResult, Box<dyn std::error::Error>> {
    let DiceRollResult {
        result: left,
        explanation: left_explanation,
    } = eval(left)?;

    let DiceRollResult {
        result: right,
        explanation: right_explanation,
    } = eval(right)?;

    Ok(DiceRollResult {
        result: match operator {
            BinOp::Mul => left * right,
            BinOp::Div => left / right,
            BinOp::Mod => left % right,
            other => unreachable!("{other:?} is not an additive operator"),
        },
        explanation: format!("{} {} {}", left_explanation, operator, right_explanation),
    })
}

fn eval_roll(rolls: &Expr, sides: &Sides) -> Result<DiceRollResult, Box<dyn std::error::Error>> {
    let DiceRollResult {
        result,
        explanation: _rolls_explanation,
    } = eval(rolls)?;

    dbg!(&result);
    dbg!(&_rolls_explanation);
    dbg!(&sides);

    let rolls = result.round() as i64;

    if rolls < 0 {
        return Err("Cannot roll a negative number of times".into());
    }

    let side_values = match sides {
        Sides::Expr(expr) => {
            let DiceRollResult {
                result,
                explanation: _,
            } = eval(expr)?;

            (1..result.round() as i64 + 1).collect()
        }
        Sides::Range { min, max } => {
            let DiceRollResult {
                result: min,
                explanation: _,
            } = eval(min)?;

            let min = min.round() as i64;

            let DiceRollResult {
                result: max,
                explanation: _,
            } = eval(max)?;

            let max = max.round() as i64;

            (min..=max).collect()
        }
        Sides::Values(values) => {
            let mut results = vec![];

            for value in values {
                let DiceRollResult {
                    result,
                    explanation: _,
                } = eval(value)?;

                results.push(result.round() as i64);
            }

            results
        }
        Sides::Fudge => (-1..=1).collect(),
    };

    let mut rng = rand::thread_rng();
    let mut results = vec![];

    for _ in 0..rolls {
        let len = side_values.len();

        if len == 0 {
            continue;
        }

        let index = rng.gen_range(0..len);
        results.push(side_values[index]);
    }

    let explanation = if _rolls_explanation.starts_with('[') {
        format!(
            "{}d{}: [{}]",
            result,
            {
                let explanation = sides.explain();

                if explanation.starts_with('[') {
                    format!("({})", explanation)
                } else {
                    explanation
                }
            },
            results
                .iter()
                .map(|r| r.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    } else {
        format!(
            "[{}]",
            results
                .iter()
                .map(|r| r.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    };

    Ok(DiceRollResult {
        result: results.iter().sum::<i64>() as f64,
        explanation,
    })
}

pub struct DiceRollResult {
    pub result: f64,
    pub explanation: String,
}
