pub fn tokenize(expression: &str) -> Vec<Token> {
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
            '-' => tokens.push(Token::Sub),
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
            '!' => tokens.push(Token::Explanation),
            'r' => tokens.push(Token::R),
            '0'..='9' => {
                let mut value = c.to_string();

                while let Some('0'..='9') = chars.peek() {
                    value.push(chars.next().unwrap());
                }

                if let Some('.') = chars.peek() {
                    value.push(chars.next().unwrap());

                    while let Some('0'..='9') = chars.peek() {
                        value.push(chars.next().unwrap());
                    }

                    tokens.push(Token::Float(value.parse().unwrap()));
                } else {
                    tokens.push(Token::Int(value.parse().unwrap()));
                }
            }
            _ => panic!("Invalid character: {}", c),
        }
    }
    tokens
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
    Explanation, // Explode
    R,           // Reroll
}
