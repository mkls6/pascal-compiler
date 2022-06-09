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