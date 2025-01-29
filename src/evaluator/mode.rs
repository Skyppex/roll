use rand::Rng;

use crate::cli;

use super::DiceRolls;

pub trait Mode {
    fn eval(&self, rolls: i64, side_values: &[i64]) -> Result<Vec<DiceRolls>, String>;
}

impl Mode for Option<cli::Mode> {
    fn eval(&self, rolls: i64, side_values: &[i64]) -> Result<Vec<DiceRolls>, String> {
        match self {
            None => {
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

                Ok(results)
            }
            Some(cli::Mode::Avg) => {
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

                Ok(results)
            }
            Some(cli::Mode::Min) => {
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

                Ok(results)
            }
            Some(cli::Mode::Max) => {
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

                Ok(results)
            }
            Some(cli::Mode::Med) => {
                let mut results = vec![];

                for _ in 0..rolls {
                    let len = side_values.len();

                    if len == 0 {
                        continue;
                    }

                    results.push(DiceRolls::new(med(side_values), side_values.to_vec()));
                }

                Ok(results)
            }
            Some(cli::Mode::Simavg(v)) => {
                let mut evals = vec![];

                for _ in 0..*v {
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

                    evals.push(results.iter().map(|v| v.sum()).collect::<Vec<_>>());
                }

                Ok(vec![DiceRolls::new(
                    evals.iter().map(|v| avg(v)).sum::<f64>() / *v as f64,
                    side_values.to_vec(),
                )])
            }
        }
    }
}

fn avg(values: &[f64]) -> f64 {
    let sum: f64 = values.iter().sum();
    sum / values.len() as f64
}

fn med(values: &[i64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort();

    let len = values.len();

    let mid = len / 2;

    if len % 2 == 0 {
        (values[mid - 1] + values[mid]) as f64 / 2.0
    } else {
        values[len / 2] as f64
    }
}
