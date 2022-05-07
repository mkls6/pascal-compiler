use std::fmt;

#[derive(Debug)]
pub enum Token {
    INTEGER(i32),
    IDENTIFIER(String),
    PROGRAM,
    BEGIN,
    END,
    PLUS,
    MINUS,
    MUL,
    DIV,
    MOD,
    EOF,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::INTEGER(i) => write!(f, "Integer({})", i),
            Token::PLUS => write!(f, "Operator(+)"),
            Token::MINUS => write!(f, "Operator(-)"),
            Token::EOF => write!(f, "EOF"),
            Token::MUL => write!(f, "Operator('*')"),
            Token::DIV => write!(f, "Operator('div')"),
            Token::MOD => write!(f, "Operator('mod')"),
            Token::IDENTIFIER(s) => write!(f, "Identifier('{}')", s),
            Token::BEGIN => write!(f, "'BEGIN' keyword"),
            Token::END => write!(f, "'END' keyword"),
            Token::PROGRAM => write!(f, "'PROGRAM' keyword"),
        }
    }
}
