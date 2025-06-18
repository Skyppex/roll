use std::str::FromStr;

use clap::{ArgGroup, Parser};

/// Roll some dice using the command line.
#[derive(Debug, Clone, Parser)]
#[command(version, author, about)]
#[command(group=ArgGroup::new("log").args(["verbose", "quiet"]).multiple(false))]
#[command(group=ArgGroup::new("multi").args(["source", "amount"]).multiple(false))]
pub struct Cli {
    /// The source file to read from. If not provided, read from stdin.
    #[arg(short, long)]
    pub source: Option<String>,

    /// The destination file to write to. If not provided, write to stdout.
    #[arg(short, long)]
    pub destination: Option<String>,

    /// Enable verbose logging.
    #[arg(short, long)]
    pub verbose: bool,

    /// Suppress all informational output.
    /// Errors will still be printed to stderr.
    #[arg(short, long)]
    pub quiet: bool,

    /// The number of times to evaluate the expression. Each evaluation will be printed on a new line.
    #[arg(short = 'n', long)]
    pub amount: Option<i32>,

    /// Print the result of the expression evaluation.
    #[arg(short, long)]
    pub explain: bool,

    /// The mode to evaluate the expression with. rng (default), avg, min, max, med, simavg:<iteration>.
    #[arg(short, long)]
    pub mode: Option<Mode>,

    /// The expression to evaluate. If not provided, read from source or stdin.
    /// ‎
    /// Syntax:
    /// - `d6` - roll a 6-sided die
    /// - `d[3..6]` - roll a die with a range of sides (inclusive)
    /// - `d[3, 5, 7]` - roll a die with a set of sides
    /// - `df` - roll a fudge die (same as d[-1, 0, 1])
    /// - `2d8` - roll two 8-sided die
    /// - `2d6k or 2d6kh` - roll two 6-sided die and keep the highest
    /// - `2d6kl` - roll two 6-sided die and keep the lowest
    /// - `2d6d or 2d6dl` - roll two 6-sided die and drop the lowest
    /// - `2d6dh` - roll two 6-sided die and drop the highest
    /// - `1d6!` - roll a 6-sided die and explode on 6
    /// - `1d6r` - roll a 6-sided die and reroll on 1
    /// ‎
    /// Conditionals:
    /// For reroll or explode, you can add a condition.
    /// After the '!' or 'r', append:
    /// - `=3` - equals 3
    /// - `~=3` - not equals 3
    /// - `>3` - greater than 3
    /// - `<3` - less than 3
    /// - `>=3` - greater than or equals 3
    /// - `<=3` - less than or equals 3
    /// ‎
    /// Examples:
    /// - `1d20 + 5` - roll a 20-sided die and add 5
    /// - `(1d4)d4` - roll 1d4 number of d4s
    /// - `4d(1d4)` - roll 4 dice with 1d4 sides
    /// - `2d6kh!>=5` - roll two 6-sided die, keep highest, then explode on 5 or higher
    #[arg(last = true, verbatim_doc_comment)]
    pub expression: Vec<String>,
}

impl Cli {
    pub fn verbose<T, F: Fn() -> T>(&self, f: F) {
        if self.verbose {
            _ = f();
        }
    }

    pub fn quiet<T, F: Fn() -> T>(&self, f: F) {
        if !self.quiet {
            _ = f();
        }
    }
}

#[derive(Debug, Clone)]
pub enum Mode {
    Rng,
    Avg,
    Min,
    Max,
    Med,
    Simavg(u32),
}

impl FromStr for Mode {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "rng" | "rand" | "r" => Ok(Self::Rng),
            "avg" | "a" => Ok(Self::Avg),
            "min" | "-" => Ok(Self::Min),
            "max" | "+" => Ok(Self::Max),
            "med" | "m" => Ok(Self::Med),
            s if s.starts_with("simavg") => {
                let n = s
                    .split(':')
                    .nth(1)
                    .ok_or("expected 'simavg:<iterations>'")?;

                let n = n.parse().map_err(|_| "invalid number")?;
                Ok(Self::Simavg(n))
            }
            _ => Err("invalid mode"),
        }
    }
}
