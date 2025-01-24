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

pub fn run<R: Read, W: Write>(
    mut reader: R,
    mut writer: W,
    cli: Cli,
) -> Result<(), Box<dyn Error>> {
    // tokenize expression string
    let mut buf = String::new();

    let expression = cli.expression.join(" ");

    if !expression.is_empty() {
        buf.push_str(&expression);
    } else {
        reader.read_to_string(&mut buf)?;
    }

    let tokens = tokenize(&buf)?;

    cli.verbose(|| dbg!(&tokens));

    // parse tokens
    let mut cursor = Cursor::new(tokens);
    let tree = parse(&mut cursor)?;
    cli.verbose(|| dbg!(&tree));
    cli.verbose(|| eprintln!());

    for _ in 0..cli.amount.unwrap_or(1) - 1 {
        let result = eval(&tree)?;
        writeln!(writer, "{}", format_result(result, &cli))?;
    }

    let result = eval(&tree)?;
    write!(writer, "{}", format_result(result, &cli))?;

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
