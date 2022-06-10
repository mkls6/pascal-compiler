use std::fmt;
use std::fmt::Formatter;

#[derive(Clone)]
pub struct Token {
    pub(crate) token: TokenType,
    pub(crate) pos: (usize, usize),
}

impl Token {
    pub fn new(token: TokenType, pos: (usize, usize)) -> Self {
        Token { token, pos }
    }
}

#[derive(Debug, Clone)]
pub enum TokenType {
    Integer(i32),
    Identifier(String),
    StringLiteral(String),
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
    Comma,
    Semicolon,
    EOF,
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Token")
            .field("type", &self.token)
            .field("position", &self.pos)
            .finish()
    }
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenType::Integer(i) => write!(f, "Integer({})", i),
            TokenType::PlusOp => write!(f, "Operator(+)"),
            TokenType::MinusOp => write!(f, "Operator(-)"),
            TokenType::EOF => write!(f, "EOF"),
            TokenType::MulOp => write!(f, "Operator('*')"),
            TokenType::DivOp => write!(f, "Operator('div')"),
            TokenType::ModOp => write!(f, "Operator('mod')"),
            TokenType::Identifier(s) => write!(f, "Identifier('{}')", s),
            TokenType::BeginKeyword => write!(f, "'BEGIN' keyword"),
            TokenType::EndKeyword => write!(f, "'END' keyword"),
            TokenType::ProgramKeyword => write!(f, "'PROGRAM' keyword"),
            TokenType::VarKeyword => write!(f, "'VAR' keyword"),
            TokenType::AssignOp => write!(f, "'Assign (:=)' operator"),
            TokenType::Colon => write!(f, "Colon"),
            TokenType::Comma => write!(f, ","),
            TokenType::Semicolon => write!(f, "Semicolon"),
            TokenType::Period => write!(f, "Period sign"),
            TokenType::LBrace => write!(f, "("),
            TokenType::RBrace => write!(f, ")"),
            TokenType::StringLiteral(s) => write!(f, "String literal '{}'", s),
            TokenType::Real(r) => write!(f, "Real literal '{}'", r),
        }
    }
}
