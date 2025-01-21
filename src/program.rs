use std::io::{Read, Write};

use crate::cli::Cli;

pub fn run<R: Read, W: Write>(reader: R, mut writer: W, cli: Cli) {
    // tokenize expression

    //
}

#[cfg(test)]
mod tests {}
