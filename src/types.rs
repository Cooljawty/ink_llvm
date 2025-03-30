use crate::ast::{Expression};

#[derive(Clone, PartialEq, Debug,)]
pub enum Value {
    Integer(isize),
    Decimal(f32),
    String(String),
    Bool(bool),
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

