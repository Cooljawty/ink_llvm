macro_rules! print_nom_input {
    ( $($input:expr),* ) => {
        $(
            eprintln!("{:name_w$}|{:>count_w$}|: {:?}",
                stringify!($input), $input.input_len(), 
                $input.iter_elements().map(|c| c.as_char()).collect::<String>(),
                name_w = 6, count_w = 3,
            );
        )*
    }
}

use std::collections::HashMap;

#[allow(unused_imports)]
use nom::{
    IResult,
    AsChar,
    Parser,
    multi::*,
    bytes::{tag, is_a, is_not, take_until,take_till},
    character::{
        anychar,one_of,
        complete::*,
    },
    number::float,
    combinator::*,
    branch::{alt,},
    sequence::{delimited, preceded,},
};

use crate::{ast, ast::Subprogram, };

pub trait Parse<I> 
where
    for<'p> I: nom::Input + nom::Offset + nom::Compare<&'p str> + nom::FindSubstring<&'p str>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'p> &'p str: nom::FindToken<<I as nom::Input>::Item>
{
    fn parse(input: I) -> nom::IResult<I, Self> where Self: Sized { fail().parse(input) }
}

impl<I> ast::Story<I> 
where
	for<'p> I: nom::Input + nom::Offset + nom::Compare<&'p str> + nom::FindSubstring<&'p str> + std::fmt::Debug,
    <I as nom::Input>::Item: nom::AsChar,
    for<'p> &'p str: nom::FindToken<<I as nom::Input>::Item>,
{
    pub fn parse(input: I) -> IResult<I, Self>
    {
        #[cfg(debug_assertions)]
        {
            print_nom_input!(input);
            eprintln!("== __root ==");
        }

        let (remaining, (root_root_stitch, root_stitches)) = ast::Knot::parse_body.parse(input)?;
        let root_knot = ast::Knot {
            signature: ast::Signature{ name: "__root".into(), parameters: vec![], ret: None},
            root:      root_root_stitch,
            body:      root_stitches
        };

        #[cfg(debug_assertions)]
        {
            if root_knot.body.len() == 0 { eprintln!("= ="); }
            eprintln!("== ==");
        }

        let (remaining, ((knots, functions), _)) = (
            fold_many0(
                complete(Self::knot_or_function),
                ||(Vec::new(), Vec::new()),
                |(mut knot_acc, mut function_acc), (knot, function)|
                {
                    match (knot, function) {
                        (Some(knot), None) => { knot_acc.push(knot); }, 
                        (None, Some(function)) => { function_acc.push(function); }, 
                        //TODO: Convert panics to errors
                        (None, None) => { panic!("Knot_or_Function returned neither knot nor function, or both"); }
                        (Some(_), Some(_)) => { panic!("Knot_or_Function returned both a knot and function"); }
                    };
                    (knot_acc, function_acc)
                }
            ),
            alt((line_ending, eof))
        ).parse(remaining)?;

        Ok((remaining, ast::Story(root_knot, knots, functions)))
    }

    //Must return a knot or function. Fails if neither parser succeeds
    fn knot_or_function(input: I) -> IResult<I, (Option<ast::Knot<I>>, Option<ast::Function<I>>)>
    {   
        Ok(match opt(ast::Knot::parse).parse(input)? {
            (rem, Some(knot)) => (rem, (Some(knot), None)),
            (rem, None) => {
                let (rem, function) = ast::Function::parse.parse(rem)?;
                (rem, (None, Some(function))) 
            }
        })
    }

    fn text_body(input: I) -> IResult<I, I> 
    { 
        
        if input.input_len() == 0 { return Ok((input.clone(), input.take_from(0))) }
        
        let rem = input.clone();
        let mut body_size = 0;
        let (rem, body) = loop {
            let (rem, line) = peek(recognize((not_line_ending, opt(line_ending)))).parse(rem.take_from(body_size))?;

            let res: IResult<I, ()> = peek(not((
                space0, 
                alt(
                    (eof, tag("="))
                )
            ))).parse(line.clone());
            if res.is_ok() { 
                body_size += line.input_len(); 
            }
            else { 
                break (rem, input.take(body_size)) 
            }
        };

        #[cfg(debug_assertions)]
        {
            print_nom_input!(body, rem);
        }

        Ok((rem, body))
    }

}

//Knots and Stitches
impl<I> ast::Subprogram<I> for ast::Knot<I> 
where
    for<'p> I: nom::Input + nom::Offset + nom::Compare<&'p str> + nom::FindSubstring<&'p str> + std::fmt::Debug,
    <I as nom::Input>::Item: nom::AsChar,
    for<'p> &'p str: nom::FindToken<<I as nom::Input>::Item>,
{
    fn parse(input: I) -> IResult<I, ast::Knot<I>> 
    {   
        let (rem, signature) = ast::Knot::parse_signature.parse(input)?;

        #[cfg(debug_assertions)]
        { 
            eprintln!("\n== {} ==", signature.name); 
        }

        let (rem, (root, body)) = ast::Knot::parse_body.parse(rem)?;

        #[cfg(debug_assertions)]
        {
            eprintln!("== ==");
            print_nom_input!(rem);
        }

        Ok ((rem, ast::Knot { signature, root, body }))
    }

    fn parse_signature(input: I) -> IResult<I, ast::Signature> 
    { 
        let (rem, (_, (name, _, parameters), _)) =
        (
            (space0, tag("=="), opt(is_a("=")), space0),
            (identifier, space0, opt(parameter_list)),
            (space0, opt(is_a("=")), line_ending)
        ).parse(input)?;
        
        Ok((rem, ast::Signature{
            name: name.into(), 
            parameters: parameters.unwrap_or(vec![]), 
            ret: None,
        }))
    }

    type Body = (ast::Stitch<I>, Vec<ast::Stitch<I>>);

    fn parse_body(input: I) -> IResult<I, Self::Body> 
    {
        //Note: Knot body consumes input diffrently than the text_body parser.
        //      Knots contain nested stitches. 
        //      Thus it needs to search for a full "==" tag instead of any line starting with a '='.
        let (rem, body) = match take_until("==").parse(input.clone()) {
            //If parser never reaches the tag then assume all input is in the body
            nom::IResult::Err(nom::Err::Incomplete(_)) => {
                input.take_split(input.input_len()-1)
            },
            //Check if the remaining line is infact a signature
            Ok((rem, body)) => {
                value(body, peek(recognize(alt((ast::Knot::parse_signature, ast::Function::parse_signature))))).parse(rem.clone())?
            },
            err => err?
        };
        
        let (_, (body, stitches)) = (ast::Stitch::parse_body, many0(complete(ast::Stitch::parse))).parse(body)?;

        let root_stitch = ast::Stitch {
            signature: ast::Signature{ 
                name: "__root".into(), 
                parameters: vec![], 
                ret: None,
            }, 
            body,
        };

        #[cfg(debug_assertions)]
        {
            print_nom_input!(rem);
        }

        Ok((rem, (root_stitch, stitches)))
    }

}

impl<I> ast::Subprogram<I> for ast::Stitch<I> 
where
    for<'p> I: nom::Input + nom::Offset + nom::Compare<&'p str> + nom::FindSubstring<&'p str> + std::fmt::Debug,
    <I as nom::Input>::Item: nom::AsChar,
    for<'p> &'p str: nom::FindToken<<I as nom::Input>::Item>,
{
    fn parse(input: I) -> IResult<I, ast::Stitch<I>>
    {   
        let (rem, signature) = ast::Stitch::parse_signature.parse(input)?;

        #[cfg(debug_assertions)] {
            eprintln!("\n= {}", signature.name);
        }

        let (rem, body) = ast::Stitch::parse_body.parse(rem)?;

        #[cfg(debug_assertions)] {
            eprintln!("= =");
        }

        Ok((rem, ast::Stitch{signature, body}))
    }

    fn parse_signature(input: I) -> IResult<I, ast::Signature> 
    { 
        let (rem, (_, (name, _, parameters), _)) =
        (
            (space0, tag("="), space0),
            (identifier, space0, opt(parameter_list)),
            (space0, line_ending)
        ).parse(input)?;
        
        Ok((rem, ast::Signature{
            name: name.into(), 
            parameters: parameters.unwrap_or(vec![]), 
            ret: None,
        }))
    }

    type Body = I;

    fn parse_body(input: I) -> IResult<I, Self::Body> { ast::Story::text_body(input) }
}

//Functions
impl<I> ast::Subprogram<I> for ast::Function<I> 
where
    for<'p> I: nom::Input + nom::Offset + nom::Compare<&'p str> + nom::FindSubstring<&'p str> + std::fmt::Debug,
    <I as nom::Input>::Item: nom::AsChar,
    for<'p> &'p str: nom::FindToken<<I as nom::Input>::Item>,
{
    fn parse(input: I) -> IResult<I, ast::Function<I>>
    {   
        let (rem, signature) = ast::Function::parse_signature.parse(input)?;

        #[cfg(debug_assertions)]
        {
            eprintln!("\n== function {}() ==", signature.name);
        }

        let (rem, body) = ast::Function::parse_body.parse(rem)?;

        #[cfg(debug_assertions)]
        {
            eprintln!("== ==");
        }

        Ok((rem, ast::Function{signature, body}))
    }

    fn parse_signature(input: I) -> IResult<I, ast::Signature> 
    { 
        let (rem, (_, (name, _, parameters), _)) =
        (
            (space0, tag("=="), opt(is_a("=")), space0, tag("function"), space0),
            (identifier, space0, opt(parameter_list)),
            (space0, opt(is_a("=")), line_ending)
        ).parse(input)?;
        
        Ok((rem, ast::Signature{
            name: name.into(), 
            parameters: parameters.unwrap_or(vec![]), 
            ret: None,
        }))
    }

    type Body = I;
    fn parse_body(input: I) -> IResult<I, Self::Body> { ast::Story::text_body(input) }

}

fn collect_input<I, T>(input: I) -> T
where
    T: FromIterator<char>,

	for<'p> I: nom::Input + nom::Offset + nom::Compare<&'p str> + nom::FindSubstring<&'p str>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'p> &'p str: nom::FindToken<<I as nom::Input>::Item>,
{
    input.iter_elements().map(|c| c.as_char()).collect::<T>()
}

fn identifier<I>(input: I) -> IResult<I, ast::Identifier> 
where
	for<'p> I: nom::Input + nom::Offset + nom::Compare<&'p str> + nom::FindSubstring<&'p str>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'p> &'p str: nom::FindToken<<I as nom::Input>::Item>,
{
    //TODO: Add parsers for non-ASCII characters
    let (rem, name) = recognize(( 
        alt(( tag("_"), alpha1 )),
        many0( alt( (tag("_"), alphanumeric1) ) ) 
    )).parse(input)?;
    
    Ok((rem, collect_input(name)))
}

fn parameter_list<I>(input: I) -> IResult<I, Vec<ast::Parameter>> 
where
	for<'p> I: nom::Input + nom::Offset + nom::Compare<&'p str> + nom::FindSubstring<&'p str>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'p> &'p str: nom::FindToken<<I as nom::Input>::Item>,
{
    let (rem, (_, param_list, _)) = (
        (tag("("), space0),
            separated_list0(
                (space0, tag(","), space0),
                (
                    opt(tag("ref")), space0, opt(tag("->")), 
                    space0, 
                    identifier 
                ),
            ),
        (space0, tag(")")),
    ).parse(input)?;

    let param_list = param_list.iter().map(|(is_ref, _, is_divert, _, name)| { 
        ast::Parameter{ 
            name: name.to_string(), 
            refrence: is_ref.is_some(), 
            is_divert: is_divert.is_some() 
        } 
    }).collect();

    Ok((rem, param_list))
}

//Content
impl<I> ast::Content<I>
where
    for<'p> I: nom::Input + nom::Offset + nom::Compare<&'p str> + nom::FindSubstring<&'p str> + std::fmt::Debug + nom::ParseTo<f32>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'p> &'p str: nom::FindToken<<I as nom::Input>::Item>,
{

    #[allow(dead_code)]
    pub fn parse(input: I) -> IResult<I, Self>
    {
        /* TODO
        if let (rem, expr) = ((space0, tag("~"), space0), ast::Expression::parse).parse(input)
        {
        }
        else if let (rem, branch) = ast::Branch::parse(input) 
        {
        }
        */
        if      let Ok((rem, _block)) = peek(char::<I, nom::error::Error<I>>('{')).parse(input.clone()) { Self::parse_block(rem) }
        else if let Ok((rem, _newline)) = many1(line_ending::<I, nom::error::Error<I>>).parse(input.clone()) { Ok( (rem, ast::Content::Newline) ) }
        else { Self::parse_text(input, "\\\n{}") }
    }

    #[allow(dead_code)]
    fn parse_block(input: I) -> IResult<I, Self>
    {
        let (rem, block) = delimited(
            tag("{"), 
            alt((
                map(ast::Switch::parse,      |switch| Self::Switch(switch)), //TODO: Solve precedence issue
                map(ast::Conditional::parse, |conditional| Self::Conditional(conditional)),
                map(ast::Alternative::parse, |alternative| Self::Alternative(alternative)),
                //TODO: map(ast::Expression::parse,  |expr| Self::Evaluation(expr)),
            )),
            tag("}")
        ).parse(input)?;
        Ok((rem, block))
    }

   fn parse_text(input: I, delimiters: &'static str) -> IResult<I, Self> 
   {
        let (input, text_length) = peek(fold_many1(
            alt((
                complete(verify(is_not(delimiters), |fragment: &I|fragment.input_len() > 0)), //Literal
                complete(recognize(preceded(char('\\'), is_not("\r\n")))), //Escaped char
            )),
            || 0usize,
            |text_length, fragment: I| text_length + fragment.input_len() 
        )).parse(input)?;

        let (rem, text) = input.take_split(text_length);
        
        //Escaped whitespace
        let (rem, _) = opt(recognize(preceded(char('\\'), multispace1))).parse(rem)?;

        Ok( (rem, Self::Text(text)) )
    }
}

fn condition_list_block<I, Expr, Case, Cmp, Sep, Text>(
    cmp_parser: Cmp,
    case_separater: Sep,
    case_text_parser: Text,
) -> impl nom::Parser<I, Output = (Expr, Vec<(Case, Vec<ast::Content<I>>)>), Error = nom::error::Error<I>>
where
    Cmp:  nom::Parser<I, Output = Expr, Error = nom::error::Error<I>>,
    Sep:  nom::Parser<I, Output = Case, Error = nom::error::Error<I>>,
    Text: nom::Parser<I, Output = I, Error = nom::error::Error<I>>,

    for<'p> I: nom::Input + nom::Offset + nom::Compare<&'p str> + nom::FindSubstring<&'p str> + std::fmt::Debug + nom::ParseTo<f32>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'p> &'p str: nom::FindToken<<I as nom::Input>::Item>,
{
    let case = |text_parser|{
        fold_many0(
        alt(( 
            map_parser(
                text_parser,
                many0(complete(ast::Content::parse)),
            ),
            map(
                ast::Content::parse_block,
                |block|{ vec![block] },
            ),
        )),
        Vec::<ast::Content<I>>::new,
        |mut acc, content|{acc.extend(content); acc},
        )
    };
    
    (
        cmp_parser,

        many1( ( case_separater, case(case_text_parser) ), ),
    )
}

fn condition_list_inline<I, Expr, Cmp, Sep, Text>(
    cmp_parser: Cmp,
    case_separater: Sep,
    case_text_parser: Text,
) -> impl nom::Parser<I, Output = (Expr, Vec<Vec<ast::Content<I>>>), Error = nom::error::Error<I>>
where
    Cmp:  nom::Parser<I, Output =  Expr, Error = nom::error::Error<I>>,
    Sep:  nom::Parser<I, Output = I, Error = nom::error::Error<I>>,
    Text: nom::Parser<I, Output = I, Error = nom::error::Error<I>>,

    for<'p> I: nom::Input + nom::Offset + nom::Compare<&'p str> + nom::FindSubstring<&'p str> + std::fmt::Debug + nom::ParseTo<f32>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'p> &'p str: nom::FindToken<<I as nom::Input>::Item>,
{
    let case = |text_parser|{
        fold_many0(
        alt(( 
            map_parser(
                text_parser,
                many0(complete(ast::Content::parse)),
            ),
            map(
                ast::Content::parse_block,
                |block|{ vec![block] },
            ),
        )),
        Vec::<ast::Content<I>>::new,
        |mut acc, content|{acc.extend(content); acc},
        )
    };
    
    (
        cmp_parser,

        separated_list1( case_separater, case(case_text_parser)),
    )
}

impl<I> Parse<I> for ast::Alternative<I> where
    for<'p> I: nom::Input + nom::Offset + nom::Compare<&'p str> + nom::FindSubstring<&'p str> + std::fmt::Debug + nom::ParseTo<f32>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'p> &'p str: nom::FindToken<<I as nom::Input>::Item>,
{ 
    fn parse(input: I) -> IResult<I, Self> { 
        let cmp_parser_block = map(
            (
            alt(( 
                value((ast::AlternateType::Stopping, false), tag("stopping")),
                value((ast::AlternateType::Once, false),     tag("once")),
                value((ast::AlternateType::Cycle, false),    tag("cycle")), 
                value((ast::AlternateType::Stopping, true),  (tag("shuffle"), space0, tag("stopping"))),
                value((ast::AlternateType::Once, true),      (tag("shuffle"), space0, tag("once"))),
                value((ast::AlternateType::Cycle, true),     tag("shuffle"))
            )),
            recognize((space0, tag(":"), space0)),
            ),
            |((method, shuffle), _)|(method, shuffle)
        );
        let case_separater_block = recognize((line_ending, space0, tag("-"), space0)); 
        let case_text_parser_block = recognize( many1(
            peek(not(( line_ending, space0, tag("-"))))
            .and(is_not("\n{}"))
        )); 

        let cmp_parser_inline = alt(( 
            value((ast::AlternateType::Stopping, false), tag("!")),
            value((ast::AlternateType::Cycle, false),    tag("&")), 
            value((ast::AlternateType::Cycle, true),     tag("~")),
            success((ast::AlternateType::Once, false)),
        ));
        let case_separater_inline = tag("|");
        let case_text_parser_inline = is_not("\n|{}"); //TODO: Allow delimited line breaks (Move into function?)


        let (rem, ( ( (method, shuffle), cases), _)) = (
            map(
                condition_list_block( cmp_parser_block, case_separater_block, case_text_parser_block ),
                |( method, cases)| ( method, cases.into_iter().map(|(_, content)|content).collect())
            ),
            multispace0
        ).or((
            condition_list_inline( cmp_parser_inline, case_separater_inline, case_text_parser_inline ),
            space0
        )).parse(input)?;

        let cases = HashMap::from_iter(
            cases.into_iter().enumerate()
                //TODO: Represent {..||..} as None or empty vec
                //.filter(|(_index, cases)|!cases.is_empty())
        );

        Ok((rem, ast::Alternative{ cases, method, shuffle }))
    }
} 

impl<I> Parse<I> for ast::Conditional<I> where
    for<'p> I: nom::Input + nom::Offset + nom::Compare<&'p str> + nom::FindSubstring<&'p str> + std::fmt::Debug + nom::ParseTo<f32>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'p> &'p str: nom::FindToken<<I as nom::Input>::Item>,
{ 
    fn parse(input: I) -> IResult<I, Self> { 
        let (rem, ( ( _, cases), default, _)) = (
            condition_list_block(
                success(()),

                map(
                    (
                        (line_ending, space0, tag("-"), space0),
                        map(not(tag("else")).and(ast::Expression::parse), |(_, expr)|expr),
                        (space0, tag(":"), space0)
                    ),
                    |(_, expr, _)|expr
                ),

                recognize( many1(
                    peek(not(( line_ending, space0, tag("-"))))
                    .and(is_not("\n{}"))
                )) 
            ),
            opt( map( 
                    (
                        (line_ending, space0, tag("-"), space0, tag("else"), space0, tag(":"), space0),
                        map_parser(is_not("\n{}"), many1(ast::Content::parse)),
                    ),
                    |(_, content)| content,
            )),
            multispace0
        ).or((
            map(
                (
                    success(()),
                    (
                        map(((space0), ast::Expression::parse, (space0, tag(":"))), |(_, expr, _)|expr),
                        map_parser(is_not("\n{}"), many1(ast::Content::parse)),
                    ),
                ),
                |(cmp, cases)|(cmp, vec![cases]),
            ),
            opt( map( 
                    (
                        tag("|"),
                        map_parser(is_not("\n{}"), many1(ast::Content::parse)),
                    ),
                    |(_, content)| content,
            )),
            space0
        )).parse(input)?;

        let switch = ast::Conditional{ 
            cases,
            default,
        };

        Ok((rem, switch ))
    }
} 

impl<I> Parse<I> for ast::Switch<I> 
where
    for<'p> I: nom::Input + nom::Offset + nom::Compare<&'p str> + nom::FindSubstring<&'p str> + std::fmt::Debug + nom::ParseTo<f32>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'p> &'p str: nom::FindToken<<I as nom::Input>::Item>,
{ 
    fn parse(input: I) -> IResult<I, Self> { 
        let (rem, ( ( compairison, cases), default, _)) = (
            condition_list_block(
                map((ast::Expression::parse, (space0, tag(":"), space0)), |(expr, _)|expr),

                map(
                    (
                        (line_ending, space0, tag("-"), space0),
                        map(not(tag("else")).and(ast::Expression::parse), |(_, expr)|expr),
                        (space0, tag(":"), space0)
                    ),
                    |(_, expr, _)|expr
                ),

                recognize( many1(
                    peek(not(( line_ending, space0, tag("-"))))
                    .and(is_not("\n{}"))
                )) 
            ),
            opt( map( 
                    (
                        (line_ending, space0, tag("-"), space0, tag("else"), space0, tag(":"), space0),
                        map_parser(is_not("\n{}"), many1(ast::Content::parse)),
                    ),
                    |(_, content)| content,
            )),
            multispace0
        ).parse(input)?;

        let switch = ast::Switch{ 
            compairison, 
            cases,
            default,
        };

        Ok((rem, switch ))
    }
} 

impl<I> Parse<I> for ast::Expression 
where
    for<'p> I: nom::Input + nom::Offset + nom::Compare<&'p str> + nom::FindSubstring<&'p str> + std::fmt::Debug + nom::ParseTo<f32>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'p> &'p str: nom::FindToken<<I as nom::Input>::Item>,
{
    fn parse(input: I) -> IResult<I, Self>  
    { 
        use crate::types::Value;

        use std::str::FromStr;
        let int_parser = recognize((opt(tag("-")), digit1));
        let float_parser = recognize((opt(tag("-")), digit1, tag("."), digit1));

        println!("Parsing Expression");
        print_nom_input!(input);

        let (rem, (unary_op, _)) = (
            opt( alt((
                value(ast::Operation::Not, alt((tag("not"), tag("!")))),
                value(ast::Operation::Negate, tag("-")),
            ))),
            space0,
        ).parse(input)?;

        let (rem, expr): (I, Self) = alt((
            //Parens
            delimited( tag("("), Self::parse, tag(")") ),

            //String Expression
            map(
                (
                    tag("\""), 
                    recognize(many0(
                        |input| ast::Content::parse_text(input, "\\\n{}\"")
                    )),
                    tag("\"")
                ),
                |(_, string, _)|Self::Literal(Value::String(collect_input(string))),
            ),

            //Numbers (Integers and decimals)
            map(
                map_res(float_parser, |num: I| {
                    f32::from_str(num.iter_elements().map(|c| c.as_char()).collect::<String>().as_str())
                }),
                |num| Self::Literal(Value::Decimal(num))
            ),
            map(
                map_res(int_parser, |num: I| {
                    isize::from_str(num.iter_elements().map(|c| c.as_char()).collect::<String>().as_str())
                }),
                |num| Self::Literal(Value::Integer(num))
            ),
            
            //Boolean 
            value(Self::Literal(Value::Bool(true)), tag("true")),
            value(Self::Literal(Value::Bool(false)), tag("false")),

            //Variable
            map(identifier, |name| Self::Variable(name)),

        )).parse(rem)?;

        let expr = if let Some(op) = unary_op {
            Self::UnaryOp(op, Box::new(expr))
        } else {
            expr
        };

        let (rem, expr) = if let Ok((rem, (_, op, _))) = ( space0, ast::Operation::parse, space0 ).parse(rem.clone()) {
            Self::parse_binop(rem, op, expr)?
        } else {
            (rem, expr)
        };

        print_nom_input!(rem);
        println!("End Expression");

        Ok((rem, expr))
    }
}

impl ast::Expression 
{
    fn parse_binop<I>(input: I, root_op: ast::Operation, left_expr: ast::Expression) -> IResult<I, Self>  
    where
        for<'p> I: nom::Input + nom::Offset + nom::Compare<&'p str> + nom::FindSubstring<&'p str> + std::fmt::Debug + nom::ParseTo<f32>,
        <I as nom::Input>::Item: nom::AsChar,
        for<'p> &'p str: nom::FindToken<<I as nom::Input>::Item>,
    { 
        println!("Parsing Binop");
        print_nom_input!(input);

        let (rem, right_expr) = Self::parse(input)?;

        let expr = match right_expr {
            Self::BinOp(right_op, right_expr_left, right_expr_right)
            if right_op.precedence() < root_op.precedence() => {
                //Tree swap!!
                Self::BinOp(
                    right_op,
                    Box::new(
                        Self::BinOp(
                            root_op,
                            Box::new(left_expr),
                            right_expr_left,
                        ),
                    ),
                    right_expr_right,
                )
            },

            right_expr => {
                Self::BinOp(root_op, Box::new(left_expr), Box::new(right_expr))
            },
        };

        print_nom_input!(rem);
        println!("End Binop");

        Ok(( rem, expr ))
    }
}

impl<I> Parse<I> for ast::Operation 
where
    for<'p> I: nom::Input + nom::Offset + nom::Compare<&'p str> + nom::FindSubstring<&'p str> + std::fmt::Debug + nom::ParseTo<f32>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'p> &'p str: nom::FindToken<<I as nom::Input>::Item>,
{
    fn parse(input: I) -> IResult<I, Self>  
    { 
        alt((
            value(ast::Operation::And, alt((tag("and"), tag("&&")))),
            value(ast::Operation::Or, alt((tag("or"), tag("||")))),
            value(ast::Operation::Not, alt((tag("not"), tag("!")))),

            value(ast::Operation::Equal, tag("==")),
            value(ast::Operation::NotEqual, tag("!=")),
            value(ast::Operation::Contains, tag("?")),

            value(ast::Operation::Add, tag("+")),
            value(ast::Operation::Subtract, tag("-")),
            value(ast::Operation::Multiply, tag("*")),
            value(ast::Operation::Divide, tag("/")),
            value(ast::Operation::Mod, alt((tag("mod"), tag("%")))),

            //See ast::Expression::parse for unary negation
            //value(ast::Operation::Negate, tag("-")),
        )).parse(input)
    }
}
impl ast::Operation
{
    fn precedence(&self) -> usize  
    { 
        match self {
            Self::Negate | Self::Not => 1<<0,

            Self::Equal | Self::NotEqual | Self::Contains => 1<<1,
            Self::Add | Self::Subtract => 1<<2,

            Self::Multiply | Self::Divide | Self::Mod => 1<<3,

            Self::And | Self::Or  => 1<<4,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use nom::Input;

    #[test]
    fn parse_knots_with_root() -> Result<(), Box<dyn std::error::Error>>    {
        let (unparsed, ast::Story(root, knots, _functions)) = ast::Story::parse(include_str!("../tests/knots_with_root.ink"))?;

        match eof::<&str,nom::error::Error<&str>>.parse(unparsed) {
            Ok(_) => {},
            _ => assert!(false, "Incomplete parse. Remaining text: {}:'{}'", unparsed.input_len(), unparsed),
        }

        match root {
            root @ ast::Knot{ signature: ast::Signature { ret: None, ..}, ..  } => {
                assert_eq!(root.signature.name, "__root");
                assert_ne!(root.root.body.trim(), "", "Root body parse error");
                assert_ne!(root.body[0].body.trim(), "", "Root stitch body parse error");
            },
            _ => {
                panic!("Invalid parse. Error with root\nRemaining: \n{:?}\n---", unparsed);
            }
        };

        match knots.as_slice() {
            [
                k1 @ ast::Knot{ signature: ast::Signature { ..}, ..  }, 
                k2 @ ast::Knot{ signature: ast::Signature { ..}, ..  }, 
            ] => {
                assert_eq!(k1.signature.name, "K1");
                assert!(matches!(k1.signature.ret, None));
                assert_ne!(k1.root.body.trim(), "", "K1 body parse error");

                assert_eq!(k1.body[0].signature.name, "K1_1", "K1_1 body parse error");
                assert!(matches!(k1.signature.ret, None));
                assert_ne!(k1.body[0].body.trim(), "", "K1_1 body parse error");

                assert_eq!(k2.signature.name, "K2");
                assert!(matches!(k2.signature.ret, None));
                assert_eq!(k2.root.body.trim(), "", "K1 body parse error");

                assert_eq!(k2.body[0].signature.name, "K2_1", "K2_1 body parse error");
                assert!(matches!(k2.body[0].signature.ret, None));
                assert_ne!(k2.body[0].body.trim(), "", "K2_1 body parse error");

            },
            _ => {
                panic!("Invalid parse.\nFounc {} knots, expected {}\nRemaining: \n{:?}\n---", knots.len(), 2, unparsed);
            }
        };

        Ok(())
    }

    #[test]
    fn parse_knots_without_root() -> Result<(), Box<dyn std::error::Error>>    {
        let (unparsed, ast::Story(root, knots, _functions)) = ast::Story::parse(include_str!("../tests/knots_without_root.ink"))?;

        match eof::<&str,nom::error::Error<&str>>.parse(unparsed) {
            Ok(_) => {},
            _ => assert!(false, "Incomplete parse. Remaining text: {}:'{}'", unparsed.input_len(), unparsed),
        }

        match root {
            root @ ast::Knot{ signature: ast::Signature { ret: None, ..}, ..  } => {
                assert_eq!(root.signature.name, "__root");
                assert_eq!(root.root.body.trim(), "", "Root body parse error");
                assert!(root.body.len() == 0, "Root stitch body parse error");
            },
            _ => {
                panic!("Invalid parse. Error with root\nRemaining: \n{:?}\n---", unparsed);
            }
        };

        match knots.as_slice() {
            [
                k1 @ ast::Knot{ signature: ast::Signature {..}, ..  }, 
                k2 @ ast::Knot{ signature: ast::Signature {..}, ..  }, 
            ] => {
                assert_eq!(k1.signature.name, "K1");
                assert!(matches!(k1.signature.ret, None));
                assert_ne!(k1.root.body.trim(), "", "K1 body parse error");
                                                                                          
                assert_eq!(k1.body[0].signature.name, "K1_1", "K1_1 body parse error");
                assert!(matches!(k1.signature.ret, None));
                assert_ne!(k1.body[0].body.trim(), "", "K1_1 body parse error");
                                                                                          
                assert_eq!(k2.signature.name, "K2");
                assert!(matches!(k2.signature.ret, None));
                assert_eq!(k2.root.body.trim(), "", "K1 body parse error");
                                                                                          
                assert_eq!(k2.body[0].signature.name, "K2_1", "K2_1 body parse error");
                assert!(matches!(k2.body[0].signature.ret, None));
                assert_ne!(k2.body[0].body.trim(), "", "K2_1 body parse error");
            },
            _ => {
                panic!("Invalid parse.\nFounc {} knots, expected {}\nRemaining: \n{:?}\n---", knots.len(), 2, unparsed);
            }
        };

        Ok(())
    }

    #[test]
    fn parse_knots_and_functions() -> Result<(), Box<dyn std::error::Error>>    {
        let (unparsed, ast::Story(root, knots, functions)) = ast::Story::parse(include_str!("../tests/knots_and_functions.ink"))?;

        match eof::<&str,nom::error::Error<&str>>.parse(unparsed) {
            Ok(_) => {},
            _ => assert!(false, "Incomplete parse. Remaining text: {}:'{}'", unparsed.input_len(), unparsed),
        };

        match root {
            root @ ast::Knot{ signature: ast::Signature {..}, ..  } => {
                assert_eq!(root.signature.name, "__root");
                assert_eq!(root.root.body.trim(), "", "Root body parse error");
                assert!(root.body.len() == 0, "Root stitch body parse error");
            },
        };

        match knots.as_slice() {
            [
                k1 @ ast::Knot{ signature: ast::Signature {..}, ..  }, 
                k2 @ ast::Knot{ signature: ast::Signature {..}, ..  }, 
            ] => {
                assert_eq!(k1.signature.name, "K1");
                assert!(matches!(k1.signature.ret, None));
                assert_ne!(k1.root.body.trim(), "", "K1 body parse error");
                                                                                          
                assert_eq!(k1.body[0].signature.name, "K1_1", "K1_1 body parse error");
                assert!(matches!(k1.signature.ret, None));
                assert_ne!(k1.body[0].body.trim(), "", "K1_1 body parse error");
                                                                                          
                assert_eq!(k2.signature.name, "K2");
                assert!(matches!(k2.signature.ret, None));
                assert_eq!(k2.root.body.trim(), "", "K1 body parse error");
                                                                                          
                assert_eq!(k2.body[0].signature.name, "K2_1", "K2_1 body parse error");
                assert!(matches!(k2.body[0].signature.ret, None));
                assert_ne!(k2.body[0].body.trim(), "", "K2_1 body parse error");
            },
            _ => {
                panic!("Invalid parse.\nFounc {} knots, expected {}\nRemaining: \n{:?}\n---", knots.len(), 2, unparsed);
            }
        };
        match functions.as_slice() {
            [
                f1 @ ast::Function{ signature: ast::Signature {..}, ..  }, 
                f2 @ ast::Function{ signature: ast::Signature {..}, ..  }, 
                f3 @ ast::Function{ signature: ast::Signature {..}, ..  }, 
            ] => {
                assert_eq!(f1.signature.name, "f1");
                assert_ne!(f1.body.trim(), "", "f1 body parse error");

                assert_eq!(f2.signature.name, "f2");
                assert_ne!(f2.body.trim(), "", "f2 body parse error");

                assert_eq!(f3.signature.name, "f3");
                assert_ne!(f3.body.trim(), "", "f3 body parse error");
            },
            _ => { 
                panic!("Invalid parse.\nFound {} functions, expected {}\nRemaining: \n{:?}\n---", functions.len(), 3, unparsed); 
            }
        
        };

        Ok(())
    }

    #[test]
    fn parse_knot_with_parameters() -> Result<(), Box<dyn std::error::Error>>    {
        let (unparsed, knots) = nom::multi::many1(complete(ast::Knot::parse)).parse(include_str!("../tests/knots_with_parameters.ink"))?;
        match knots.as_slice() {
            [
                ast::Knot{ signature: ast::Signature { parameters: k1_parameters, ret: k1_ret, ..}, ..}, 
                ast::Knot{ signature: ast::Signature { parameters: k2_parameters, ret: k2_ret, ..}, ..}, 
                ast::Knot{ signature: ast::Signature { parameters: k3_parameters, ret: k3_ret, ..}, ..}, 
                ast::Knot{ signature: ast::Signature { parameters: k4_parameters, ret: k4_ret, ..}, ..}, 
                ast::Knot{ signature: ast::Signature { parameters: k5_parameters, ret: k5_ret, ..}, ..}, 
                ast::Knot{ signature: ast::Signature { parameters: k6_parameters, ret: k6_ret, ..}, ..}, 
            ] => {
                assert!(matches!(k1_parameters.as_slice(), []), "Expected 0 arguments");
                assert!(matches!(k2_parameters.as_slice(), [ast::Parameter{..}]), "Expected 1 argument");
                assert!(matches!(k3_parameters.as_slice(), [ast::Parameter{..}, ast::Parameter{..}, ast::Parameter{..}]), "Expected 3 arguments");
                assert!(matches!(k4_parameters.as_slice(), [ast::Parameter{refrence:  true,  is_divert: false, ..}]), "Expected 1 argument by refrence");
                assert!(matches!(k5_parameters.as_slice(), [ast::Parameter{refrence:  false, is_divert: true,  ..}]), "Expected 1 divert argument by value");
                assert!(matches!(k6_parameters.as_slice(), [ast::Parameter{refrence:  true,  is_divert: true,  ..}]), "Expected 1 divert argument by refrence");

                assert!(matches!(k1_ret, None), "Exptected no return type");
                assert!(matches!(k2_ret, None), "Exptected no return type");
                assert!(matches!(k3_ret, None), "Exptected no return type");
                assert!(matches!(k4_ret, None), "Exptected no return type");
                assert!(matches!(k5_ret, None), "Exptected no return type");
                assert!(matches!(k6_ret, None), "Exptected no return type");
            },
            _ => { panic!("Invalid parse.\nFound {} knots, expected {}\nRemaining: \n{:?}\n---", knots.len(), 6, unparsed); }, 
        };

        Ok(())
    }

    #[test]
    fn parse_functions_with_parameters() -> Result<(), Box<dyn std::error::Error>>    {
        let (unparsed, functions) = nom::multi::many1(complete(ast::Function::parse)).parse(include_str!("../tests/functions_with_parameters.ink"))?;
        match functions.as_slice() {
            [
                ast::Function{ signature: ast::Signature { parameters: f1_parameters, ret: _k1_ret, ..}, ..}, 
                ast::Function{ signature: ast::Signature { parameters: f2_parameters, ret: _k2_ret, ..}, ..}, 
                ast::Function{ signature: ast::Signature { parameters: f3_parameters, ret: _k3_ret, ..}, ..}, 
                ast::Function{ signature: ast::Signature { parameters: f4_parameters, ret: _k4_ret, ..}, ..}, 
                ast::Function{ signature: ast::Signature { parameters: f5_parameters, ret: _k5_ret, ..}, ..}, 
                ast::Function{ signature: ast::Signature { parameters: f6_parameters, ret: _k6_ret, ..}, ..}, 
            ] => {
                assert!(matches!(f1_parameters.as_slice(), []), "Expected 0 arguments");
                assert!(matches!(f2_parameters.as_slice(), [ast::Parameter{..}]), "Expected 1 argument");
                assert!(matches!(f3_parameters.as_slice(), [ast::Parameter{..}, ast::Parameter{..}, ast::Parameter{..}]), "Expected 3 arguments");
                assert!(matches!(f4_parameters.as_slice(), [ast::Parameter{refrence:  true,  is_divert: false, ..}]), "Expected 1 argument by refrence");
                assert!(matches!(f5_parameters.as_slice(), [ast::Parameter{refrence:  false, is_divert: true,  ..}]), "Expected 1 divert argument by value");
                assert!(matches!(f6_parameters.as_slice(), [ast::Parameter{refrence:  true,  is_divert: true,  ..}]), "Expected 1 divert argument by refrence");

                /* TODO:
                assert!(matches!(f1_ret, None), "Exptected no return type");
                assert!(matches!(f2_ret, None), "Exptected no return type");
                assert!(matches!(f3_ret, None), "Exptected no return type");
                assert!(matches!(f4_ret, None), "Exptected no return type");
                assert!(matches!(f5_ret, None), "Exptected no return type");
                assert!(matches!(f6_ret, None), "Exptected no return type");
                */
            },
            _ => { panic!("Invalid parse.\nFound {} functions, expected {}\nRemaining: \n{:?}\n---", functions.len(), 6, unparsed); }, 
        };

        Ok(())
    }

    #[test]
    fn parse_identifier() -> Result<(), Box<dyn std::error::Error>> {
        let (res, id) = identifier("a;")?; assert_eq!((";", "a"), (res, id.as_str()));
        let (res, id) = identifier("var;")?; assert_eq!((";", "var"), (res, id.as_str()));
        let (res, id) = identifier("a_var;")?; assert_eq!((";", "a_var"), (res, id.as_str()));
        let (res, id) = identifier("_var;")?; assert_eq!((";", "_var"), (res, id.as_str()));
        let (res, id) = identifier("a_var_w_1_number;")?; assert_eq!((";", "a_var_w_1_number"), (res, id.as_str()));

        if let Ok((res, id)) = identifier("1var_w_num;") { panic!("Invalid parse! Should of returned error.\nresult: ('{}','{}')", res, id); }

        Ok(())
    }

    fn match_content<'test>(content: Option<&'test ast::Content<&'test str>>, expected: Option<&'test ast::Content<&'test str>>, unparsed: &'test str) -> bool {
        match (content, expected) { 
            //Text matching
            ( Some(ast::Content::Text(text)), Some(ast::Content::Text(expected)) ) => {
                assert!(text == expected, "invalid text content parse\nParsed:   {:?}\nExpected: {:?}", text, expected);
            }, 
            
            (Some(content_block @ ast::Content::Alternative(ast::Alternative{cases: content, ..})), Some(expected_block @ ast::Content::Alternative(ast::Alternative{cases: expected, ..})))
            => {
                assert!(content.len() == expected.len(), "Diffrent number of cases!\nParsed:   {:?}\nExpected: {:?}", content_block, expected_block);

                for i in expected.keys() {
                    let mut content  = content.get(&i).unwrap().iter();
                    let mut expected = expected.get(&i).unwrap().iter();

                    while match_content( content.next(), expected.next(), unparsed) {};
                }
            },

            (Some(ast::Content::Conditional(content_block @ ast::Conditional{cases: content, ..})), Some(ast::Content::Conditional(expected_block @ ast::Conditional{cases: expected, ..})))
            => {
                for (content, expected) in content.iter().zip(expected.iter()) {
                    let ((content_expr, content), (expected_expr, expected)) = (content, expected);
                    assert_eq!(content_expr, expected_expr);

                    for (content, expected) in content.iter().zip(expected.iter()) {
                        match_content( Some(content), Some(expected), unparsed);
                    }
                }

                match (&content_block.default, &expected_block.default){
                    (Some(content), Some(expected)) => { 
                        for (content, expected) in content.iter().zip(expected.iter()) {
                            match_content( Some(content), Some(expected), unparsed);
                        }
                    },
                    (None, None) => {},

                    (Some(_content), None) => { panic!("Did not expect a default clause in block!\nParsed:   {:?}\nExpected: {:?}", content_block, expected_block) },
                    (None, Some(_expected)) => { panic!("Expected a default clause in block!\nParsed:   {:?}\nExpected: {:?}", content_block, expected_block) },
                }
            },

            (Some(ast::Content::Switch(content_block @ ast::Switch{cases: content, ..})), Some(ast::Content::Switch(expected_block @ ast::Switch{cases: expected, ..})))
            => {
                match_expression(Some(&content_block.compairison), Some(&expected_block.compairison), &unparsed);
                    
                for (content, expected) in content.iter().zip(expected.iter()) {
                    let ((content_expr, content), (expected_expr, expected)) = (content, expected);
                    assert_eq!(content_expr, expected_expr);

                    for (content, expected) in content.iter().zip(expected.iter()) {
                        match_content( Some(content), Some(expected), unparsed);
                    }
                }

                match (&content_block.default, &expected_block.default){
                    (Some(content), Some(expected)) => { 
                        for (content, expected) in content.iter().zip(expected.iter()) {
                            match_content( Some(content), Some(expected), unparsed);
                        }
                    },
                    (None, None) => {},

                    (Some(_content), None) => { panic!("Did not expect a default clause in block!\nParsed:   {:?}\nExpected: {:?}", content_block, expected_block) },
                    (None, Some(_expected)) => { panic!("Expected a default clause in block!\nParsed:   {:?}\nExpected: {:?}", content_block, expected_block) },
                }
            },

            ( Some(content), Some(expected) ) => match (content, expected) {
                  (ast::Content::Logic(_),       ast::Content::Logic(_)      )
                | (ast::Content::Evaluation(_),  ast::Content::Evaluation(_) )
                | (ast::Content::Alternative(_), ast::Content::Alternative(_))
                | (ast::Content::Conditional(_), ast::Content::Conditional(_))
                | (ast::Content::Switch(_),      ast::Content::Switch(_)     )
                | (ast::Content::Branch(_),      ast::Content::Branch(_)     )
                | (ast::Content::Text(_),        ast::Content::Text(_)       )
                | (ast::Content::Newline,        ast::Content::Newline       ) => {},

                (content, expected) => {panic!("Content parsed incorrectly!\nParsed content:   {:?}\nExpected content: {:?}", content, expected) },
            }

            ( None, Some(expected) ) => {
                panic!("Expected content but input left unparsed!\nExpected content: {:?}\nUnparsed input:   {:?}", expected, unparsed);
            },
            ( Some(content), None ) => {
                panic!("Expected end of input but found content!\nParsed content: {:?}\nUnparsed text:  {:?}", content, unparsed);
            },
            ( None, None ) => { return false; },
        };
        
        true
    }

    #[test]
    fn parse_content() -> Result<(), Box<dyn std::error::Error>> {
        let (unparsed, content) = nom::multi::many1(complete(ast::Content::parse)).parse(include_str!("../tests/content.ink"))?;

        println!("Parsed: {:#?}", content);
        let mut content = content.into_iter();
        print_nom_input!(unparsed);

        let mut expected = [
            ast::Content::Text("Line of text"), ast::Content::Newline,

            ast::Content::Text("\tSecond line of text"), ast::Content::Newline,

            ast::Content::Text("Text with delmited newline "), ast::Content::Text("continuing line"), ast::Content::Newline,

            ast::Content::Text("Text with delemiter \\{ block \\}"), ast::Content::Newline,
            
            ast::Content::Text("Text with "), 
            ast::Content::Alternative(ast::Alternative{
                method: ast::AlternateType::Cycle, shuffle: false,
                cases: HashMap::from([
                    (0, vec![ast::Content::Text("cycling")]),
                    (1, vec![ast::Content::Text("repeating")]),
                    (2, vec![ast::Content::Text("alternating")]),
                ])
            }),
            ast::Content::Text(" content"), ast::Content::Newline,

            ast::Content::Text("Text with "), 
            ast::Content::Alternative(ast::Alternative{
                method: ast::AlternateType::Cycle, shuffle: false,
                cases: HashMap::from([
                    (0, vec![ast::Content::Text("cycling")]),
                    (1, vec![
                        ast::Content::Text("nested "),
                        ast::Content::Alternative(ast::Alternative{
                            method: ast::AlternateType::Cycle, shuffle: true,
                            cases: HashMap::from([
                                (0, vec![ast::Content::Text("random!")]),
                                (1, vec![]),
                            ])
                        }),
                        ast::Content::Text(" content"),
                    ]),
                    (2, vec![ast::Content::Text("alternating")]),
                ])
            }),
            ast::Content::Text(" content"), ast::Content::Newline,

            ast::Content::Text("Text with switch "), 
            ast::Content::Switch(ast::Switch{
                compairison: ast::Expression::Variable("cond".to_string()),
                cases: vec![
                    (ast::Expression::Literal(crate::types::Value::Bool(true)), vec![ast::Content::Text("True!")]),
                ],
                default: Some(vec![ast::Content::Text("False")]),
            }),
            ast::Content::Text("."), ast::Content::Newline,
            
            ast::Content::Text("Text with conditional block "),
            ast::Content::Conditional(ast::Conditional{
                cases: vec![
                    (ast::Expression::Variable("cond".to_string()), vec![ast::Content::Text("True!")]),
                ],
                default: Some(vec![ast::Content::Text("False")]),
            }),
            ast::Content::Text("."), ast::Content::Newline,

            ast::Content::Text("Text with conditional "),
            ast::Content::Conditional(ast::Conditional{
                cases: vec![
                    (ast::Expression::Variable("cond".to_string()), vec![ast::Content::Text("  True!")]),
                ],
                default: None,
            }), ast::Content::Text("."),
            ast::Content::Newline, //TODO: End of input should not be newline
        ].into_iter();



        while match_content( (&content.next()).into(), (&expected.next()).into(), &unparsed) {};

        Ok(())
    }

    fn match_expression<'test>(expression: Option<&'test ast::Expression>, expected: Option<&'test ast::Expression>, unparsed: &'test str) -> bool {
        match (expression, expected) { 
            (
                Some(expression @ ast::Expression::Literal(value)),
                Some(expected_expression @ ast::Expression::Literal(expected))
            ) => {
                use crate::types::Value;
                match (value, expected) {
                    (Value::Integer(value), Value::Integer(expected)) => { assert_eq!(value, expected) },
                    (Value::Decimal(value), Value::Decimal(expected)) => { assert_eq!(value, expected) },
                    (Value::String(value), Value::String(expected)) => { assert_eq!(value, expected) },
                    (Value::Bool(value), Value::Bool(expected)) => { assert_eq!(value, expected) },
                    (Value::Divert,    Value::Divert) => { todo!("Divert comparison") },
                    (Value::ListValue, Value::ListValue) => { todo!("List item comparision") },
                    (_, _) => {
                        panic!(
                            "Expression parsed incorrectly!\nParsed expression:   {:?}\nExpected expression: {:?}",
                            expression, expected_expression
                        )
                    }
                }
            },
            //TODO: (Some(ast::Expression::Constant(crate::types::Value)), Some(ast::Expression::Constant(crate::types::Value))) => {},
            (
                Some(expression @ ast::Expression::Variable(name)),
                Some(expected_expression @ ast::Expression::Variable(expected_name)),
            ) => {
                assert!(name == expected_name,
                    "Mis-matched variables!\nParsed expression:   {:?}\nExpected expression: {:?}",
                    expression, expected_expression
                );
            }
            (
                Some(expression @ ast::Expression::UnaryOp(op, inner)),
                Some(expected_expression @ ast::Expression::UnaryOp(expected_op, expected_inner))
            ) => {
                assert!(op == expected_op,
                    "Mis-matched operation for unary operation!\nParsed expression:   {:?}\nExpected expression: {:?}",
                    expression, expected_expression
                );

                match_expression(Some(&inner), Some(&expected_inner), unparsed);
            },
            (
                Some(expression @ ast::Expression::BinOp(op, left, right)),
                Some(expected_expression @ ast::Expression::BinOp(expected_op, expected_left, expected_right)),
            ) => {
                assert!(op == expected_op,
                    "Mis-matched operation for binary operation!\nParsed expression:   {:?}\nExpected expression: {:?}",
                    expression, expected_expression
                );

                match_expression(Some(&left), Some(&expected_left), unparsed);
                match_expression(Some(&right), Some(&expected_right), unparsed);
            },

            ( Some(expression), Some(expected) ) => match (expression, expected) {
                  (ast::Expression::Literal(_),    ast::Expression::Literal(_),    )
                | (ast::Expression::Constant(_),   ast::Expression::Constant(_),   )
                | (ast::Expression::Variable(_),      ast::Expression::Variable(_),)
                | (ast::Expression::UnaryOp(_, _), ast::Expression::UnaryOp(_, _), )
                | (ast::Expression::BinOp(_, _, _),ast::Expression::BinOp(_, _, _),)
                 => {},

                (expression, expected) => {panic!("Expression parsed incorrectly!\nParsed expression:   {:?}\nExpected expression: {:?}", expression, expected) },
            }

            ( None, Some(expected) ) => {
                panic!("Expected expression but input left unparsed!\nExpected expression: {:?}\nUnparsed input:   {:?}", expected, unparsed);
            },
            ( Some(expression), None ) => {
                panic!("Expected end of input but found expression!\nParsed expression: {:?}\nUnparsed text:  {:?}", expression, unparsed);
            },
            ( None, None ) => { return false; },
        };
        
        true
    }
    #[test]
    fn parse_expressions() -> Result<(), Box<dyn std::error::Error>> {
        use crate::types::Value;
        use ast::{Expression, Operation};
        let expected = vec![
            Expression::Literal(Value::Integer(401)),
            Expression::Literal(Value::Decimal(4.1)),
            Expression::Literal(Value::String("string".to_string())),
            Expression::Literal(Value::String("string\\\ndelimited".to_string())),
            Expression::Literal(Value::Bool(true)),
            Expression::Literal(Value::Bool(false)),

            Expression::UnaryOp( Operation::Negate, Box::new(Expression::Variable("a".to_string()) )),
            Expression::BinOp( 
                Operation::Subtract,
                Box::new(Expression::Variable("a".to_string()) ),
                Box::new(Expression::Variable("b".to_string()) ),
            ),
            Expression::BinOp( 
                Operation::Subtract,
                Box::new(Expression::UnaryOp(
                    Operation::Negate,
                    Box::new(Expression::Variable("a".to_string())),
                )),
                Box::new(Expression::Variable("b".to_string()) ),
            ),
            Expression::UnaryOp(
                Operation::Negate,
                Box::new(Expression::BinOp(
                    Operation::Subtract,
                    Box::new(Expression::Variable("a".to_string())),
                    Box::new(Expression::Variable("b".to_string())),
                )),
            ),
            Expression::BinOp( 
                Operation::Equal,
                Box::new(Expression::BinOp( 
                    Operation::Subtract,
                    Box::new(Expression::Variable("a".to_string()) ),
                    Box::new(Expression::BinOp(
                        Operation::Multiply,
                        Box::new(Expression::Variable("b".to_string())),
                        Box::new(Expression::Literal(Value::Integer(2))),
                    )),
                )),
                Box::new(Expression::BinOp( 
                    Operation::Subtract,
                    Box::new(Expression::Variable("a".to_string()) ),
                    Box::new(Expression::BinOp(
                        Operation::Multiply,
                        Box::new(Expression::Variable("b".to_string())),
                        Box::new(Expression::Literal(Value::Integer(2))),
                    )),
                )),
            )
        ];
        
        let (unparsed, expressions) = nom::multi::many1(
            map(
                (complete(Expression::parse), multispace0),
                |(expression, _)|expression,
            )
        ).parse(include_str!("../tests/expressions.ink"))?;

        println!("Parsed: {:?}", expressions);

        for (expression, expected) in expressions.into_iter().zip(expected.into_iter()) {
            match_expression(Some(&expression), Some(&expected), &unparsed);
        }

        eof::<&str, nom::error::Error<&str>>.parse(unparsed)?;

        Ok(())
    }
}
