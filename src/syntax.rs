use crate::token::{Token, TokenType};
use std::fmt;

pub enum Factor {
    Integer(Token),
    Real(Token),
    Identifier(Identifier),
    Expression(Box<Expression>),
}

pub enum AdditiveOp {
    Plus,
    Minus,
}

pub enum MultiplicativeOp {
    Mul,
    Div,
    Mod,
}

#[derive(Clone)]
pub struct Identifier {
    pub(crate) id: Token,
}

impl Identifier {
    pub fn get_id(&self) -> String {
        match &self.id.token {
            TokenType::Identifier(s) => s.clone(),
            _ => "".into(),
        }
    }
}

pub struct VarAssignment {
    pub(crate) name: Identifier,
    pub(crate) value: Expression,
}

pub struct Term {
    pub(crate) factor: Factor,
    pub(crate) sub_term: Option<SubTerm>,
    pub(crate) term_type: String,
}

pub struct SubTerm {
    pub(crate) op: MultiplicativeOp,
    pub(crate) factor: Factor,
    pub(crate) sub_term_type: String,
    pub(crate) sub_term: Option<Box<SubTerm>>,
}

pub struct SubExpression {
    pub(crate) op: AdditiveOp,
    pub(crate) term: Term,
    pub(crate) sub_expr_type: String,
    pub(crate) sub_expr: Option<Box<SubExpression>>,
}

pub struct Expression {
    pub(crate) term: Term,
    pub(crate) sub_expr: Option<SubExpression>,
    // pub(crate) expr_type: Identifier,
}

pub struct Compound {
    pub(crate) statements: Vec<Statement>,
}

pub enum Statement {
    Simple(VarAssignment),
}

pub struct VarDeclaration {
    pub(crate) id: Identifier,
    pub(crate) type_name: Identifier,
}

pub struct VarSection {
    pub(crate) declarations: Vec<VarDeclaration>,
}

pub struct Program {
    pub(crate) identifier: Identifier,
    pub(crate) var_section: Option<VarSection>,
    pub(crate) compound: Compound,
}

impl fmt::Debug for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Program")
            .field("identifier", &self.identifier)
            .field("var_section", &self.var_section)
            .field("compound", &self.compound)
            .finish()
    }
}

impl fmt::Debug for VarSection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VarSection")
            .field("declarations", &self.declarations)
            .finish()
    }
}

impl fmt::Debug for VarDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VarDeclaration")
            .field("id", &self.id)
            .field("type_name", &self.type_name)
            .finish()
    }
}

impl fmt::Debug for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Statement::Simple(a) => f
                .debug_struct("Simple Statement")
                .field("value", &a)
                .finish(),
        }
    }
}

impl fmt::Debug for Compound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Compound")
            .field("statements", &self.statements)
            .finish()
    }
}

impl fmt::Debug for Factor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Factor::Integer(i) => write!(f, "Factor<Int>({:?})", i),
            Factor::Real(real) => write!(f, "Factor<Real>({:?})", real),
            Factor::Identifier(i) => write!(f, "Factor<Variable>({:?})", i),
            Factor::Expression(inner) => f
                .debug_struct("Factor")
                .field("expression", &inner)
                .finish(),
        }
    }
}

impl fmt::Debug for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Expression")
            .field("term", &self.term)
            .field("sub_expr", &self.sub_expr)
            .finish()
    }
}

impl fmt::Debug for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Term")
            .field("factor", &self.factor)
            .field("sub_term", &self.sub_term)
            .field("term_type", &self.term_type)
            .finish()
    }
}

impl fmt::Debug for SubTerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SubTerm")
            .field("op", &self.op)
            .field("factor", &self.factor)
            .field("sub_term", &self.sub_term)
            .field("sub_term_type", &self.sub_term_type)
            .finish()
    }
}

impl fmt::Debug for AdditiveOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdditiveOp::Plus => write!(f, "Plus <+>"),
            AdditiveOp::Minus => write!(f, "Minus <->"),
        }
    }
}

impl fmt::Debug for MultiplicativeOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MultiplicativeOp::Mul => write!(f, "Mul <*>"),
            MultiplicativeOp::Div => write!(f, "Div"),
            MultiplicativeOp::Mod => write!(f, "Mod"),
        }
    }
}

impl fmt::Debug for SubExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SubExpression")
            .field("op", &self.op)
            .field("term", &self.term)
            .field("sub_expr", &self.sub_expr)
            .field("sub_expr_type", &self.sub_expr_type)
            .finish()
    }
}

impl fmt::Debug for VarAssignment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VarAssignment")
            .field("identifier", &self.name)
            .field("value", &self.value)
            .finish()
    }
}

impl fmt::Debug for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Identifier")
            .field("name", &self.id)
            .finish()
    }
}
