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
                    results.push(DiceRolls::new(side_values[index], side_values.to_vec()));
                }

                Ok(results)
            }
            Some(cli::Mode::Avg) => todo!("Implement average mode"),
            Some(cli::Mode::Min) => todo!("Implement minimum mode"),
            Some(cli::Mode::Max) => todo!("Implement maximum mode"),
            Some(cli::Mode::Med) => todo!("Implement median mode"),
            Some(cli::Mode::Simavg(v)) => todo!("Implement simavg:{} mode", v),
        }
    }
}
