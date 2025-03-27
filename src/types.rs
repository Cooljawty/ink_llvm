use crate::ast::{Expression};

#[derive(Clone, Debug,)]
pub enum Value {
    Integer,
    Decimal,
    String,
    Bool,
    Divert, //NOTE: Divert values cannot contain arguments
    ListValue,
}

#[allow(dead_code)]
pub struct Choice {
    index: usize,

    condition: Option<Expression>,

    text: String,
    choice_text: String,
    post_text: String,

    destination: Value,
}

