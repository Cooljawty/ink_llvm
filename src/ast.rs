use crate::types::Value;

#[derive(Clone,)]
pub enum Subprogram {
    Knot,
    Stitch,
    Function,
}

pub enum Content {
    Text,
    Alternative,
    Conditional,
    Logic,
}

pub enum Expression<'ast>  {
    Literal(Value),
    Variable,
    Constant(Value),
    UnaryOp(Operation, &'ast Expression<'ast>),
    BinOp(Operation, &'ast Expression<'ast>, &'ast Expression<'ast>),
}

pub enum Operation {
    ///Logical:
    And,
    Or,
    Not,
    ///Strings:
    Equal,
    NotEqual,
    Contains,
    ///Mathmatical:
    Add,
    Subtract,
    Multiply,
    Divide,
    Mod,
    Negate,
}
