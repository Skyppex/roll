use std::str::FromStr;

use clap::{ArgGroup, Parser};

/// Write a concise description of the command here.
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

    /// The mode to evaluate the expression with.
    #[arg(short, long)]
    pub mode: Option<Mode>,

    /// The expression to evaluate.
    /// If not provided, read from source or stdin.
    #[arg(last = true)]
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
    Avg,
    Min,
    Max,
    Med,
    Simavg(u32),
}

impl FromStr for Mode {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "avg" => Ok(Self::Avg),
            "min" => Ok(Self::Min),
            "max" => Ok(Self::Max),
            "med" => Ok(Self::Med),
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
