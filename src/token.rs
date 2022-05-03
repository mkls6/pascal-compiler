use std::fmt;

#[derive(Debug)]
pub enum Token {
    INTEGER(i32),
    PLUS,
    MINUS,
    EOF,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::INTEGER(i) => write!(f, "Integer({})", i),
            Token::PLUS => write!(f, "Operator(+)"),
            Token::MINUS => write!(f, "Operator(-)"),
            Token::EOF => write!(f, "EOF"),
        }
    }
}
