use crate::syntax::Identifier;
use crate::token::{Token, TokenType};
use std::collections::HashMap;
use std::fmt;

#[derive(PartialEq, Clone)]
pub enum Usage {
    Constant(String),
    Type(Option<String>),
    Program,
    Variable(String),
    // Procedure, function…
}

impl fmt::Debug for Usage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Usage::Constant(s) => write!(f, "constant of type \"{:?}\"", s),
            Usage::Type(s) => write!(f, "type alias of \"{:?}\"", s),
            Usage::Program => write!(f, "program"),
            Usage::Variable(s) => write!(f, "variable of type \"{:?}\"", s),
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
        let identifiers = HashMap::from([
            ("integer".into(), Usage::Type(None)),
            ("real".into(), Usage::Type(None)),
            ("char".into(), Usage::Type(None)),
            ("boolean".into(), Usage::Type(None)),
            ("true".into(), Usage::Constant("boolean".into())),
            ("false".into(), Usage::Constant("boolean".into())),
        ]);

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
            ..
        } = id
        {
            self.identifiers.insert(s.clone(), usage);
        }
    }
}
