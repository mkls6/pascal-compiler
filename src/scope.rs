use crate::syntax::Identifier;
use crate::token::{Token, TokenType};
use std::collections::HashMap;
use std::fmt;

#[derive(PartialEq)]
pub enum Usage {
    Constant,
    Type,
    Program,
    Variable,
    // Procedure, function…
}

impl fmt::Debug for Usage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Usage::Constant => write!(f, "constant"),
            Usage::Type => write!(f, "type"),
            Usage::Program => write!(f, "program"),
            Usage::Variable => write!(f, "variable")
        }
    }
}

pub struct Scope {
    identifiers: HashMap<String, Usage>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            identifiers: HashMap::new(),
        }
    }
    pub fn default() -> Self {
        let identifiers = HashMap::from(
            [("integer".into(), Usage::Type),
                ("real".into(), Usage::Type),
                ("char".into(), Usage::Type),
                ("boolean".into(), Usage::Type),
                ("true".into(), Usage::Constant),
                ("false".into(), Usage::Constant)]
        );

        Self { identifiers }
    }

    pub fn get(&self, id: String) -> Option<&Usage> {
        self.identifiers.get(id.as_str())
    }

    pub fn insert(&mut self, id: &Identifier, usage: Usage) {
        if let Identifier {
            id:
                Token {
                    token: TokenType::Identifier(s),
                    ..
                },
        } = id
        {
            self.identifiers.insert(s.clone(), usage);
        }
    }
}
