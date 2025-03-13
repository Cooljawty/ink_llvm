use crate::ast::{Expression};

#[derive(Debug,)]
pub enum Value {
    Integer,
    Decimal,
    String,
    Bool,
    Divert, //NOTE: Divert values cannot contain arguments
    ListValue,
}

#[allow(dead_code)]
pub struct Choice<'ast> {
    index: usize,

    condition: Option<Expression<'ast>>,

    text: String,
    choice_text: String,
    post_text: String,

    destination: Value,
}

