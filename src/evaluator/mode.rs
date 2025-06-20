use rand::Rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    cli::{self, Cli},
    parser::{Condition, Modifier, RelOp},
    program::DynError,
};

use super::{eval, DiceRolls, EvalResult};

type Roller<'a> = Box<
    dyn Fn(i64, &[i64], &[Modifier], &Option<cli::Mode>, &Cli) -> Result<Vec<DiceRolls>, DynError>
        + 'a,
>;

pub trait Mode {
    fn eval(
        &self,
        rolls: i64,
        side_values: &[i64],
        modifiers: &[Modifier],
        cli: &Cli,
    ) -> Result<Vec<DiceRolls>, DynError>;
}

impl Mode for Option<cli::Mode> {
    fn eval(
        &self,
        rolls: i64,
        side_values: &[i64],
        modifiers: &[Modifier],
        cli: &Cli,
    ) -> Result<Vec<DiceRolls>, DynError> {
        let roller = get_roller(self);
        let results = roller(rolls, side_values, modifiers, self, cli)?;
        Ok(results)
    }
}

fn get_roller(mode: &Option<cli::Mode>) -> Roller {
    match mode {
        None | Some(cli::Mode::Rng) => Box::new(
            |rolls: i64,
             side_values: &[i64],
             modifiers: &[Modifier],
             mode: &Option<cli::Mode>,
             cli: &Cli| {
                let mut rng = rand::thread_rng();
                let mut results = vec![];

                for _ in 0..rolls {
                    let len = side_values.len();
                    if len == 0 {
                        continue;
                    }

                    let index = rng.gen_range(0..len);
                    results.push(DiceRolls::new(
                        side_values[index] as f64,
                        side_values.to_vec(),
                    ));
                }

                apply_modifiers(
                    rolls,
                    side_values,
                    get_roller(mode),
                    modifiers,
                    &mut results,
                    mode,
                    cli,
                )?;

                Ok(results)
            },
        ),
        Some(cli::Mode::Avg) => Box::new(
            |rolls: i64,
             side_values: &[i64],
             modifiers: &[Modifier],
             mode: &Option<cli::Mode>,
             cli: &Cli| {
                let mut results = vec![];

                for _ in 0..rolls {
                    let len = side_values.len();
                    if len == 0 {
                        continue;
                    }

                    results.push(DiceRolls::new(
                        avg(&side_values.iter().map(|v| *v as f64).collect::<Vec<_>>()),
                        side_values.to_vec(),
                    ));
                }

                apply_modifiers(
                    rolls,
                    side_values,
                    get_roller(mode),
                    modifiers,
                    &mut results,
                    mode,
                    cli,
                )?;

                Ok(results)
            },
        ),
        Some(cli::Mode::Min) => Box::new(
            |rolls: i64,
             side_values: &[i64],
             modifiers: &[Modifier],
             mode: &Option<cli::Mode>,
             cli: &Cli| {
                let mut results = vec![];

                for _ in 0..rolls {
                    let len = side_values.len();
                    if len == 0 {
                        continue;
                    }

                    results.push(DiceRolls::new(
                        *side_values.iter().min().expect("No sides") as f64,
                        side_values.to_vec(),
                    ));
                }

                apply_modifiers(
                    rolls,
                    side_values,
                    get_roller(mode),
                    modifiers,
                    &mut results,
                    mode,
                    cli,
                )?;

                Ok(results)
            },
        ),
        Some(cli::Mode::Max) => Box::new(
            |rolls: i64,
             side_values: &[i64],
             modifiers: &[Modifier],
             mode: &Option<cli::Mode>,
             cli: &Cli| {
                let mut results = vec![];

                for _ in 0..rolls {
                    let len = side_values.len();
                    if len == 0 {
                        continue;
                    }

                    results.push(DiceRolls::new(
                        *side_values.iter().max().expect("No sides") as f64,
                        side_values.to_vec(),
                    ));
                }

                apply_modifiers(
                    rolls,
                    side_values,
                    get_roller(mode),
                    modifiers,
                    &mut results,
                    mode,
                    cli,
                )?;

                Ok(results)
            },
        ),
        Some(cli::Mode::Med) => Box::new(
            |rolls: i64,
             side_values: &[i64],
             modifiers: &[Modifier],
             mode: &Option<cli::Mode>,
             cli: &Cli| {
                let mut results = vec![];

                for _ in 0..rolls {
                    let len = side_values.len();
                    if len == 0 {
                        continue;
                    }

                    results.push(DiceRolls::new(med(side_values), side_values.to_vec()));
                }

                apply_modifiers(
                    rolls,
                    side_values,
                    get_roller(mode),
                    modifiers,
                    &mut results,
                    mode,
                    cli,
                )?;

                Ok(results)
            },
        ),
        Some(cli::Mode::Simavg(v)) => Box::new(
            move |rolls: i64,
                  side_values: &[i64],
                  modifiers: &[Modifier],
                  _mode: &Option<cli::Mode>,
                  cli: &Cli| {
                let evals = (0..*v)
                    .into_par_iter()
                    .map(|_| -> Result<_, DynError> {
                        Ok(Some(cli::Mode::Rng)
                            .eval(rolls, side_values, modifiers, cli)?
                            .iter()
                            .map(|v| v.sum(cli.mode.as_ref()))
                            .collect::<Vec<_>>())
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(vec![DiceRolls::new(
                    avg(&evals
                        .iter()
                        .map(|rolls| rolls.iter().sum::<f64>())
                        .collect::<Vec<_>>()),
                    side_values.to_vec(),
                )])
            },
        ),
    }
}

fn apply_modifiers(
    rolls: i64,
    side_values: &[i64],
    roller: Roller,
    modifiers: &[Modifier],
    results: &mut [DiceRolls],
    mode: &Option<cli::Mode>,
    cli: &Cli,
) -> Result<(), DynError> {
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

                results.sort_by(|a, b| {
                    a.sum(cli.mode.as_ref())
                        .partial_cmp(&b.sum(cli.mode.as_ref()))
                        .expect("Cannot compare")
                });

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

                results.sort_by(|a, b| {
                    b.sum(cli.mode.as_ref())
                        .partial_cmp(&a.sum(cli.mode.as_ref()))
                        .expect("Cannot compare")
                });

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

                results.sort_by(|a, b| {
                    b.sum(cli.mode.as_ref())
                        .partial_cmp(&a.sum(cli.mode.as_ref()))
                        .expect("Cannot compare")
                });

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

                results.sort_by(|a, b| {
                    a.sum(cli.mode.as_ref())
                        .partial_cmp(&b.sum(cli.mode.as_ref()))
                        .expect("Cannot compare")
                });

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
                    match mode {
                        Some(cli::Mode::Avg) => {
                            let len = side_values.len();

                            if len == 0 {
                                continue;
                            }

                            let avg =
                                avg(&side_values.iter().map(|v| *v as f64).collect::<Vec<_>>());

                            for i in 1..=value {
                                let prob =
                                    reroll_probability(side_values, i as u64, condition.clone())?;

                                let new_roll = prob * avg;
                                result.reroll(new_roll);
                            }
                        }
                        _ => {
                            for _ in 0..value {
                                if let Some((operator, ref condition_value)) = condition {
                                    if !rel_op_eval(operator, result, condition_value)? {
                                        continue;
                                    }
                                } else if result.last() > result.min_side() as f64 {
                                    continue;
                                }

                                let len = side_values.len();

                                if len == 0 {
                                    continue;
                                }

                                let new_roll = roller(1, side_values, &[], mode, cli)?;
                                result.reroll(
                                    new_roll.iter().map(|r| r.sum(cli.mode.as_ref())).sum(),
                                );
                            }
                        }
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
                    match mode {
                        Some(cli::Mode::Avg) => {
                            let len = side_values.len();

                            if len == 0 {
                                continue;
                            }

                            let avg =
                                avg(&side_values.iter().map(|v| *v as f64).collect::<Vec<_>>());

                            for i in 1..=value {
                                let prob =
                                    explode_probability(side_values, i as u64, condition.clone())?;

                                let new_roll = prob * avg;
                                result.explode(new_roll);
                            }
                        }
                        _ => {
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

                                let new_roll = roller(1, side_values, &[], mode, cli)?;
                                result.explode(
                                    new_roll.iter().map(|r| r.sum(cli.mode.as_ref())).sum(),
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn avg(values: &[f64]) -> f64 {
    let sum: f64 = values.iter().sum();
    sum / values.len() as f64
}

fn med(values: &[i64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort();

    let len = sorted.len();

    let mid = len / 2;

    if len % 2 == 0 {
        (sorted[mid - 1] + sorted[mid]) as f64 / 2.0
    } else {
        sorted[len / 2] as f64
    }
}

fn rel_op_eval(operator: &RelOp, left: &DiceRolls, right: &EvalResult) -> Result<bool, DynError> {
    let left = left.last();
    let right = right.result;

    rel_op_eval_value(operator, left, right)
}

fn rel_op_eval_value(operator: &RelOp, left: f64, right: f64) -> Result<bool, DynError> {
    Ok(match operator {
        RelOp::Equals => left == right,
        RelOp::NotEquals => left != right,
        RelOp::Greater => left > right,
        RelOp::GreaterEqual => left >= right,
        RelOp::Less => left < right,
        RelOp::LessEqual => left <= right,
    })
}

fn explode_probability(
    side_values: &[i64],
    depth: u64,
    condition: Option<(&RelOp, EvalResult)>,
) -> Result<f64, DynError> {
    let len = side_values.len();
    let mut will_explode_count = 0;

    let (op, rhs) = condition.map_or_else(
        || {
            (
                &RelOp::Equals,
                *side_values.iter().max().unwrap_or(&0) as f64,
            )
        },
        |(op, ref result)| (op, result.result),
    );

    for v in side_values {
        if rel_op_eval_value(op, *v as f64, rhs)? {
            will_explode_count += 1;
        }
    }

    let prob = will_explode_count as f64 / len as f64;

    Ok(prob.powf(depth as f64))
}

fn reroll_probability(
    side_values: &[i64],
    depth: u64,
    condition: Option<(&RelOp, EvalResult)>,
) -> Result<f64, DynError> {
    let len = side_values.len();
    let mut will_reroll_count = 0;

    let (op, rhs) = condition.map_or_else(
        || {
            (
                &RelOp::Equals,
                *side_values.iter().min().unwrap_or(&0) as f64,
            )
        },
        |(op, ref result)| (op, result.result),
    );

    for v in side_values {
        if rel_op_eval_value(op, *v as f64, rhs)? {
            will_reroll_count += 1;
        }
    }

    let prob = will_reroll_count as f64 / len as f64;

    Ok(prob.powf(depth as f64))
}
