use crate::ast::{Expression, Content};

pub enum Value {
    Integer,
    Decimal,
    String,
    Bool,
    Divert, //NOTE: Divert values cannot contain arguments
    ListValue,
}

pub struct Choice<'ast> {
    index: usize,

    condition: Option<Expression<'ast>>,

    text: String,
    choice_text: String,
    post_text: String,

    destination: Value,
}

struct Weave<'ast> {
    content: Vec<Content>,
    choices: Vec<Choice<'ast>>,
}

