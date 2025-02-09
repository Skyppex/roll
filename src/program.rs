use std::{
    error::Error,
    io::{Read, Write},
    sync::{Arc, Mutex},
};

use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    cli::Cli,
    evaluator::{eval, EvalResult},
    lexer::tokenize,
    parser::{parse, Cursor},
};

pub type DynError = Box<dyn Error + Send + Sync>;

pub fn run<R: Read, W: Write + Send + 'static>(
    reader: R,
    writer: W,
    cli: &Cli,
) -> Result<(), DynError> {
    // tokenize expression string
    let expression = cli.expression.join(" ");

    if !expression.is_empty() {
        run_amount(writer, &expression, cli)?;
    } else {
        run_lines(reader, writer, cli)?;
    }

    Ok(())
}

fn run_amount<W: Write + Send + 'static>(writer: W, buf: &str, cli: &Cli) -> Result<(), DynError> {
    let tokens = tokenize(buf)?;

    cli.verbose(|| dbg!(&tokens));

    // parse tokens
    let mut cursor = Cursor::new(tokens);
    let tree = parse(&mut cursor)?;
    cli.verbose(|| dbg!(&tree));
    cli.verbose(|| eprintln!());

    let writer = Arc::new(Mutex::new(writer));

    (0..cli.amount.unwrap_or(1) - 1)
        .into_par_iter()
        .try_for_each(|_| -> Result<_, DynError> {
            let result = eval(&tree, cli)?;
            let mut writer = writer.lock().unwrap();
            writeln!(writer, "{}", format_result(result, cli)).map_err(|e| e.into())
        })?;

    let result = eval(&tree, cli)?;
    let mut writer = writer.lock().unwrap();
    write!(writer, "{}", format_result(result, cli))?;
    Ok(())
}

fn run_lines<R: Read, W: Write>(mut reader: R, mut writer: W, cli: &Cli) -> Result<(), DynError> {
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;

    let lines = buf.lines();
    let lines_count = lines.clone().count();

    for (i, line) in buf.lines().enumerate() {
        if line.is_empty() {
            continue;
        }

        let tokens = tokenize(line)?;

        cli.verbose(|| dbg!(&tokens));

        // parse tokens
        let mut cursor = Cursor::new(tokens);
        let tree = parse(&mut cursor)?;
        cli.verbose(|| dbg!(&tree));
        cli.verbose(|| eprintln!());

        if i < lines_count - 1 {
            let result = eval(&tree, cli)?;
            writeln!(writer, "{}", format_result(result, cli))?;
        } else {
            let result = eval(&tree, cli)?;
            write!(writer, "{}", format_result(result, cli))?;
        }
    }

    Ok(())
}

fn format_result(result: EvalResult, cli: &Cli) -> String {
    if cli.explain {
        format!("{:.5} : {}", result.result, result.explanation)
    } else {
        format!("{:.5}", result.result)
    }
}

#[cfg(test)]
mod tests {}
