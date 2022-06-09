use std::fmt;
use std::fmt::Formatter;

pub enum Factor {
    Integer(i32),
    Real(f32),
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

pub struct Identifier {
    pub(crate) name: String
    // TODO: type, usage, etc
}

pub struct VarAssignment {
    pub(crate) name: Identifier,
    pub(crate) value: Expression
}

pub struct Term {
    pub(crate) factor: Factor,
    pub(crate) sub_term: Option<SubTerm>,
}

pub struct SubTerm {
    pub(crate) op: MultiplicativeOp,
    pub(crate) factor: Factor,
    pub(crate) sub_term: Option<Box<SubTerm>>,
}

pub struct SubExpression {
    pub(crate) op: AdditiveOp,
    pub(crate) term: Term,
    pub(crate) sub_expr: Option<Box<SubExpression>>,
}

pub struct Expression {
    pub(crate) term: Term,
    pub(crate) sub_expr: Option<SubExpression>,
}

impl fmt::Debug for Factor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Factor::Integer(i) => write!(f, "Factor<Int>({:?})", i),
            Factor::Real(real) => write!(f, "Factor<Real>({:?})", real),
            Factor::Expression(inner) => {
                f.debug_struct("Factor")
                    .field("expression", &inner)
                    .finish()
            }
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
            .finish()
    }
}

impl fmt::Debug for SubTerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SubTerm")
            .field("op", &self.op)
            .field("factor", &self.factor)
            .field("sub_term", &self.sub_term)
            .finish()
    }
}

impl fmt::Debug for AdditiveOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdditiveOp::Plus => write!(f, "Plus <+>"),
            AdditiveOp::Minus => write!(f, "Minus <->")
        }
    }
}

impl fmt::Debug for MultiplicativeOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MultiplicativeOp::Mul => write!(f, "Mul <*>"),
            MultiplicativeOp::Div => write!(f, "Div"),
            MultiplicativeOp::Mod => write!(f, "Mod")
        }
    }
}

impl fmt::Debug for SubExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SubExpression")
            .field("op", &self.op)
            .field("term", &self.term)
            .field("sub_expr", &self.sub_expr)
            .finish()
    }
}

impl fmt::Debug for VarAssignment {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("VarAssignment")
            .field("identifier", &self.name)
            .field("value", &self.value)
            .finish()
    }
}

impl fmt::Debug for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Identifier")
            .field("name", &self.name)
            .finish()
    }
}