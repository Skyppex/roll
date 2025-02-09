use crate::program::DynError;

pub fn tokenize(expression: &str) -> Result<Vec<Token>, DynError> {
    let mut tokens = Vec::new();
    let mut chars = expression.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            ' ' => {}
            '(' => tokens.push(Token::OpenParen),
            ')' => tokens.push(Token::CloseParen),
            '[' => tokens.push(Token::OpenBracket),
            ']' => tokens.push(Token::CloseBracket),
            '+' => tokens.push(Token::Add),
            '-' => {
                if let Some('0'..='9') = chars.peek() {
                    println!("negative number");
                    parse_number(c, &mut chars, &mut tokens)
                } else {
                    tokens.push(Token::Sub);
                }
            }
            '*' => tokens.push(Token::Mul),
            '/' => tokens.push(Token::Div),
            '%' => tokens.push(Token::Mod),
            ',' => tokens.push(Token::Comma),
            '.' => tokens.push(Token::Dot),
            'd' => tokens.push(Token::D),
            'f' => tokens.push(Token::F),
            'k' => tokens.push(Token::K),
            'h' => tokens.push(Token::H),
            'l' => tokens.push(Token::L),
            '!' => tokens.push(Token::Exclamation),
            'r' => tokens.push(Token::R),
            '=' => tokens.push(Token::Equals),
            '>' => tokens.push(Token::Greater),
            '<' => tokens.push(Token::Less),
            '~' => tokens.push(Token::Tilde),
            '0'..='9' => {
                parse_number(c, &mut chars, &mut tokens);
            }
            _ => Err(format!("Unexpected character: {}", c))?,
        }
    }

    Ok(tokens)
}

fn parse_number(
    c: char,
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    tokens: &mut Vec<Token>,
) {
    let mut value = c.to_string();

    while let Some('0'..='9') = chars.peek() {
        value.push(chars.next().unwrap());
    }

    let mut clone = chars.clone();
    let first = clone.next();
    let second = clone.next();

    #[allow(clippy::almost_complete_range)]
    if let (Some('.'), Some('0'..'9')) = (first, second) {
        value.push(chars.next().unwrap());

        while let Some('0'..='9') = chars.peek() {
            value.push(chars.next().unwrap());
        }

        let parsed = value.parse::<f64>().unwrap();
        tokens.push(Token::Float(parsed));
    } else {
        let parsed = value.parse::<i64>().unwrap();
        tokens.push(Token::Int(parsed));
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Values
    Int(i64),
    Float(f64),

    // Operations
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    // Punctuation
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
    Comma,
    Dot,

    // Rolls
    D, // Also a modifier
    F, // Fudge

    // Modifiers
    K,           // Keep
    H,           // Highest
    L,           // Lowest
    Exclamation, // Explode
    R,           // Reroll

    // Conditions
    Equals,
    Greater,
    Less,
    Tilde,
}
