use std::{
    error::Error,
    io::{Read, Write},
};

use crate::{cli::Cli, parser::parse, tokenizer::tokenize};

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

    let mut tokens = tokenize(&buf);
    dbg!(&tokens);

    // parse tokens
    let tree = parse(&mut tokens);
    dbg!(&tree);

    Ok(())
}

#[cfg(test)]
mod tests {}
