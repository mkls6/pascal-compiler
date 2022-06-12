use crate::error::CompilerError;
use crate::scope::{Scope, Usage};
use crate::syntax::*;

pub struct Analyzer {
    scopes: Vec<Scope>,
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            scopes: Vec::from([Scope::default()]),
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    pub fn leave_scope(&mut self) {
        self.scopes.pop();
    }

    /// Check if identifier is already defined
    pub fn check_declaration(
        &mut self,
        decl: VarDeclaration,
    ) -> Result<VarDeclaration, CompilerError> {
        let len = self.scopes.len();
        let cur_scope = &mut self.scopes[len - 1];
        let str = decl.id.get_id();

        match cur_scope.get(str.clone()) {
            Some(_) => Err(CompilerError::semantic(
                format!("Redeclaration of {:?}", str),
                decl.id.id.pos,
            )),
            None => {
                cur_scope.insert(&decl.id, Usage::Variable);
                Ok(decl)
            }
        }
    }

    pub fn find_identifier(&self, id: &Identifier, usg: &Usage) -> Result<(), CompilerError> {
        let mut scopes = self.scopes.iter().rev();

        loop {
            let cur_scope = scopes.next();

            if cur_scope.is_none() {
                break Err(CompilerError::semantic(
                    format!("Unknown identifier {:?}", id.get_id()),
                    id.id.pos,
                ));
            } else {
                match cur_scope.unwrap().get(id.get_id()) {
                    Some(u) if u == usg => break Ok(()),
                    Some(u) => {
                        break Err(CompilerError::semantic(
                            format!("Identifier<{:?}> found, expected <{:?}>", u, usg),
                            id.id.pos,
                        ))
                    }
                    None => continue,
                }
            }
        }
    }
}
