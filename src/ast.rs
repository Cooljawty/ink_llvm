use std::collections::HashMap;
use crate::types::{Value};

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
#[derive(Debug)]
pub struct Knot<I> {
    pub(crate) signature: Callable, 
    pub(crate) root: Stitch<I>, 
    pub(crate) body: Vec<Stitch<I>>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Stitch<I> {
    pub(crate) signature: Callable,
    pub(crate) body: I
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Function<I> {
    pub(crate) signature: Callable,
    pub(crate) body: I
}

#[allow(dead_code)]
pub struct Weave<I> {
    label: Option<Identifier>,

    content: Vec<Content<I>>,
    choices: Vec<(Choice<I>, Option<Box<Weave<I>>>)>,
    gather: Option<Box<Weave<I>>>, //Holds address of next Box<Weave> in chain
}

pub struct Choice<I> {
    level: usize,
    label: Option<Identifier>,

    condition: Option<Vec<Expression>>,

    text: Content<I>,
    choice_text: Content<I>,
    post_text: Content<I>,
}

pub struct ChoiceBlock<I> {
    comparison: Option<Expression>,
    cases: Vec<(Expression, (Choice<I>, Branch))>,
    default: Option<Vec<(Expression, (Choice<I>, Branch))>>,
}


pub enum Target {
    Callable,
    Weave,
    Choice, //Yeah... you can do that
}

pub enum Branch {
    Divert(Identifier), // -> <Callable>
    Tunnel(Identifier), // -> <Callable> -> Divert | Tunnel
    Thread(Identifier), // <- <Callable>

    ReturnTunnel, // ->->
    Return(Option<Expression>), // ~ return <Expression>

    Done, // -> DONE
    End, // -> END
}

pub trait ConditionList: {
    type Item;

    type Comparison;
    type Cases;
    type Default;
}

#[allow(dead_code)]
#[derive(Clone, Debug,)]
pub struct Parameter {
    pub(crate) name: Identifier,
    pub(crate) refrence: bool,
    pub(crate) is_divert: bool,
}

pub struct Alternative<I> { 
    cases: HashMap<usize, Vec<Content<I>>>,

    method: AlternateType,
    shuffle: bool,

}
pub enum AlternateType { Once, Cycle, Stopping, }

pub struct Conditional<I> {
    cases: Vec<(Expression, Vec<Content<I>>)>,
    default: Option<Vec<Content<I>>>,
}                                                 

pub struct Switch<I> {                            
    comparision: Expression,                      
    cases: Vec<(Expression, Vec<Content<I>>)>,
    default: Option<Vec<Content<I>>>,
}

pub enum Content<I> {
    Logic(Expression),
    Evaluation(Expression),
    Alternative(Alternative<I>),
    Conditional(Conditional<I>),
    Switch(Switch<I>),
    Branch(Branch),
    Text(I),
}

pub enum Expression  {
    Literal(Value),
    Variable,
    Constant(Value),
    UnaryOp(Operation, Box<Expression>),
    BinOp(Operation, Box<Expression>, Box<Expression>),
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
