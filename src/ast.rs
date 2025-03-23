use std::collections::HashMap;
use crate::types::{Value};

pub type Identifier = String;

#[allow(dead_code)]
#[derive(Debug,)]
pub struct Signature {
    pub(crate) name: Identifier,
    pub(crate) parameters: Vec<Parameter>,
    pub(crate) ret: Option<Value>,
}

pub trait Subprogram<I> where 
    for<'p> I: nom::Input + nom::Offset + nom::Compare<&'p str> + nom::FindSubstring<&'p str>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'p> &'p str: nom::FindToken<<I as nom::Input>::Item>
{
    type Body;

    fn parse(input: I) -> nom::IResult<I, Self> where Self: Sized;

    fn parse_signature(input: I) -> nom::IResult<I, Signature>;

    fn parse_body(input: I) -> nom::IResult<I, Self::Body>;
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Knot<I> {
    pub(crate) signature: Signature, 
    pub(crate) root: Stitch<I>, 
    pub(crate) body: Vec<Stitch<I>>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Stitch<I> {
    pub(crate) signature: Signature,
    pub(crate) body: I
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Function<I> {
    pub(crate) signature: Signature,
    pub(crate) body: I
}

#[allow(dead_code)]
pub struct Weave<I> {
    label: Option<Identifier>,

    content: Vec<Content<I>>,
    choices: Vec<(Choice<I>, Option<Box<Weave<I>>>)>,
    gather: Option<Box<Weave<I>>>, //Holds address of next Box<Weave> in chain
}

#[allow(dead_code)]
pub struct Choice<I> {
    level: usize,
    label: Option<Identifier>,

    condition: Option<Vec<Expression>>,

    text: Content<I>,
    choice_text: Content<I>,
    post_text: Content<I>,
}

#[allow(dead_code)]
pub struct ChoiceBlock<I> {
    comparison: Option<Expression>,
    cases: Vec<(Expression, (Choice<I>, Branch))>,
    default: Option<Vec<(Expression, (Choice<I>, Branch))>>,
}


#[allow(dead_code)]
pub enum Target {
    Signature,
    Weave,
    Choice, //Yeah... you can do that
}

#[allow(dead_code)]
pub enum Branch {
    Divert(Identifier), // -> <Signature>
    Tunnel(Identifier), // -> <Signature> -> Divert | Tunnel
    Thread(Identifier), // <- <Signature>

    ReturnTunnel, // ->->
    Return(Option<Expression>), // ~ return <Expression>

    Done, // -> DONE
    End, // -> END
}

#[allow(dead_code)]
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

#[allow(dead_code)]
pub struct Alternative<I> { 
    cases: HashMap<usize, Vec<Content<I>>>,

    method: AlternateType,
    shuffle: bool,

}
#[allow(dead_code)]
pub enum AlternateType { Once, Cycle, Stopping, }

#[allow(dead_code)]
pub struct Conditional<I> {
    cases: Vec<(Expression, Vec<Content<I>>)>,
    default: Option<Vec<Content<I>>>,
}                                                 

#[allow(dead_code)]
pub struct Switch<I> {                            
    comparision: Expression,                      
    cases: Vec<(Expression, Vec<Content<I>>)>,
    default: Option<Vec<Content<I>>>,
}

#[allow(dead_code)]
pub enum Content<I> {
    Logic(Expression),
    Evaluation(Expression),
    Alternative(Alternative<I>),
    Conditional(Conditional<I>),
    Switch(Switch<I>),
    Branch(Branch),
    Text(I),
}

#[allow(dead_code)]
pub enum Expression  {
    Literal(Value),
    Variable,
    Constant(Value),
    UnaryOp(Operation, Box<Expression>),
    BinOp(Operation, Box<Expression>, Box<Expression>),
}

#[allow(dead_code)]
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
