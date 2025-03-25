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
    combinator::*,
    branch::{alt,},
    sequence::{delimited, preceded,},
};

use crate::{ast, ast::Subprogram, };

impl<I> ast::Story<I> where
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

fn identifier<I>(input: I) -> IResult<I, ast::Identifier> 
where
	for<'p> I: nom::Input + nom::Offset + nom::Compare<&'p str> + nom::FindSubstring<&'p str>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'p> &'p str: nom::FindToken<<I as nom::Input>::Item>,
{
    let (rem, name) = verify(
        //TODO: Add parsers for non-ASCII characters
        recognize(many0(alt((alphanumeric1, tag("_"))))),
        |n: &I| {let c = 
            n.iter_elements()
             .nth(0)
             .expect("Parsed identifier as empty string?!")
             .as_char();

            c.is_alpha() || c == '_'
        }).parse(input)?;

    
    let name = name.iter_elements().map(|c| c.as_char()).collect();

    Ok((rem, name))
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
        for<'p> I: nom::Input + nom::Offset + nom::Compare<&'p str> + nom::FindSubstring<&'p str> + std::fmt::Debug,
        <I as nom::Input>::Item: nom::AsChar,
        for<'p> &'p str: nom::FindToken<<I as nom::Input>::Item>,
{

    #[allow(dead_code)]
    fn parse(input: I) -> IResult<I, Self>
    {
        /* TODO
        if let (rem, expr) = ((space0, tag("~"), space0), ast::Expression::parse).parse(input)
        {
        }
        else if let (rem, branch) = ast::Branch::parse(input) 
        {
        }
        */
        if let Ok((rem, _newline)) = many1(line_ending::<I, nom::error::Error<I>>).parse(input.clone()) { Ok( (rem, ast::Content::Newline) ) }
        else
        {
            let (input, text_length) = peek(fold_many1(
                alt((
                    verify(is_not("\\\n{}"), |fragment: &I| fragment.input_len() > 0), //Literal
                    recognize(preceded(char('\\'), multispace1)), //Escaped whitespace
                    recognize(preceded(char('\\'), anychar)), //Escaped char
                )),
                ||0usize,
                |text_length, fragment: I| text_length + fragment.input_len() 
            )).parse(input)?;

            let (rem, text) = input.take_split(text_length);
            Ok( (rem, Self::Text(text)) )
        }
    }

    /*TODO:
    fn parse_block(input: I) -> IResult<I, Self>
    {
        let mut parser = alt((
            //TODO: ast::Alternative::parse,
            //TODO: ast::Conditional::parse,
            //TODO: ast::Switch::parse,
            //TODO: map(ast::Expression::parse, |expr| Self::Evaluation(expr)),
            Self::parse,
        ));
        let (rem, content) = delimited(tag("{"), parser.into(), tag("}")).parse(input)?;
    }
    */
}
#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn parse_content() -> Result<(), Box<dyn std::error::Error>> {
        let (unparsed, content) = nom::multi::many1(complete(ast::Content::parse)).parse(include_str!("../tests/content.ink"))?;
        let mut content = content.into_iter();

        let mut expected = [
            ast::Content::Text("Line of text"), ast::Content::Newline,

            ast::Content::Text("Second line of text"), ast::Content::Newline,

            ast::Content::Text("Text with delmited newline "), ast::Content::Text("continuing line"), ast::Content::Newline,

            ast::Content::Text("Text with delemiter \\{ block \\}"), ast::Content::Newline,
        //TODO: "Text with condition " /*{cond: True!}*/ "."
        //TODO: "Text with " /*{& cycling|repeating|alternating}*/" content"
        ].into_iter();

        loop {
            match (content.next(), expected.next()) { 
                //Text matching
                ( Some(ast::Content::Text(text)), Some(ast::Content::Text(expected)) ) => {
                    assert!(text == expected, "Error: Invalid text content parse\nParsed:   {:?}\nExpected: {:?}", text, expected);
                }, 
                //Newlines
                ( Some(ast::Content::Text(text)), Some(ast::Content::Newline) ) => {
                    panic!("Expected newline, got a string of text!\nText: {:?}", text);
                },
                ( Some(ast::Content::Newline), Some(ast::Content::Text(expected)) ) => {
                    panic!("Expected text but got a new lines!\nExpected text: {:?}", expected);
                },

                ( None, Some(ast::Content::Text(expected)) ) => {
                    panic!("Expected text but text left unparsed!\nExpected text: {:?}\nUnparsed text: {:?}", expected, unparsed);
                },
                ( None, Some(ast::Content::Newline) ) => {
                    panic!("Expected newline but text left unparsed!\nUnparsed text: {:?}", unparsed);
                },

                ( Some(ast::Content::Text(text)), None ) => {
                    panic!("Expected end of input but found text!\nParsed text: {:?}\nUnparsed text: {:?}", text, unparsed);
                },
                ( Some(ast::Content::Newline), None ) => {
                    panic!("Expected newline but found newline!\nUnparsed text: {:?}", unparsed);
                },

                //End of input
                ( None, None ) => { break; },
                _ => {
                    panic!("Invalid parse!\nRemaining text: {:?}", unparsed);
                } 
            }
        }

        Ok(())
    }
}
