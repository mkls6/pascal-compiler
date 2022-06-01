use std::fmt;

#[derive(Clone)]
pub enum ErrorType {
    Lexical,
    Syntax,
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
    line: usize,
    column: usize,
    err_type: ErrorType,
}

impl CompilerError {
    pub fn new(description: String, line: usize, column: usize, etype: ErrorType) -> Self {
        Self {
            description,
            line,
            column,
            err_type: etype,
        }
    }

    pub fn lexical(description: String, line: usize, column: usize) -> Self {
        CompilerError::new(description, line, column, ErrorType::Lexical)
    }

    pub fn syntax(description: String, line: usize, column: usize) -> Self {
        CompilerError::new(description, line, column, ErrorType::Syntax)
    }

    pub fn semantic(description: String, line: usize, column: usize) -> Self {
        CompilerError::new(description, line, column, ErrorType::Semantic)
    }
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} Error [{}:{}] {}",
            self.err_type, self.line, self.column, self.description
        )
    }
}
