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

    /// Check if identifier is already defined and add in case it is not
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
                cur_scope.insert(&decl.id, Usage::Variable(decl.type_name.get_id()));
                Ok(decl)
            }
        }
    }

    pub fn get_factor_type(&self, f: &Factor) -> Result<Usage, CompilerError> {
        match f {
            Factor::Real(_) => Ok(Usage::Constant("real".into())),
            Factor::Integer(_) => Ok(Usage::Constant("integer".into())),
            Factor::Identifier(s) => {
                let usg = self.find_identifier(s)?;
                Ok(usg.clone())
            }
            Factor::Expression(e) => Ok(Usage::Variable(e.expr_type.clone())),
        }
    }

    pub fn find_identifier(&self, id: &Identifier) -> Result<&Usage, CompilerError> {
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
                    Some(u) => break Ok(u),
                    None => continue,
                }
            }
        }
    }

    pub fn merge_types(
        &self,
        type1: &String,
        type2: &String,
        pos: (usize, usize),
        strong: bool,
    ) -> Result<String, CompilerError> {
        match (type1.as_str(), type2.as_str()) {
            ("integer", "real") | ("real", "integer") => {
                if strong {
                    Err(CompilerError::semantic("Type mismatch".into(), pos))
                } else {
                    Ok("real".into())
                }
            }
            (x, y) if x == y => Ok(x.into()),
            (x, "") => Ok(x.into()),
            _ => Err(CompilerError::semantic("Type mismatch".into(), pos)),
        }
    }

    pub fn get_sub_term_type(&self, sub_term: &SubTerm) -> Result<String, CompilerError> {
        // Type of subterm is its factor's type or merge with inner subterm type
        let usg = self.get_factor_type(&sub_term.factor)?;
        let factor_type_str = match usg {
            Usage::Constant(s) | Usage::Variable(s) => s,
            _ => todo!(),
        };

        if sub_term.sub_term.is_none() {
            Ok(factor_type_str)
        } else {
            let sub_term_type = self.get_sub_term_type(sub_term.sub_term.as_ref().unwrap())?;
            self.merge_types(&factor_type_str, &sub_term_type, (0, 0), false)
        }
    }

    pub fn check_assignment(&self, a: VarAssignment) -> Result<VarAssignment, CompilerError> {
        let var_id = &a.name;
        let var_type = self.find_identifier(var_id)?;
        let value_type = &a.value.expr_type;

        match var_type {
            Usage::Variable(s) => {
                self.merge_types(s, value_type, a.name.id.pos, true)?;
                Ok(a)
            }
            // We can't actually get here but Rust enforces to do check anyway
            _ => todo!(),
        }
    }

    // pub fn check_expr_type(expr: &Expression) -> Result<(), CompilerError> {
    //     let term_type = expr.term.term_type;
    //     let sub_expr_type = match expr.sub_expr
    // }
}
