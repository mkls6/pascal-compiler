use std::fmt;

#[allow(dead_code)]
pub struct LexicalError {
    pub description: String,
}

impl fmt::Display for LexicalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Lexical Error: {}", self.description)
    }
}
