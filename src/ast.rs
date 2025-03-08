use crate::types::Value;

pub(crate) enum Subprogram {
    Knot,
    Stitch,
    Function,
}

pub(crate) enum Content {
    Text,
    Alternative,
    Conditional,
    Logic,
}

pub(crate) enum Expression<'ast>  {
    Literal(Value),
    Variable,
    Constant(Value),
    UnaryOp(Operation, &'ast Expression<'ast>),
    BinOp(Operation, &'ast Expression<'ast>, &'ast Expression<'ast>),
}

pub(crate) enum Operation {
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
