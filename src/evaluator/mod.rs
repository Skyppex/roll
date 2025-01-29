mod mode;

use mode::Mode;
use rand::Rng;

use crate::{
    cli::Cli,
    parser::{BinOp, Condition, Expr, Modifier, RelOp, Sides},
    program::DynError,
};

pub fn eval(tree: &Expr, cli: &Cli) -> Result<EvalResult, DynError> {
    match tree {
        Expr::Int(v) => Ok(EvalResult {
            result: *v as f64,
            explanation: v.to_string(),
            is_roll: false,
        }),
        Expr::Float(v) => Ok(EvalResult {
            result: *v,
            explanation: v.to_string(),
            is_roll: false,
        }),
        Expr::Additive {
            left,
            operator,
            right,
        } => eval_additive(left, operator, right, cli),
        Expr::Multiplicative {
            left,
            operator,
            right,
        } => eval_multiplicative(left, operator, right, cli),
        Expr::Roll {
            rolls,
            sides,
            modifiers,
        } => eval_roll(rolls, sides, modifiers, cli),
    }
}

fn eval_additive(
    left: &Expr,
    operator: &BinOp,
    right: &Expr,
    cli: &Cli,
) -> Result<EvalResult, DynError> {
    let EvalResult {
        result: left,
        explanation: left_explanation,
        ..
    } = eval(left, cli)?;

    let EvalResult {
        result: right,
        explanation: right_explanation,
        ..
    } = eval(right, cli)?;

    Ok(EvalResult {
        result: match operator {
            BinOp::Add => left + right,
            BinOp::Sub => left - right,
            other => unreachable!("{other:?} is not an additive operator"),
        },
        explanation: format!("{} {} {}", left_explanation, operator, right_explanation),
        is_roll: false,
    })
}

fn eval_multiplicative(
    left: &Expr,
    operator: &BinOp,
    right: &Expr,
    cli: &Cli,
) -> Result<EvalResult, DynError> {
    let EvalResult {
        result: left,
        explanation: left_explanation,
        ..
    } = eval(left, cli)?;

    let EvalResult {
        result: right,
        explanation: right_explanation,
        ..
    } = eval(right, cli)?;

    Ok(EvalResult {
        result: match operator {
            BinOp::Mul => left * right,
            BinOp::Div => left / right,
            BinOp::Mod => left % right,
            other => unreachable!("{other:?} is not an additive operator"),
        },
        explanation: format!("{} {} {}", left_explanation, operator, right_explanation),
        is_roll: false,
    })
}

fn eval_roll(
    rolls: &Expr,
    sides: &Sides,
    modifiers: &Vec<Modifier>,
    cli: &Cli,
) -> Result<EvalResult, DynError> {
    let EvalResult {
        result,
        explanation: rolls_explanation,
        is_roll: rolls_explanation_is_roll,
    } = eval(rolls, cli)?;

    let rolls = result.round() as i64;

    if rolls < 0 {
        return Err("Cannot roll a negative number of times".into());
    }

    let (side_values, sides_explanation, sides_explanation_is_roll, is_fudge): (Vec<i64>, _, _, _) =
        match sides {
            Sides::Expr(expr) => {
                let EvalResult {
                    result,
                    explanation,
                    is_roll,
                } = eval(expr, cli)?;

                (
                    (1..result.round() as i64 + 1).collect(),
                    explanation,
                    is_roll,
                    false,
                )
            }
            Sides::Range { min, max } => {
                let EvalResult {
                    result: min,
                    explanation: min_explanation,
                    ..
                } = eval(min, cli)?;

                let min = min.round() as i64;

                let EvalResult {
                    result: max,
                    explanation: max_explanation,
                    ..
                } = eval(max, cli)?;

                let max = max.round() as i64;

                (
                    (min..=max).collect(),
                    format!("{}..{}", min_explanation, max_explanation),
                    false,
                    false,
                )
            }
            Sides::Values(values) => {
                let mut results = vec![];

                for value in values {
                    let EvalResult {
                        result,
                        explanation,
                        ..
                    } = eval(value, cli)?;

                    results.push((result.round() as i64, explanation));
                }

                (
                    results.iter().map(|r| r.0).collect(),
                    results
                        .into_iter()
                        .map(|r| r.1)
                        .collect::<Vec<_>>()
                        .join(", ")
                        .to_string(),
                    false,
                    false,
                )
            }
            Sides::Fudge => ((-1..=1).collect(), "f".to_string(), false, true),
        };

    let mut results = cli.mode.eval(rolls, &side_values)?;

    let rng = &mut rand::thread_rng();

    for modifier in modifiers {
        match modifier {
            Modifier::KeepHighest(expr) => {
                let EvalResult { result, .. } = eval(expr, cli)?;

                let value = result.round() as i64;

                if value < 0 {
                    return Err("Cannot keep a negative number of dice".into());
                }

                if value > rolls {
                    return Err("Cannot keep more dice than rolled".into());
                }

                results.sort_by(|a, b| a.sum().partial_cmp(&b.sum()).expect("Cannot compare"));

                for i in 0..(results.len() - value as usize) {
                    results[i].drop();
                }

                results.reverse();
            }
            Modifier::KeepLowest(expr) => {
                let EvalResult { result, .. } = eval(expr, cli)?;

                let value = result.round() as i64;

                if value < 0 {
                    return Err("Cannot keep a negative number of dice".into());
                }

                if value > rolls {
                    return Err("Cannot keep more dice than rolled".into());
                }

                results.sort_by(|a, b| b.sum().partial_cmp(&a.sum()).expect("Cannot compare"));

                for i in 0..(results.len() - value as usize) {
                    results[i].drop()
                }
            }
            Modifier::DropHighest(expr) => {
                let EvalResult { result, .. } = eval(expr, cli)?;

                let value = result.round() as i64;

                if value < 0 {
                    return Err("Cannot drop a negative number of dice".into());
                }

                if value > rolls {
                    return Err("Cannot drop more dice than rolled".into());
                }

                results.sort_by(|a, b| b.sum().partial_cmp(&a.sum()).expect("Cannot compare"));

                (0..value as usize).for_each(|i| results[i].drop());
            }
            Modifier::DropLowest(expr) => {
                let EvalResult { result, .. } = eval(expr, cli)?;

                let value = result.round() as i64;

                if value < 0 {
                    return Err("Cannot drop a negative number of dice".into());
                }

                if value > rolls {
                    return Err("Cannot drop more dice than rolled".into());
                }

                results.sort_by(|a, b| a.sum().partial_cmp(&b.sum()).expect("Cannot compare"));

                (0..value as usize).for_each(|i| results[i].drop());

                results.reverse();
            }
            Modifier::Reroll { amount, condition } => {
                let EvalResult { result, .. } = eval(amount, cli)?;

                let value = result.round() as i64;

                if value < 0 {
                    return Err("Cannot reroll a negative number of times".into());
                }

                let condition = if let Some(c) = condition {
                    let Condition { operator, value } = c;

                    Some((operator, eval(value, cli)?))
                } else {
                    None
                };

                for result in results.iter_mut() {
                    for _ in 0..value {
                        if let Some((operator, ref condition_value)) = condition {
                            if !rel_op_eval(operator, result, condition_value)? {
                                continue;
                            }
                        } else if result.sum() > result.min_side() as f64 {
                            continue;
                        }

                        let len = side_values.len();

                        if len == 0 {
                            continue;
                        }

                        let index = rng.gen_range(0..len);
                        result.reroll(side_values[index] as f64);
                    }
                }
            }
            Modifier::Explode { amount, condition } => {
                let EvalResult { result, .. } = eval(amount, cli)?;

                let value = result.round() as i64;

                if value < 0 {
                    return Err("Cannot reroll a negative number of times".into());
                }

                let condition = if let Some(c) = condition {
                    let Condition { operator, value } = c;

                    Some((operator, eval(value, cli)?))
                } else {
                    None
                };

                for result in results.iter_mut() {
                    for _ in 0..value {
                        if let Some((operator, ref condition_value)) = condition {
                            if !rel_op_eval(operator, result, condition_value)? {
                                continue;
                            }
                        } else if result.last() < result.max_side() as f64 {
                            continue;
                        }

                        let len = side_values.len();

                        if len == 0 {
                            continue;
                        }

                        let index = rng.gen_range(0..len);
                        result.explode(side_values[index] as f64);
                    }
                }
            }
        }
    }

    let mut results_explanation = String::new();

    for (i, result) in results.iter().enumerate() {
        results_explanation.push_str(to_fudge(&result.explain(), is_fudge)?.as_str());

        if i < results.len() - 1 {
            results_explanation.push_str(", ");
        }
    }

    let explanation = match (rolls_explanation_is_roll, sides_explanation_is_roll) {
        (true, true) => format!(
            "({})d({}): [{}]",
            rolls_explanation, sides_explanation, results_explanation,
        ),
        (true, false) => format!(
            "({})d{}: [{}]",
            rolls_explanation, sides_explanation, results_explanation,
        ),
        (false, true) => format!(
            "{}d({}): [{}]",
            rolls_explanation, sides_explanation, results_explanation,
        ),
        (false, false) => format!("[{}]", results_explanation),
    };

    Ok(EvalResult {
        result: results.iter().map(|r| r.sum()).sum::<f64>(),
        explanation,
        is_roll: true,
    })
}

fn to_fudge(roll_str: &str, is_fudge: bool) -> Result<String, DynError> {
    if !is_fudge {
        return Ok(roll_str.to_string());
    }

    match roll_str {
        "-1" => Ok("-".to_string()),
        "1" => Ok("+".to_string()),
        "0" => Ok("o".to_string()),
        _ => Err("Invalid fudge value".into()),
    }
}

fn rel_op_eval(operator: &RelOp, left: &DiceRolls, right: &EvalResult) -> Result<bool, DynError> {
    let left = left.last();
    let right = right.result;

    Ok(match operator {
        RelOp::Equals => left == right,
        RelOp::NotEquals => left != right,
        RelOp::Greater => left > right,
        RelOp::GreaterEqual => left >= right,
        RelOp::Less => left < right,
        RelOp::LessEqual => left <= right,
    })
}

#[derive(Debug, Clone)]
pub struct EvalResult {
    pub result: f64,
    pub explanation: String,
    pub is_roll: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiceRoll {
    pub value: f64,
    modification: Option<Modification>,
}

impl DiceRoll {
    fn new(value: f64) -> Self {
        Self {
            value,
            modification: None,
        }
    }

    fn modify_mut(&mut self, modification: Modification) {
        self.modification = Some(modification);
    }

    fn count_roll(&self) -> bool {
        matches!(&self.modification, None | Some(Modification::Exploded))
    }

    fn explain(&self) -> String {
        format!(
            "{}{}",
            self.value,
            self.modification
                .as_ref()
                .map(|m| m.suffix())
                .unwrap_or("".to_owned())
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiceRolls {
    pub values: Vec<DiceRoll>,
    sides: Vec<i64>,
    modification: Option<Modification>,
}

impl DiceRolls {
    fn new(value: f64, sides: Vec<i64>) -> Self {
        Self {
            values: vec![DiceRoll::new(value)],
            sides,
            modification: None,
        }
    }

    fn add_value(&mut self, value: f64) {
        self.values.push(DiceRoll::new(value));
    }

    // fn modify_mut(&mut self, modification: Modification) {
    //     self.modification = Some(modification);
    // }

    fn drop(&mut self) {
        self.modification = Some(Modification::Dropped)
    }

    fn reroll(&mut self, new_roll: f64) {
        if let Some(last) = self.values.iter_mut().last() {
            last.modify_mut(Modification::Rerolled);
        }

        self.add_value(new_roll);
    }

    fn explode(&mut self, new_roll: f64) {
        if let Some(last) = self.values.iter_mut().last() {
            last.modify_mut(Modification::Exploded);
        }

        self.add_value(new_roll);
    }

    fn min_side(&self) -> i64 {
        *self.sides.iter().min().expect("No sides")
    }

    // fn min_value(&self) -> i64 {
    //     *self.sides.iter().min().expect("No sides")
    //         * self.values.iter().filter(|v| v.count_roll()).count() as i64
    // }

    fn max_side(&self) -> i64 {
        *self.sides.iter().max().expect("No sides")
    }

    // fn max_value(&self) -> i64 {
    //     *self.sides.iter().max().expect("No sides")
    //         * self.values.iter().filter(|v| v.count_roll()).count() as i64
    // }

    fn count_roll(&self) -> bool {
        matches!(&self.modification, None | Some(Modification::Exploded))
    }

    fn sum(&self) -> f64 {
        if self.count_roll() {
            self.values
                .iter()
                .filter(|v| v.count_roll())
                .map(|v| v.value)
                .sum()
        } else {
            0.0
        }
    }

    fn last(&self) -> f64 {
        self.values.last().expect("No values").value
    }

    fn explain(&self) -> String {
        let modified_text = self
            .modification
            .as_ref()
            .map(|m| m.suffix())
            .unwrap_or("".to_owned());

        if self.values.len() == 1 {
            format!("{}{}", self.values[0].explain(), modified_text)
        } else {
            format!(
                "({}){}",
                self.values
                    .iter()
                    .map(|v| v.explain())
                    .collect::<Vec<_>>()
                    .join(", "),
                modified_text
            )
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Modification {
    Dropped,
    Rerolled,
    Exploded,
}

impl Modification {
    fn suffix(&self) -> String {
        match self {
            Modification::Dropped => "d".to_owned(),
            Modification::Rerolled => "r".to_owned(),
            Modification::Exploded => "!".to_owned(),
        }
    }
}
