mod mode;

use mode::Mode;

use crate::{
    cli::{self, Cli},
    parser::{BinOp, Expr, Modifier, Sides},
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
    modifiers: &[Modifier],
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

    let results = cli.mode.eval(rolls, &side_values, modifiers, cli)?;

    let mut results_explanation = String::new();

    for (i, result) in results.iter().enumerate() {
        results_explanation.push_str(
            to_fudge(
                &result.explain(),
                is_fudge,
                cli.mode.as_ref().unwrap_or(&cli::Mode::Rng),
            )
            .as_str(),
        );

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
        result: results
            .iter()
            .map(|r| r.sum(cli.mode.as_ref()))
            .sum::<f64>(),
        explanation,
        is_roll: true,
    })
}

fn to_fudge(roll_str: &str, is_fudge: bool, mode: &cli::Mode) -> String {
    if !is_fudge || !matches!(mode, cli::Mode::Rng) {
        return roll_str.to_string();
    }

    match roll_str {
        "-1" => "-".to_string(),
        "1" => "+".to_string(),
        "0" => "o".to_string(),
        other => other.to_string(),
    }
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

    fn modify(&mut self, modification: Modification) {
        self.modification = Some(modification);
    }

    fn count_roll(&self, mode: Option<&cli::Mode>) -> bool {
        matches!(
            (mode.unwrap_or(&cli::Mode::Rng), &self.modification),
            (cli::Mode::Avg, Some(Modification::Rerolled))
                | (_, None | Some(Modification::Exploded))
        )
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
            last.modify(Modification::Rerolled);
        }

        self.add_value(new_roll);
    }

    fn explode(&mut self, new_roll: f64) {
        if let Some(last) = self.values.iter_mut().last() {
            last.modify(Modification::Exploded);
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

    fn count_roll(&self, mode: Option<&cli::Mode>) -> bool {
        matches!(
            (mode.unwrap_or(&cli::Mode::Rng), &self.modification),
            (cli::Mode::Avg, Some(Modification::Rerolled))
                | (_, None | Some(Modification::Exploded))
        )
    }

    fn sum(&self, mode: Option<&cli::Mode>) -> f64 {
        if self.count_roll(mode) {
            self.values
                .iter()
                .filter(|v| v.count_roll(mode))
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
                "{{{}}}{}",
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
