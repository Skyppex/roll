mod cli;
mod evaluator;
mod io_utils;
mod parser;
mod path_utils;
mod program;
mod tokenizer;

use std::io::Result;

use clap::Parser;
use cli::Cli;
use io_utils::*;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let reader = get_reader(cli.source.as_deref())?;
    let writer = get_writer(cli.destination.as_deref())?;

    match program::run(reader, writer, cli.clone()) {
        Ok(_) => {}
        Err(e) => cli.quiet(|| eprintln!("{}", e)),
    }

    Ok(())
}
