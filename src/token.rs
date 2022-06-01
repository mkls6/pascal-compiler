use std::fmt;

#[derive(Debug, Clone)]
pub enum Token {
    Integer(i32),
    Identifier(String),
    StringLiteral(String),
    IntegerKeyword,
    RealKeyword,
    Real(f32),
    ProgramKeyword,
    VarKeyword,
    BeginKeyword,
    EndKeyword,
    PlusOp,
    MinusOp,
    MulOp,
    DivOp,
    ModOp,
    AssignOp,
    Colon,
    Period,
    LBrace,
    RBrace,
    Semicolon,
    EOF,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Integer(i) => write!(f, "Integer({})", i),
            Token::PlusOp => write!(f, "Operator(+)"),
            Token::MinusOp => write!(f, "Operator(-)"),
            Token::EOF => write!(f, "EOF"),
            Token::MulOp => write!(f, "Operator('*')"),
            Token::DivOp => write!(f, "Operator('div')"),
            Token::ModOp => write!(f, "Operator('mod')"),
            Token::Identifier(s) => write!(f, "Identifier('{}')", s),
            Token::BeginKeyword => write!(f, "'BEGIN' keyword"),
            Token::EndKeyword => write!(f, "'END' keyword"),
            Token::ProgramKeyword => write!(f, "'PROGRAM' keyword"),
            Token::IntegerKeyword => write!(f, "'INTEGER' keyword"),
            Token::VarKeyword => write!(f, "'VAR' keyword"),
            Token::AssignOp => write!(f, "'Assign (:=)' operator"),
            Token::Colon => write!(f, "Colon"),
            Token::Semicolon => write!(f, "Semicolon"),
            Token::Period => write!(f, "Period sign"),
            Token::LBrace => write!(f, "("),
            Token::RBrace => write!(f, ")"),
            Token::StringLiteral(s) => write!(f, "String literal '{}'", s),
            Token::Real(r) => write!(f, "Real literal '{}'", r),
            Token::RealKeyword => write!(f, "REAL keyword"),
        }
    }
}
