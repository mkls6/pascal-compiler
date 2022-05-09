use std::fmt;

pub struct LexicalError {
    description: String,
    line: usize,
    column: usize,
}

impl LexicalError {
    pub fn new(description: String, line: usize, column: usize) -> Self {
        Self {
            description,
            line,
            column,
        }
    }
}

impl fmt::Display for LexicalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Lexical Error [{}:{}] {}",
            self.line, self.column, self.description
        )
    }
}
