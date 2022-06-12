use std::fmt;

#[derive(Clone)]
pub enum ErrorType {
    Lexical,
    Syntax,
    #[allow(dead_code)]
    Semantic,
}

impl fmt::Display for ErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorType::Lexical => write!(f, "Lexical"),
            ErrorType::Syntax => write!(f, "Syntax"),
            ErrorType::Semantic => write!(f, "Semantic"),
        }
    }
}

// TODO: don't use copy semantics
// It's a hack to fix ownership problems
// when checking current token in parser
#[derive(Clone)]
pub struct CompilerError {
    description: String,
    pos: (usize, usize),
    err_type: ErrorType,
}

impl CompilerError {
    pub fn new(description: String, pos: (usize, usize), err_type: ErrorType) -> Self {
        Self {
            description,
            pos,
            err_type,
        }
    }

    pub fn lexical(description: String, pos: (usize, usize)) -> Self {
        CompilerError::new(description, pos, ErrorType::Lexical)
    }

    pub fn syntax(description: String, pos: (usize, usize)) -> Self {
        CompilerError::new(description, pos, ErrorType::Syntax)
    }

    #[allow(dead_code)]
    pub fn semantic(description: String, pos: (usize, usize)) -> Self {
        CompilerError::new(description, pos, ErrorType::Semantic)
    }
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} Error [{}:{}] {}",
            self.err_type, self.pos.0, self.pos.1, self.description
        )
    }
}
