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

    #[arg(short = 'n', long)]
    pub amount: Option<i32>,

    #[arg(short, long)]
    pub explain: bool,

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
