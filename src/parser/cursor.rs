use crate::lexer::Token;

pub struct Cursor {
    tokens: Vec<Token>,
}

impl Cursor {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens }
    }

    pub fn first(&self) -> Option<Token> {
        self.tokens.clone().into_iter().next()
    }

    pub fn second(&self) -> Option<Token> {
        let mut iter = self.tokens.clone().into_iter();
        iter.next();
        iter.next()
    }

    pub fn expect(&mut self, token: Token) -> Result<Token, String> {
        match self.tokens.clone().into_iter().next() {
            Some(t) if t == token => Ok(self.tokens.remove(0)),
            Some(t) => Err(format!("Expected {:?}, found {:?}", token, t)),
            None => Err(format!("Expected {:?}, found nothing", token)),
        }
    }

    pub fn bump(&mut self) -> Option<Token> {
        if self.tokens.first().is_some() {
            Some(self.tokens.remove(0))
        } else {
            None
        }
    }
}
