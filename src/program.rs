use std::{
    error::Error,
    io::{Read, Write},
};

use crate::{
    cli::Cli,
    evaluator::{eval, EvalResult},
    lexer::tokenize,
    parser::{parse, Cursor},
};

pub fn run<R: Read, W: Write>(reader: R, writer: W, cli: &Cli) -> Result<(), Box<dyn Error>> {
    // tokenize expression string
    let expression = cli.expression.join(" ");

    if !expression.is_empty() {
        run_amount(writer, &expression, cli)?;
    } else {
        run_lines(reader, writer, cli)?;
    }

    Ok(())
}

fn run_amount<W: Write>(mut writer: W, buf: &str, cli: &Cli) -> Result<(), Box<dyn Error>> {
    let tokens = tokenize(buf)?;

    cli.verbose(|| dbg!(&tokens));

    // parse tokens
    let mut cursor = Cursor::new(tokens);
    let tree = parse(&mut cursor)?;
    cli.verbose(|| dbg!(&tree));
    cli.verbose(|| eprintln!());

    for _ in 0..cli.amount.unwrap_or(1) - 1 {
        let result = eval(&tree, cli)?;
        writeln!(writer, "{}", format_result(result, cli))?;
    }

    let result = eval(&tree, cli)?;
    write!(writer, "{}", format_result(result, cli))?;
    Ok(())
}

fn run_lines<R: Read, W: Write>(
    mut reader: R,
    mut writer: W,
    cli: &Cli,
) -> Result<(), Box<dyn Error>> {
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
        format!("{} : {}", result.result, result.explanation)
    } else {
        format!("{}", result.result)
    }
}

#[cfg(test)]
mod tests {}
