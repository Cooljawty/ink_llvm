use crate::types::Value;


pub type Identifier = String;

#[allow(dead_code)]
#[derive(Debug,)]
pub struct Callable {
    pub(crate) ty: Subprogram,
    pub(crate) name: Identifier,
    pub(crate) parameters: Vec<Parameter>
}

#[derive(Clone, PartialEq, Debug,)]
pub enum Subprogram {
    Knot,
    Stitch,
    Function,
}

#[allow(dead_code)]
#[derive(Clone, Debug,)]
pub struct Parameter {
    pub(crate) name: Identifier,
    pub(crate) refrence: bool,
    pub(crate) is_divert: bool,
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
